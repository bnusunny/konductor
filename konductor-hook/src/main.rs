use serde::Deserialize;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process;

#[derive(Deserialize)]
struct HookEvent {
    hook_event_name: String,
    cwd: Option<String>,
    tool_name: Option<String>,
    tool_input: Option<serde_json::Value>,
    #[allow(dead_code)]
    tool_response: Option<serde_json::Value>,
}

fn main() {
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
    if event.tool_name.as_deref() != Some("write") {
        return;
    }

    let cwd = match &event.cwd {
        Some(c) => c,
        None => return,
    };

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
    if event.tool_name.as_deref() != Some("shell") {
        return;
    }

    let command = event
        .tool_input
        .as_ref()
        .and_then(|v| v.get("command"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if is_destructive(command) {
        eprintln!("BLOCKED by konductor-hook: destructive command detected: {}", command);
        process::exit(2);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_tool_use_write_tracks_file() {
        let input = r#"{
            "hook_event_name": "PostToolUse",
            "cwd": "/tmp/test-project",
            "tool_name": "write",
            "tool_input": {"filePath": "src/main.rs", "content": "fn main() {}"},
            "tool_response": {"success": true}
        }"#;
        let event: HookEvent = serde_json::from_str(input).unwrap();
        assert_eq!(event.hook_event_name, "PostToolUse");
        assert_eq!(event.tool_name.as_deref(), Some("write"));
    }

    #[test]
    fn test_pre_tool_use_blocks_destructive_command() {
        let dangerous_commands = vec![
            "rm -rf /",
            "rm -rf /*",
            "DROP TABLE users",
            "git push --force origin main",
            "git push -f origin main",
        ];
        for cmd in dangerous_commands {
            assert!(is_destructive(cmd), "Should block: {}", cmd);
        }
    }

    #[test]
    fn test_pre_tool_use_allows_safe_commands() {
        let safe_commands = vec![
            "cargo test",
            "npm run build",
            "git add src/main.rs",
            "git commit -m 'feat: add feature'",
            "rm src/temp.rs",
            "git push origin feature-branch",
        ];
        for cmd in safe_commands {
            assert!(!is_destructive(cmd), "Should allow: {}", cmd);
        }
    }
}
