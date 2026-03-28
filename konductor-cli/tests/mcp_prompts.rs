use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::GetPromptRequestParams,
    service::RunningService,
    transport::{ConfigureCommandExt, TokioChildProcess},
    RoleClient,
};
use std::fs;
use tempfile::TempDir;

type Client = RunningService<RoleClient, ()>;

async fn setup_test() -> Result<(Client, TempDir)> {
    let tmp = TempDir::new()?;
    let konductor_dir = tmp.path().join(".konductor");
    fs::create_dir_all(&konductor_dir)?;

    let bin = env!("CARGO_BIN_EXE_konductor");
    let transport = TokioChildProcess::new(
        tokio::process::Command::new(bin).configure(|cmd| {
            cmd.arg("mcp").current_dir(tmp.path());
        }),
    )?;
    let client = ().serve(transport).await?;
    Ok((client, tmp))
}

fn get_prompt(name: &str, args: Option<serde_json::Value>) -> GetPromptRequestParams {
    let mut params = GetPromptRequestParams::new(name);
    if let Some(v) = args {
        if let Some(obj) = v.as_object() {
            params.arguments = Some(obj.clone());
        }
    }
    params
}

fn prompt_text(result: &rmcp::model::GetPromptResult) -> String {
    result
        .messages
        .iter()
        .filter_map(|m| match &m.content {
            rmcp::model::PromptMessageContent::Text { text } => Some(text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// -- Prompt listing --

#[tokio::test]
async fn test_list_all_prompts() -> Result<()> {
    let (client, _tmp) = setup_test().await?;
    let prompts = client.list_all_prompts().await?;
    assert_eq!(prompts.len(), 12, "Expected 12 prompts, got {}", prompts.len());

    let names: Vec<String> = prompts.iter().map(|p| p.name.to_string()).collect();
    for expected in ["k-spec", "k-init", "k-design", "k-plan", "k-review", "k-exec", "k-verify", "k-ship", "k-next", "k-status", "k-discuss", "k-map"] {
        assert!(names.contains(&expected.to_string()), "Missing prompt: {expected}");
    }

    client.cancel().await?;
    Ok(())
}

// -- No-arg prompts --

#[tokio::test]
async fn test_prompt_k_init() -> Result<()> {
    let (client, _tmp) = setup_test().await?;
    let result = client.get_prompt(get_prompt("k-init", None)).await?;
    let text = prompt_text(&result);
    assert!(text.contains("spec"), "k-init should contain 'spec': {text}");
    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_prompt_k_exec() -> Result<()> {
    let (client, _tmp) = setup_test().await?;
    let result = client.get_prompt(get_prompt("k-exec", None)).await?;
    let text = prompt_text(&result);
    assert!(text.contains("Execute"), "k-exec should contain 'Execute': {text}");
    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_prompt_k_verify() -> Result<()> {
    let (client, _tmp) = setup_test().await?;
    let result = client.get_prompt(get_prompt("k-verify", None)).await?;
    let text = prompt_text(&result);
    assert!(text.contains("Verify"), "k-verify should contain 'Verify': {text}");
    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_prompt_k_ship() -> Result<()> {
    let (client, _tmp) = setup_test().await?;
    let result = client.get_prompt(get_prompt("k-ship", None)).await?;
    let text = prompt_text(&result);
    assert!(text.contains("Ship"), "k-ship should contain 'Ship': {text}");
    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_prompt_k_next() -> Result<()> {
    let (client, _tmp) = setup_test().await?;
    let result = client.get_prompt(get_prompt("k-next", None)).await?;
    let text = prompt_text(&result);
    assert!(text.contains("next"), "k-next should contain 'next': {text}");
    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_prompt_k_status() -> Result<()> {
    let (client, _tmp) = setup_test().await?;
    let result = client.get_prompt(get_prompt("k-status", None)).await?;
    let text = prompt_text(&result);
    assert!(text.contains("status"), "k-status should contain 'status': {text}");
    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_prompt_k_map() -> Result<()> {
    let (client, _tmp) = setup_test().await?;
    let result = client.get_prompt(get_prompt("k-map", None)).await?;
    let text = prompt_text(&result);
    assert!(text.contains("map"), "k-map should contain 'map': {text}");
    client.cancel().await?;
    Ok(())
}

// -- Arg prompts --

#[tokio::test]
async fn test_prompt_k_plan_with_phase() -> Result<()> {
    let (client, _tmp) = setup_test().await?;
    let result = client
        .get_prompt(get_prompt("k-plan", Some(serde_json::json!({"phase": "01"}))))
        .await?;
    let text = prompt_text(&result);
    assert!(text.contains("01"), "k-plan should contain phase '01': {text}");
    assert!(text.contains("plan"), "k-plan should contain 'plan': {text}");
    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn test_prompt_k_discuss_with_phase() -> Result<()> {
    let (client, _tmp) = setup_test().await?;
    let result = client
        .get_prompt(get_prompt("k-discuss", Some(serde_json::json!({"phase": "02"}))))
        .await?;
    let text = prompt_text(&result);
    assert!(text.contains("02"), "k-discuss should contain phase '02': {text}");
    assert!(text.contains("Discuss"), "k-discuss should contain 'Discuss': {text}");
    client.cancel().await?;
    Ok(())
}
