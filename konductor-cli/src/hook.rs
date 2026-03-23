use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process;

use crate::config;

const STUCK_THRESHOLD: u32 = 5;

#[derive(Deserialize)]
struct HookEvent {
    hook_event_name: String,
    cwd: Option<String>,
    tool_name: Option<String>,
    tool_input: Option<serde_json::Value>,
    tool_response: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Default)]
struct WriteTracker {
    last_path: String,
    consecutive_count: u32,
}

#[derive(Serialize, Deserialize, Clone)]
struct ToolCall {
    tool_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_summary: Option<String>,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_summary: Option<String>,
}

#[derive(Default)]
struct Ledger {
    tool_calls: Vec<ToolCall>,
}

pub fn run() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        process::exit(0);
    }

    let event: HookEvent = match serde_json::from_str(&input) {
        Ok(e) => e,
        Err(_) => process::exit(0),
    };

    match event.hook_event_name.as_str() {
        "PostToolUse" => handle_post_tool_use(&event),
        "PreToolUse" => handle_pre_tool_use(&event),
        _ => process::exit(0),
    }
}

fn handle_post_tool_use(event: &HookEvent) {
    let cwd = match &event.cwd {
        Some(c) => c,
        None => return,
    };

    // Record every tool call to the ledger
    if let Some(tool_name) = &event.tool_name {
        let input_summary = summarize_input(tool_name, &event.tool_input);
        let success = event
            .tool_response
            .as_ref()
            .and_then(|r| r.get("success"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let output_summary = summarize_output(tool_name, &event.tool_response);

        let entry = ToolCall {
            tool_name: tool_name.clone(),
            input_summary,
            success,
            output_summary,
        };
        append_to_ledger(cwd, entry);
    }

    // File tracking (writes only)
    if event.tool_name.as_deref() != Some("write") {
        return;
    }

    let file_path = event
        .tool_input
        .as_ref()
        .and_then(|v| v.get("filePath"))
        .and_then(|v| v.as_str());

    if let Some(path) = file_path {
        let tracking_dir = Path::new(cwd).join(".konductor/.tracking");
        let _ = fs::create_dir_all(&tracking_dir);
        let log_path = tracking_dir.join("modified-files.log");
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
            let _ = writeln!(file, "{}", path);
        }
    }
}

fn handle_pre_tool_use(event: &HookEvent) {
    let cwd = event.cwd.as_deref().unwrap_or(".");

    // Workflow validation applies to all tools
    validate_workflow(cwd, event);

    match event.tool_name.as_deref() {
        Some("shell") => {
            let command = event
                .tool_input
                .as_ref()
                .and_then(|v| v.get("command"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if is_destructive(command) {
                eprintln!(
                    "BLOCKED by konductor: destructive command detected: {}",
                    command
                );
                process::exit(2);
            }

            if modifies_state_file(command) {
                eprintln!(
                    "BLOCKED by konductor: direct state.toml modification detected. Use MCP tools (state_init, state_get, state_transition) instead."
                );
                process::exit(2);
            }

            if modifies_config_file(command) {
                eprintln!(
                    "BLOCKED by konductor: direct config.toml modification detected. Use MCP tools (config_get, config_init) instead."
                );
                process::exit(2);
            }

            // Running a shell command means the agent is doing something new — reset tracker
            reset_write_tracker(cwd);
        }
        Some("write") => {
            let path = event
                .tool_input
                .as_ref()
                .and_then(|v| v.get("path"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if path.ends_with("state.toml") && path.contains(".konductor") {
                eprintln!(
                    "BLOCKED by konductor: direct state.toml write detected. Use MCP tools (state_init, state_get, state_transition) instead."
                );
                process::exit(2);
            }

            if path.ends_with("config.toml") && path.contains(".konductor") {
                eprintln!(
                    "BLOCKED by konductor: direct config.toml write detected. Use MCP tools (config_get, config_init) instead."
                );
                process::exit(2);
            }

            check_stuck_detection(cwd, path);
        }
        _ => {}
    }
}

fn tracker_path(cwd: &str) -> std::path::PathBuf {
    Path::new(cwd).join(".konductor/.tracking/write-tracker.json")
}

fn read_write_tracker(cwd: &str) -> WriteTracker {
    fs::read_to_string(tracker_path(cwd))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_write_tracker(cwd: &str, tracker: &WriteTracker) {
    let path = tracker_path(cwd);
    let _ = fs::create_dir_all(path.parent().unwrap());
    let _ = fs::write(path, serde_json::to_string(tracker).unwrap_or_default());
}

fn reset_write_tracker(cwd: &str) {
    save_write_tracker(cwd, &WriteTracker::default());
}

fn check_stuck_detection(cwd: &str, path: &str) {
    if path.is_empty() {
        return;
    }

    let mut tracker = read_write_tracker(cwd);

    if tracker.last_path == path {
        tracker.consecutive_count += 1;
    } else {
        tracker.last_path = path.to_string();
        tracker.consecutive_count = 1;
    }

    save_write_tracker(cwd, &tracker);

    if tracker.consecutive_count >= STUCK_THRESHOLD {
        eprintln!(
            "BLOCKED by konductor: You have written to `{}` {} times consecutively without running any commands. \
            You may be stuck in a loop. Stop and reconsider:\n\
            1. Read the latest error output carefully\n\
            2. Consider a fundamentally different approach\n\
            3. Check if you're missing an import, dependency, or misunderstanding the API\n\
            4. If unsure, ask the user for guidance\n\n\
            The write counter resets after you run a shell command.",
            path, tracker.consecutive_count
        );
        process::exit(2);
    }
}

// -- Ledger --

fn ledger_path(cwd: &str) -> std::path::PathBuf {
    Path::new(cwd).join(".konductor/.tracking/ledger.jsonl")
}

fn read_ledger(cwd: &str) -> Ledger {
    let content = match fs::read_to_string(ledger_path(cwd)) {
        Ok(s) => s,
        Err(_) => return Ledger::default(),
    };
    let tool_calls = content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    Ledger { tool_calls }
}

fn append_to_ledger(cwd: &str, entry: ToolCall) {
    let path = ledger_path(cwd);
    let _ = fs::create_dir_all(path.parent().unwrap());
    let line = match serde_json::to_string(&entry) {
        Ok(s) => s,
        Err(_) => return,
    };
    // O_APPEND writes are atomic on POSIX for sizes under PIPE_BUF (4KB)
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{}", line);
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let end = s.floor_char_boundary(max);
        format!("{}...", &s[..end])
    }
}

fn summarize_input(tool_name: &str, input: &Option<serde_json::Value>) -> Option<String> {
    let v = input.as_ref()?;
    match tool_name {
        "write" => v.get("path").and_then(|p| p.as_str()).map(|s| s.to_string()),
        "shell" => v.get("command").and_then(|c| c.as_str()).map(|s| truncate(s, 120)),
        "read" => {
            // Extract first path from operations array
            v.get("operations")
                .and_then(|ops| ops.as_array())
                .and_then(|arr| arr.first())
                .and_then(|op| op.get("path"))
                .and_then(|p| p.as_str())
                .map(|s| s.to_string())
        }
        _ => {
            // For MCP tools, compact JSON truncated
            let s = serde_json::to_string(v).unwrap_or_default();
            Some(truncate(&s, 120))
        }
    }
}

fn summarize_output(tool_name: &str, response: &Option<serde_json::Value>) -> Option<String> {
    let v = response.as_ref()?;
    match tool_name {
        "shell" => {
            v.get("result")
                .and_then(|r| r.as_array())
                .and_then(|arr| arr.first())
                .and_then(|item| item.as_str())
                .map(|s| {
                    let trimmed = s.trim();
                    let last_lines: String = trimmed.lines().rev().take(3).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n");
                    truncate(&last_lines, 200)
                })
        }
        _ => None,
    }
}

// -- Workflow validation --

fn validate_workflow(cwd: &str, event: &HookEvent) {
    let tool_name = match event.tool_name.as_deref() {
        Some(n) => n,
        None => return,
    };

    // Only read ledger when we actually need to validate
    let needs_plan_check = tool_name == "write" && {
        let path = event
            .tool_input
            .as_ref()
            .and_then(|v| v.get("path"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        path.contains(".konductor/phases/") && path.ends_with(".md") && path.contains("/plans/")
    };

    let needs_complete_check = tool_name.contains("state_transition") && {
        event
            .tool_input
            .as_ref()
            .and_then(|v| v.get("step"))
            .and_then(|v| v.as_str())
            == Some("complete")
    };

    if !needs_plan_check && !needs_complete_check {
        return;
    }

    let ledger = read_ledger(cwd);

    // Rule: must call state_get or state_transition before writing plan files
    if needs_plan_check {
        let has_state_check = ledger.tool_calls.iter().any(|c| {
            c.tool_name.contains("state_get")
                || c.tool_name.contains("state_transition")
                || c.tool_name.contains("state_init")
        });
        if !has_state_check {
            eprintln!(
                "BLOCKED by konductor: You are writing a plan file without first checking pipeline state. \
                Call the state_get MCP tool first to verify the current phase and step before creating plans."
            );
            process::exit(2);
        }
    }

    // Rule: must run tests before transitioning to 'complete'
    if needs_complete_check {
        let test_patterns = config::read_config()
            .map(|c| c.hooks.test_patterns)
            .unwrap_or_default();

        let has_test_run = ledger.tool_calls.iter().any(|c| {
            c.tool_name == "shell"
                && c.input_summary
                    .as_ref()
                    .map(|s| {
                        let lower = s.to_lowercase();
                        test_patterns.iter().any(|p| lower.contains(p))
                    })
                    .unwrap_or(false)
        });
        if !has_test_run {
            eprintln!(
                "BLOCKED by konductor: You are transitioning to 'complete' without running any tests or linting. \
                Run tests first to verify your changes work, then retry the transition."
            );
            process::exit(2);
        }
    }
}

fn is_destructive(command: &str) -> bool {
    let lower = command.to_lowercase();
    let patterns = [
        "rm -rf /",
        "rm -rf /*",
        "rm -rf ~",
        "drop table",
        "drop database",
        "truncate table",
        "git push --force origin main",
        "git push --force origin master",
        "git push -f origin main",
        "git push -f origin master",
        "git reset --hard origin",
        "mkfs.",
        "dd if=/dev/zero",
        "> /dev/sda",
    ];
    patterns.iter().any(|p| lower.contains(p))
}

fn modifies_state_file(command: &str) -> bool {
    let state_indicators = ["state.toml"];
    let write_commands = ["sed ", "awk ", "echo ", "cat >", "cat>>", "tee ", "printf ", "> ", ">> "];
    let has_state = state_indicators.iter().any(|s| command.contains(s));
    let has_write = write_commands.iter().any(|w| command.contains(w));
    has_state && has_write
}

fn modifies_config_file(command: &str) -> bool {
    let config_indicators = ["config.toml"];
    let write_commands = ["sed ", "awk ", "echo ", "cat >", "cat>>", "tee ", "printf ", "> ", ">> "];
    let has_config = config_indicators.iter().any(|s| command.contains(s));
    let has_write = write_commands.iter().any(|w| command.contains(w));
    has_config && has_write
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs as stdfs;
    use tempfile::TempDir;

    // -- HookEvent deserialization tests --

    #[test]
    fn test_deserialize_post_tool_use() {
        let json = r#"{"hook_event_name":"PostToolUse","cwd":"/tmp","tool_name":"write","tool_input":{"filePath":"src/main.rs"}}"#;
        let event: HookEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.hook_event_name, "PostToolUse");
        assert_eq!(event.tool_name.as_deref(), Some("write"));
        assert_eq!(
            event.tool_input.as_ref().unwrap()["filePath"].as_str(),
            Some("src/main.rs")
        );
    }

    #[test]
    fn test_deserialize_pre_tool_use() {
        let json = r#"{"hook_event_name":"PreToolUse","tool_name":"shell","tool_input":{"command":"cargo test"}}"#;
        let event: HookEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.hook_event_name, "PreToolUse");
        assert_eq!(event.tool_name.as_deref(), Some("shell"));
    }

    #[test]
    fn test_deserialize_missing_optional_fields() {
        let json = r#"{"hook_event_name":"PostToolUse"}"#;
        let event: HookEvent = serde_json::from_str(json).unwrap();
        assert!(event.cwd.is_none());
        assert!(event.tool_name.is_none());
        assert!(event.tool_input.is_none());
    }

    #[test]
    fn test_deserialize_malformed_json() {
        let result: Result<HookEvent, _> = serde_json::from_str("not json");
        assert!(result.is_err());
    }

    // -- File tracking tests --

    #[test]
    fn test_post_tool_use_tracks_write() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_str().unwrap().to_string();
        let event = HookEvent {
            hook_event_name: "PostToolUse".into(),
            cwd: Some(cwd.clone()),
            tool_name: Some("write".into()),
            tool_input: Some(serde_json::json!({"filePath": "src/lib.rs"})),
            tool_response: None,
        };
        handle_post_tool_use(&event);
        let log = stdfs::read_to_string(tmp.path().join(".konductor/.tracking/modified-files.log")).unwrap();
        assert!(log.contains("src/lib.rs"));
    }

    #[test]
    fn test_post_tool_use_skips_non_write() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_str().unwrap().to_string();
        let event = HookEvent {
            hook_event_name: "PostToolUse".into(),
            cwd: Some(cwd),
            tool_name: Some("read".into()),
            tool_input: Some(serde_json::json!({"filePath": "src/lib.rs"})),
            tool_response: None,
        };
        handle_post_tool_use(&event);
        assert!(!tmp.path().join(".konductor/.tracking/modified-files.log").exists());
    }

    #[test]
    fn test_post_tool_use_skips_missing_cwd() {
        let event = HookEvent {
            hook_event_name: "PostToolUse".into(),
            cwd: None,
            tool_name: Some("write".into()),
            tool_input: Some(serde_json::json!({"filePath": "src/lib.rs"})),
            tool_response: None,
        };
        // Should not panic
        handle_post_tool_use(&event);
    }

    #[test]
    fn test_post_tool_use_skips_missing_file_path() {
        let tmp = TempDir::new().unwrap();
        let event = HookEvent {
            hook_event_name: "PostToolUse".into(),
            cwd: Some(tmp.path().to_str().unwrap().into()),
            tool_name: Some("write".into()),
            tool_input: Some(serde_json::json!({})),
            tool_response: None,
        };
        handle_post_tool_use(&event);
        assert!(!tmp.path().join(".konductor/.tracking/modified-files.log").exists());
    }

    // -- Destructive command tests --

    #[test]
    fn test_blocks_destructive_commands() {
        for cmd in [
            "rm -rf /",
            "rm -rf /*",
            "rm -rf ~",
            "DROP TABLE users",
            "drop database mydb",
            "TRUNCATE TABLE logs",
            "git push --force origin main",
            "git push --force origin master",
            "git push -f origin main",
            "git push -f origin master",
            "git reset --hard origin",
            "mkfs.ext4 /dev/sda1",
            "dd if=/dev/zero of=/dev/sda",
            "> /dev/sda",
        ] {
            assert!(is_destructive(cmd), "Should block: {}", cmd);
        }
    }

    #[test]
    fn test_allows_safe_commands() {
        for cmd in [
            "cargo test",
            "npm run build",
            "git add src/main.rs",
            "git commit -m 'feat: add feature'",
            "rm src/temp.rs",
            "git push origin feature-branch",
            "git push --force origin feature-branch",
            "git reset --hard HEAD~1",
        ] {
            assert!(!is_destructive(cmd), "Should allow: {}", cmd);
        }
    }

    #[test]
    fn test_destructive_case_insensitive() {
        assert!(is_destructive("RM -RF /"));
        assert!(is_destructive("Drop Table users"));
        assert!(is_destructive("GIT PUSH --FORCE ORIGIN MAIN"));
    }

    #[test]
    fn test_pre_tool_use_skips_non_shell() {
        let event = HookEvent {
            hook_event_name: "PreToolUse".into(),
            cwd: None,
            tool_name: Some("write".into()),
            tool_input: Some(serde_json::json!({"command": "rm -rf /"})),
            tool_response: None,
        };
        // Should not panic or exit — non-shell tools are skipped
        handle_pre_tool_use(&event);
    }

    #[test]
    fn test_pre_tool_use_handles_missing_command() {
        let event = HookEvent {
            hook_event_name: "PreToolUse".into(),
            cwd: None,
            tool_name: Some("shell".into()),
            tool_input: Some(serde_json::json!({})),
            tool_response: None,
        };
        // Empty command is not destructive — should not panic
        handle_pre_tool_use(&event);
    }

    // -- Stuck detection tests --

    #[test]
    fn test_write_tracker_increments_same_file() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_str().unwrap();

        for i in 1..STUCK_THRESHOLD {
            check_stuck_detection(cwd, "src/main.rs");
            let tracker = read_write_tracker(cwd);
            assert_eq!(tracker.consecutive_count, i);
            assert_eq!(tracker.last_path, "src/main.rs");
        }
    }

    #[test]
    fn test_write_tracker_resets_on_different_file() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_str().unwrap();

        check_stuck_detection(cwd, "src/main.rs");
        check_stuck_detection(cwd, "src/main.rs");
        check_stuck_detection(cwd, "src/lib.rs"); // different file

        let tracker = read_write_tracker(cwd);
        assert_eq!(tracker.consecutive_count, 1);
        assert_eq!(tracker.last_path, "src/lib.rs");
    }

    #[test]
    fn test_write_tracker_resets_on_shell_command() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_str().unwrap();

        check_stuck_detection(cwd, "src/main.rs");
        check_stuck_detection(cwd, "src/main.rs");
        reset_write_tracker(cwd);

        let tracker = read_write_tracker(cwd);
        assert_eq!(tracker.consecutive_count, 0);
    }

    #[test]
    fn test_write_tracker_skips_empty_path() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_str().unwrap();

        check_stuck_detection(cwd, "");
        assert!(!tracker_path(cwd).exists());
    }

    #[test]
    fn test_write_tracker_default_on_missing_file() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_str().unwrap();

        let tracker = read_write_tracker(cwd);
        assert_eq!(tracker.consecutive_count, 0);
        assert_eq!(tracker.last_path, "");
    }

    // -- Ledger tests --

    #[test]
    fn test_ledger_append_and_read() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_str().unwrap();

        append_to_ledger(cwd, ToolCall {
            tool_name: "shell".into(),
            input_summary: Some("cargo test".into()),
            success: true,
            output_summary: None,
        });
        append_to_ledger(cwd, ToolCall {
            tool_name: "write".into(),
            input_summary: Some("src/main.rs".into()),
            success: true,
            output_summary: None,
        });

        let ledger = read_ledger(cwd);
        assert_eq!(ledger.tool_calls.len(), 2);
        assert_eq!(ledger.tool_calls[0].tool_name, "shell");
        assert_eq!(ledger.tool_calls[1].tool_name, "write");
    }

    #[test]
    fn test_ledger_append_is_additive() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_str().unwrap();

        for i in 0..10 {
            append_to_ledger(cwd, ToolCall {
                tool_name: format!("tool_{}", i),
                input_summary: None,
                success: true,
                output_summary: None,
            });
        }

        let ledger = read_ledger(cwd);
        assert_eq!(ledger.tool_calls.len(), 10);
        assert_eq!(ledger.tool_calls[0].tool_name, "tool_0");
        assert_eq!(ledger.tool_calls[9].tool_name, "tool_9");
    }

    #[test]
    fn test_ledger_default_on_missing() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_str().unwrap();

        let ledger = read_ledger(cwd);
        assert!(ledger.tool_calls.is_empty());
    }

    // -- Summarize tests --

    #[test]
    fn test_summarize_input_shell() {
        let input = serde_json::json!({"command": "cargo test --release"});
        let result = summarize_input("shell", &Some(input));
        assert_eq!(result, Some("cargo test --release".into()));
    }

    #[test]
    fn test_summarize_input_write() {
        let input = serde_json::json!({"path": "/tmp/foo.rs"});
        let result = summarize_input("write", &Some(input));
        assert_eq!(result, Some("/tmp/foo.rs".into()));
    }

    #[test]
    fn test_summarize_input_read() {
        let input = serde_json::json!({"operations": [{"mode": "Line", "path": "/tmp/bar.rs"}]});
        let result = summarize_input("read", &Some(input));
        assert_eq!(result, Some("/tmp/bar.rs".into()));
    }

    #[test]
    fn test_summarize_input_none() {
        let result = summarize_input("shell", &None);
        assert!(result.is_none());
    }

    // -- Workflow validation tests --

    #[test]
    fn test_workflow_plan_write_allowed_after_state_get() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_str().unwrap();

        // Simulate state_get in ledger
        append_to_ledger(cwd, ToolCall {
            tool_name: "@konductor/state_get".into(),
            input_summary: None,
            success: true,
            output_summary: None,
        });

        let event = HookEvent {
            hook_event_name: "PreToolUse".into(),
            cwd: Some(cwd.into()),
            tool_name: Some("write".into()),
            tool_input: Some(serde_json::json!({"path": "/proj/.konductor/phases/01/plans/plan-1.md"})),
            tool_response: None,
        };
        // Should not panic/exit
        validate_workflow(cwd, &event);
    }

    #[test]
    fn test_workflow_non_plan_write_always_allowed() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_str().unwrap();
        // Empty ledger — no state_get

        let event = HookEvent {
            hook_event_name: "PreToolUse".into(),
            cwd: Some(cwd.into()),
            tool_name: Some("write".into()),
            tool_input: Some(serde_json::json!({"path": "src/main.rs"})),
            tool_response: None,
        };
        // Should not panic — not a plan file
        validate_workflow(cwd, &event);
    }

    #[test]
    fn test_workflow_transition_complete_allowed_after_tests() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_str().unwrap();

        append_to_ledger(cwd, ToolCall {
            tool_name: "shell".into(),
            input_summary: Some("cargo test".into()),
            success: true,
            output_summary: None,
        });

        let event = HookEvent {
            hook_event_name: "PreToolUse".into(),
            cwd: Some(cwd.into()),
            tool_name: Some("@konductor/state_transition".into()),
            tool_input: Some(serde_json::json!({"step": "complete"})),
            tool_response: None,
        };
        // Should not panic
        validate_workflow(cwd, &event);
    }

    #[test]
    fn test_workflow_transition_non_complete_always_allowed() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_str().unwrap();
        // Empty ledger — no tests run

        let event = HookEvent {
            hook_event_name: "PreToolUse".into(),
            cwd: Some(cwd.into()),
            tool_name: Some("@konductor/state_transition".into()),
            tool_input: Some(serde_json::json!({"step": "executing"})),
            tool_response: None,
        };
        // Should not panic — not transitioning to complete
        validate_workflow(cwd, &event);
    }
}
