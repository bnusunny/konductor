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
        eprintln!(
            "BLOCKED by konductor: destructive command detected: {}",
            command
        );
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
}
