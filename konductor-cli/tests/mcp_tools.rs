use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParams,
    service::RunningService,
    transport::{ConfigureCommandExt, TokioChildProcess},
    RoleClient,
};
use std::fs;
use tempfile::TempDir;

type Client = RunningService<RoleClient, ()>;

async fn setup_test(state_toml: &str) -> Result<(Client, TempDir)> {
    let tmp = TempDir::new()?;
    let konductor_dir = tmp.path().join(".konductor");
    fs::create_dir_all(konductor_dir.join("phases"))?;
    fs::create_dir_all(konductor_dir.join(".results"))?;

    if !state_toml.is_empty() {
        fs::write(konductor_dir.join("state.toml"), state_toml)?;
    }

    let bin = env!("CARGO_BIN_EXE_konductor");
    let transport = TokioChildProcess::new(
        tokio::process::Command::new(bin).configure(|cmd| {
            cmd.arg("mcp").current_dir(tmp.path());
        }),
    )?;
    let client = ().serve(transport).await?;
    Ok((client, tmp))
}

fn call_tool(name: &str, args: Option<serde_json::Value>) -> CallToolRequestParams {
    let mut params = CallToolRequestParams::default();
    params.name = name.to_string().into();
    params.arguments = args.and_then(|v| v.as_object().cloned());
    params
}

fn tool_text(result: &rmcp::model::CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("")
}

const SAMPLE_STATE: &str = r#"
[project]
name = "test-project"
initialized = "2026-01-01T00:00:00Z"

[current]
phase = "01"
step = "initialized"
status = "idle"
plan = ""
wave = 0

[progress]
phases_total = 3
phases_complete = 0
current_phase_plans = 0
current_phase_plans_complete = 0

[metrics]
last_activity = "2026-01-01T00:00:00Z"
total_agent_sessions = 1
"#;

// -- state_get tests --

#[tokio::test]
async fn test_state_get_returns_state() -> Result<()> {
    let (client, _tmp) = setup_test(SAMPLE_STATE).await?;
    let result = client.call_tool(call_tool("state_get", None)).await?;
    let text = tool_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(parsed["project"]["name"], "test-project");
    assert_eq!(parsed["current"]["step"], "initialized");
    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_state_get_error_no_state() -> Result<()> {
    let (client, _tmp) = setup_test("").await?;
    let result = client.call_tool(call_tool("state_get", None)).await?;
    let text = tool_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(parsed["error"], true);
    assert_eq!(parsed["code"], "STATE_NOT_FOUND");
    client.cancel().await?;
    Ok(())
}

// -- config_get tests --

#[tokio::test]
async fn test_config_get_returns_defaults() -> Result<()> {
    let (client, _tmp) = setup_test(SAMPLE_STATE).await?;
    let result = client.call_tool(call_tool("config_get", None)).await?;
    let text = tool_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(parsed["general"]["default_model"], "claude-sonnet-4");
    assert_eq!(parsed["execution"]["max_wave_parallelism"], 4);
    assert_eq!(parsed["features"]["research"], true);
    client.cancel().await?;
    Ok(())
}

// -- state_transition tests --

#[tokio::test]
async fn test_state_transition_valid() -> Result<()> {
    let (client, _tmp) = setup_test(SAMPLE_STATE).await?;
    let result = client
        .call_tool(call_tool(
            "state_transition",
            Some(serde_json::json!({"step": "planned"})),
        ))
        .await?;
    let text = tool_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(parsed["current"]["step"], "planned");
    assert_eq!(parsed["current"]["status"], "idle");
    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_state_transition_invalid() -> Result<()> {
    let (client, _tmp) = setup_test(SAMPLE_STATE).await?;
    let result = client
        .call_tool(call_tool(
            "state_transition",
            Some(serde_json::json!({"step": "executed"})),
        ))
        .await?;
    let text = tool_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(parsed["error"], true);
    assert_eq!(parsed["code"], "INVALID_TRANSITION");
    client.cancel().await?;
    Ok(())
}

// -- blocker tests --

#[tokio::test]
async fn test_add_and_resolve_blocker() -> Result<()> {
    let (client, _tmp) = setup_test(SAMPLE_STATE).await?;

    // Add blocker
    let result = client
        .call_tool(call_tool(
            "state_add_blocker",
            Some(serde_json::json!({"phase": "01", "reason": "test failure"})),
        ))
        .await?;
    let text = tool_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(parsed["current"]["status"], "blocked");
    assert!(!parsed["blockers"].as_array().unwrap().is_empty());

    // Resolve blocker
    let result = client
        .call_tool(call_tool(
            "state_resolve_blocker",
            Some(serde_json::json!({"phase": "01"})),
        ))
        .await?;
    let text = tool_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(parsed["current"]["status"], "idle");

    client.cancel().await?;
    Ok(())
}

// -- plans_list tests --

#[tokio::test]
async fn test_plans_list_with_plans() -> Result<()> {
    let (client, tmp) = setup_test(SAMPLE_STATE).await?;

    // Create a plan file
    let plans_dir = tmp.path().join(".konductor/phases/01/plans");
    fs::create_dir_all(&plans_dir)?;
    fs::write(
        plans_dir.join("001.md"),
        "+++\nphase = \"01\"\nplan = 1\nwave = 1\ntitle = \"Test Plan\"\n+++\n# Test\n",
    )?;

    let result = client
        .call_tool(call_tool(
            "plans_list",
            Some(serde_json::json!({"phase": "01"})),
        ))
        .await?;
    let text = tool_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(parsed["total"], 1);
    assert_eq!(parsed["plans"][0]["title"], "Test Plan");
    assert_eq!(parsed["plans"][0]["wave"], 1);

    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_plans_list_no_plans_dir() -> Result<()> {
    let (client, _tmp) = setup_test(SAMPLE_STATE).await?;
    let result = client
        .call_tool(call_tool(
            "plans_list",
            Some(serde_json::json!({"phase": "99"})),
        ))
        .await?;
    let text = tool_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(parsed["error"], true);
    assert_eq!(parsed["code"], "NO_PLANS");
    client.cancel().await?;
    Ok(())
}

// -- status tests --

#[tokio::test]
async fn test_status_returns_report() -> Result<()> {
    let (client, _tmp) = setup_test(SAMPLE_STATE).await?;
    let result = client.call_tool(call_tool("status", None)).await?;
    let text = tool_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(parsed["project"], "test-project");
    assert_eq!(parsed["current_phase"], "01");
    assert_eq!(parsed["current_step"], "initialized");
    assert!(parsed["next_suggestion"].as_str().unwrap().contains("next"));
    client.cancel().await?;
    Ok(())
}
