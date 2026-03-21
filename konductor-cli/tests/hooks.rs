use std::process::{Command, Stdio};
use std::io::Write;
use tempfile::TempDir;

fn run_hook(input: &str, cwd: &str) -> (i32, String, String) {
    let bin = env!("CARGO_BIN_EXE_konductor");
    let mut child = Command::new(bin)
        .arg("hook")
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn konductor hook");

    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

#[test]
fn test_hook_post_tool_use_tracks_write() {
    let tmp = TempDir::new().unwrap();
    let cwd = tmp.path().to_str().unwrap();
    let input = serde_json::json!({
        "hook_event_name": "PostToolUse",
        "cwd": cwd,
        "tool_name": "write",
        "tool_input": { "filePath": "src/main.rs" }
    });

    let (code, _, _) = run_hook(&input.to_string(), cwd);
    assert_eq!(code, 0);

    let log = std::fs::read_to_string(tmp.path().join(".konductor/.tracking/modified-files.log")).unwrap();
    assert!(log.contains("src/main.rs"));
}

#[test]
fn test_hook_post_tool_use_ignores_non_write() {
    let tmp = TempDir::new().unwrap();
    let cwd = tmp.path().to_str().unwrap();
    let input = serde_json::json!({
        "hook_event_name": "PostToolUse",
        "cwd": cwd,
        "tool_name": "read",
        "tool_input": { "filePath": "src/main.rs" }
    });

    let (code, _, _) = run_hook(&input.to_string(), cwd);
    assert_eq!(code, 0);
    assert!(!tmp.path().join(".konductor/.tracking/modified-files.log").exists());
}

#[test]
fn test_hook_pre_tool_use_blocks_destructive_command() {
    let tmp = TempDir::new().unwrap();
    let cwd = tmp.path().to_str().unwrap();
    let input = serde_json::json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "shell",
        "tool_input": { "command": "rm -rf /" }
    });

    let (code, _, stderr) = run_hook(&input.to_string(), cwd);
    assert_eq!(code, 2, "destructive command should exit with code 2");
    assert!(stderr.contains("BLOCKED"), "stderr should contain BLOCKED: {stderr}");
}

#[test]
fn test_hook_pre_tool_use_allows_safe_command() {
    let tmp = TempDir::new().unwrap();
    let cwd = tmp.path().to_str().unwrap();
    let input = serde_json::json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "shell",
        "tool_input": { "command": "cargo test" }
    });

    let (code, _, _) = run_hook(&input.to_string(), cwd);
    assert_eq!(code, 0);
}

#[test]
fn test_hook_malformed_json_exits_cleanly() {
    let tmp = TempDir::new().unwrap();
    let (code, _, _) = run_hook("not valid json", tmp.path().to_str().unwrap());
    assert_eq!(code, 0, "malformed input should exit 0 (no-op)");
}

#[test]
fn test_hook_unknown_event_exits_cleanly() {
    let tmp = TempDir::new().unwrap();
    let input = serde_json::json!({
        "hook_event_name": "SomeUnknownEvent",
        "tool_name": "write"
    });

    let (code, _, _) = run_hook(&input.to_string(), tmp.path().to_str().unwrap());
    assert_eq!(code, 0, "unknown event should exit 0 (no-op)");
}
