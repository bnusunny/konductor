use acp_harness::*;
use anyhow::Result;

fn should_run() -> bool {
    std::env::var("KONDUCTOR_E2E").as_deref() == Ok("1")
}

#[tokio::test(flavor = "current_thread")]
async fn test_rest_api_sam_pipeline() -> Result<()> {
    if !should_run() {
        eprintln!("Skipping rest-api-sam E2E (set KONDUCTOR_E2E=1)");
        return Ok(());
    }

    let tmp = setup_test_dir(Some("rest-api-sam"))?;
    let cwd = tmp.path();
    let spec = std::fs::read_to_string(cwd.join("synthetic-project/spec.md"))?;

    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let handle = connect_acp(cwd).await?;
            let session_id = create_session(&handle, cwd).await?;

            // Step 1: Initialize
            send_prompt(
                &handle,
                &session_id,
                &format!(
                    "Initialize this project with Konductor. Here is the project spec:\n\n{spec}\n\n\
                     This is a small 1-phase project. Generate project.md, requirements.md, roadmap.md \
                     with a single phase. Do NOT ask me questions. Run the konductor-init skill."
                ),
            )
            .await?;

            assert_file_exists(&cwd.join(".konductor/state.toml"), "state.toml");
            assert_file_not_empty(&cwd.join(".konductor/project.md"), "project.md");

            // Step 2: Plan
            send_prompt(
                &handle,
                &session_id,
                "Plan phase 01 for execution. Research the ecosystem, create execution plans \
                 with tasks and acceptance criteria, and validate them. Run the konductor-plan skill.",
            )
            .await?;

            assert_dir_exists(&cwd.join(".konductor/phases"), "phases directory");
            assert_min_file_count(&cwd.join(".konductor/phases"), ".md", 1, "plan files");

            // Step 3: Execute
            send_prompt(
                &handle,
                &session_id,
                "Execute the plans for the current phase. Spawn executor subagents to implement \
                 each plan following TDD workflow. Run the konductor-exec skill.",
            )
            .await?;

            assert_min_file_count(cwd, ".py", 1, "Python files");
            assert_min_file_count(cwd, "test_", 1, "test files");

            // Step 4: Verify hooks (REQ-16)
            let tracking_log = cwd.join(".konductor/.tracking/modified-files.log");
            assert_file_exists(&tracking_log, "hook tracking log");
            assert_file_contains(&tracking_log, ".py", "tracking log has .py entries");

            // Step 5: SAM template checks
            let template = cwd.join("template.yaml");
            assert_file_exists(&template, "template.yaml");
            assert_file_contains(&template, "AWS::Serverless::Function", "SAM function resource");
            assert_file_contains(&template, "AWS::Serverless", "SAM transform");

            if has_command("sam") {
                let (ok, output) = run_command(cwd, "sam", &["validate"]);
                if !ok {
                    eprintln!("[rest-api-sam] sam validate failed (non-fatal): {output}");
                }
            }

            // Step 6: Quality checks
            if has_command("ruff") {
                let (ok, _) = run_command(cwd, "ruff", &["check", "."]);
                assert!(ok, "ruff lint should pass");
            }
            if has_command("pytest") {
                let (ok, _) = run_command(cwd, "pytest", &["-v", "--tb=short"]);
                assert!(ok, "pytest should pass");
            }

            handle.shutdown().await;
            Ok::<_, anyhow::Error>(())
        })
        .await?;

    Ok(())
}
