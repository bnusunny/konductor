use agent_client_protocol as acp;
use acp::Agent as _;
use anyhow::{Context, Result};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

/// Repo root, resolved at runtime from CARGO_MANIFEST_DIR.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // tests/
        .unwrap()
        .parent() // repo root
        .unwrap()
        .to_path_buf()
}

/// Konductor binary built by the workspace.
fn konductor_bin() -> PathBuf {
    let root = repo_root();
    for profile in ["release", "debug"] {
        let bin = root.join(format!("konductor-cli/target/{profile}/konductor"));
        if bin.exists() {
            return bin;
        }
    }
    panic!("konductor binary not found — run `cargo build` in konductor-cli/ first");
}

/// Create a temp dir with konductor installed and synthetic project spec copied in.
pub fn setup_test_dir(project: Option<&str>) -> Result<TempDir> {
    let tmp = TempDir::new()?;
    let root = repo_root();
    let base = tmp.path();

    // Install konductor binary
    let bin_dir = base.join("bin");
    std::fs::create_dir_all(&bin_dir)?;
    std::fs::copy(konductor_bin(), bin_dir.join("konductor"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            bin_dir.join("konductor"),
            std::fs::Permissions::from_mode(0o755),
        )?;
    }

    // Copy agents, skills, steering
    copy_dir_recursive(&root.join("agents"), &base.join(".kiro/agents"))?;
    copy_dir_recursive(&root.join("skills"), &base.join(".kiro/skills"))?;
    copy_dir_recursive(&root.join(".kiro/steering"), &base.join(".kiro/steering"))?;

    // Create .konductor with config
    let konductor_dir = base.join(".konductor");
    std::fs::create_dir_all(&konductor_dir)?;
    std::fs::copy(
        root.join(".konductor/config.toml"),
        konductor_dir.join("config.toml"),
    )?;

    // Copy synthetic project spec if specified
    if let Some(project) = project {
        let spec_src = root.join(format!("tests/e2e/synthetic-projects/{project}"));
        copy_dir_recursive(&spec_src, &base.join("synthetic-project"))?;
    }

    Ok(tmp)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src).with_context(|| format!("reading {}", src.display()))? {
        let entry = entry?;
        let dest = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &dest)?;
        } else {
            std::fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

/// ACP client that auto-approves all permission requests and collects agent output.
pub struct TestClient {
    output: RefCell<Vec<String>>,
}

impl TestClient {
    pub fn new() -> Self {
        Self {
            output: RefCell::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl acp::Client for TestClient {
    async fn request_permission(
        &self,
        args: acp::RequestPermissionRequest,
    ) -> acp::Result<acp::RequestPermissionResponse> {
        let option_id = args
            .options
            .iter()
            .find(|o| {
                matches!(
                    o.kind,
                    acp::PermissionOptionKind::AllowOnce | acp::PermissionOptionKind::AllowAlways
                )
            })
            .map(|o| o.option_id.clone())
            .unwrap_or_else(|| args.options[0].option_id.clone());

        Ok(acp::RequestPermissionResponse::new(
            acp::RequestPermissionOutcome::Selected(acp::SelectedPermissionOutcome::new(option_id)),
        ))
    }

    async fn session_notification(
        &self,
        args: acp::SessionNotification,
    ) -> acp::Result<(), acp::Error> {
        if let acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk { content, .. }) =
            args.update
        {
            if let acp::ContentBlock::Text(text) = content {
                self.output.borrow_mut().push(text.text);
            }
        }
        Ok(())
    }
}

/// Handle to a running ACP session. Call `shutdown()` to clean up.
pub struct AcpHandle {
    pub conn: acp::ClientSideConnection,
    child: tokio::process::Child,
    io_task: tokio::task::JoinHandle<()>,
}

impl AcpHandle {
    /// Shut down the ACP connection and kill the child process.
    pub async fn shutdown(mut self) {
        self.io_task.abort();
        let _ = self.child.kill().await;
    }
}

/// Spawn `kiro-cli acp --agent konductor` and return an ACP handle.
///
/// Must be called from within a `tokio::task::LocalSet`.
pub async fn connect_acp(cwd: &Path) -> Result<AcpHandle> {
    let bin_dir = cwd.join("bin");
    let current_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{current_path}", bin_dir.display());

    let mut child = tokio::process::Command::new("kiro-cli")
        .args(["acp", "--agent", "konductor"])
        .current_dir(cwd)
        .env("PATH", &new_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .context("failed to spawn kiro-cli — is it installed and on PATH?")?;

    eprintln!("[acp-harness] Spawned kiro-cli acp, cwd={}", cwd.display());

    let stdin = child.stdin.take().unwrap().compat_write();
    let stdout = child.stdout.take().unwrap().compat();

    let (conn, handle_io) =
        acp::ClientSideConnection::new(TestClient::new(), stdin, stdout, |fut| {
            tokio::task::spawn_local(fut);
        });

    let io_task = tokio::task::spawn_local(async move {
        if let Err(e) = handle_io.await {
            eprintln!("ACP I/O error: {e}");
        }
    });

    Ok(AcpHandle {
        conn,
        child,
        io_task,
    })
}

/// Initialize ACP connection and create a session.
pub async fn create_session(handle: &AcpHandle, cwd: &Path) -> Result<acp::SessionId> {
    handle
        .conn
        .initialize(
            acp::InitializeRequest::new(acp::ProtocolVersion::V1).client_info(
                acp::Implementation::new("acp-harness", "0.1.0")
                    .title("Konductor ACP Test Harness"),
            ),
        )
        .await
        .context("ACP initialize failed")?;

    let response = handle
        .conn
        .new_session(acp::NewSessionRequest::new(cwd))
        .await
        .context("ACP new_session failed")?;

    eprintln!("[acp-harness] Session created: {:?}", response.session_id);
    Ok(response.session_id)
}

/// Send a prompt and wait for completion with timeout.
pub async fn send_prompt(
    handle: &AcpHandle,
    session_id: &acp::SessionId,
    message: &str,
) -> Result<acp::PromptResponse> {
    let timeout_secs: u64 = std::env::var("KONDUCTOR_TIMEOUT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300);

    eprintln!(
        "[acp-harness] Sending prompt ({} chars), timeout={timeout_secs}s",
        message.len()
    );

    let prompt = acp::PromptRequest::new(
        session_id.clone(),
        vec![acp::ContentBlock::Text(acp::TextContent::new(message))],
    );

    let response =
        tokio::time::timeout(Duration::from_secs(timeout_secs), handle.conn.prompt(prompt))
            .await
            .context("prompt timed out")?
            .context("prompt failed")?;

    eprintln!(
        "[acp-harness] Prompt completed: stop_reason={:?}",
        response.stop_reason
    );
    Ok(response)
}

// -- Assertion helpers --

pub fn assert_file_exists(path: &Path, label: &str) {
    assert!(path.is_file(), "{label} missing: {}", path.display());
}

pub fn assert_file_not_empty(path: &Path, label: &str) {
    assert!(path.is_file(), "{label} missing: {}", path.display());
    let meta = std::fs::metadata(path).unwrap();
    assert!(meta.len() > 0, "{label} is empty: {}", path.display());
}

pub fn assert_dir_exists(path: &Path, label: &str) {
    assert!(path.is_dir(), "{label} missing: {}", path.display());
}

pub fn assert_min_file_count(dir: &Path, pattern: &str, min: usize, label: &str) {
    let count = count_files_matching(dir, pattern, 0, 2);
    if count < min {
        eprintln!(
            "[acp-harness] assert_min_file_count FAILED for pattern '{pattern}' in {}",
            dir.display()
        );
        eprintln!("[acp-harness] All files in dir (depth 2):");
        list_all_files(dir, 0, 2);
    }
    assert!(
        count >= min,
        "{label}: found {count}, need at least {min} in {}",
        dir.display()
    );
}

pub fn assert_file_contains(path: &Path, pattern: &str, label: &str) {
    assert!(path.is_file(), "{label}: file missing: {}", path.display());
    let content = std::fs::read_to_string(path).unwrap();
    assert!(
        content.contains(pattern),
        "{label}: '{}' not found in {}",
        pattern,
        path.display()
    );
}

fn count_files_matching(dir: &Path, pattern: &str, depth: usize, max_depth: usize) -> usize {
    if depth > max_depth {
        return 0;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && !path.ends_with(".konductor") {
            count += count_files_matching(&path, pattern, depth + 1, max_depth);
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.contains(pattern) {
                count += 1;
            }
        }
    }
    count
}

/// Print all files in a directory recursively for debugging.
pub fn list_all_files(dir: &Path, depth: usize, max_depth: usize) {
    if depth > max_depth {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let indent = "  ".repeat(depth + 1);
        if path.is_dir() {
            eprintln!(
                "{indent}{}/",
                path.file_name().unwrap_or_default().to_string_lossy()
            );
            list_all_files(&path, depth + 1, max_depth);
        } else {
            eprintln!(
                "{indent}{}",
                path.file_name().unwrap_or_default().to_string_lossy()
            );
        }
    }
}

pub fn has_command(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn run_command(dir: &Path, cmd: &str, args: &[&str]) -> (bool, String) {
    match std::process::Command::new(cmd).args(args).current_dir(dir).output() {
        Ok(o) => (
            o.status.success(),
            String::from_utf8_lossy(&o.stdout).to_string(),
        ),
        Err(_) => (false, String::new()),
    }
}
