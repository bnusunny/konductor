use rmcp::{
    RoleServer, ServerHandler, ServiceExt,
    handler::server::{
        router::prompt::PromptRouter,
        router::tool::ToolRouter,
        wrapper::Parameters,
    },
    model::*,
    prompt, prompt_handler, prompt_router,
    service::RequestContext,
    tool, tool_handler, tool_router,
    schemars::JsonSchema,
    transport::io::stdio,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::state;

// -- Prompt argument types --

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PhaseArgs {
    #[schemars(description = "Phase number or name (e.g. '01' or '01-auth-system')")]
    pub phase: String,
}

// -- Tool argument types --

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransitionArgs {
    #[schemars(description = "Target step (initialized, discussed, planned, executing, executed, complete, shipped)")]
    pub step: String,
    #[schemars(description = "Phase to set (optional, defaults to current phase)")]
    pub phase: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BlockerArgs {
    #[schemars(description = "Phase the blocker applies to")]
    pub phase: String,
    #[schemars(description = "Reason for the blocker")]
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResolveBlockerArgs {
    #[schemars(description = "Phase to resolve blocker for")]
    pub phase: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PlansListArgs {
    #[schemars(description = "Phase number (optional, defaults to current phase)")]
    pub phase: Option<String>,
}

// -- MCP Server --

#[derive(Clone)]
pub struct KonductorMcp {
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
}

// -- Prompts --

#[prompt_router]
impl KonductorMcp {
    #[prompt(name = "k-init", description = "Initialize a new Konductor project")]
    async fn prompt_init(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "Initialize this project with Konductor. Discover project goals, generate spec documents, and set up the pipeline. Run the konductor-init skill.",
        )]
    }

    #[prompt(name = "k-plan", description = "Plan a phase for execution")]
    async fn prompt_plan(&self, Parameters(args): Parameters<PhaseArgs>) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!("Plan phase {} for execution. Research the ecosystem, create execution plans with tasks and acceptance criteria, and validate them. Run the konductor-plan skill.", args.phase),
        )]
    }

    #[prompt(name = "k-exec", description = "Execute the current phase")]
    async fn prompt_exec(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "Execute the plans for the current phase. Spawn executor subagents to implement each plan following TDD workflow. Run the konductor-exec skill.",
        )]
    }

    #[prompt(name = "k-verify", description = "Verify the current phase")]
    async fn prompt_verify(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "Verify the current phase after execution. Validate that phase goals were achieved using the 3-level verification framework. Run the konductor-verify skill.",
        )]
    }

    #[prompt(name = "k-ship", description = "Ship and finalize the project")]
    async fn prompt_ship(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "Ship and finalize the project. Verify all phases are complete, run cross-phase integration checks, create git tag, and archive project state. Run the konductor-ship skill.",
        )]
    }

    #[prompt(name = "k-next", description = "Advance to the next pipeline step")]
    async fn prompt_next(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "Advance to the next step in the Konductor pipeline. Determine what needs to happen next based on current state and execute it. Run the konductor-next skill.",
        )]
    }

    #[prompt(name = "k-status", description = "Show project status and progress")]
    async fn prompt_status(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "Show current Konductor project status. Display phase progress, current position, recent activity, blockers, and next steps. Run the konductor-status skill.",
        )]
    }

    #[prompt(name = "k-discuss", description = "Discuss and set context for a phase")]
    async fn prompt_discuss(&self, Parameters(args): Parameters<PhaseArgs>) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!("Discuss and set context for phase {} before planning. Have a conversation about approach preferences, library choices, architectural trade-offs, and constraints. Run the konductor-discuss skill.", args.phase),
        )]
    }

    #[prompt(name = "k-map", description = "Analyze and map the existing codebase")]
    async fn prompt_map(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "Analyze and map the existing codebase. Scan file structure, tech stack, architecture patterns, testing setup, and integrations. Run the konductor-map-codebase skill.",
        )]
    }
}

// -- Tools --

#[tool_router]
impl KonductorMcp {
    #[tool(description = "Get current pipeline state from .konductor/state.toml")]
    async fn state_get(&self) -> String {
        match state::read_state() {
            Ok(s) => serde_json::to_string_pretty(&s).unwrap_or_else(|e| format!("Error: {e}")),
            Err(e) => e,
        }
    }

    #[tool(description = "Transition pipeline to a new step. Validates the transition is allowed.")]
    async fn state_transition(&self, Parameters(args): Parameters<TransitionArgs>) -> String {
        match state::transition(&args.step, args.phase.as_deref()) {
            Ok(s) => serde_json::to_string_pretty(&s).unwrap_or_else(|e| format!("Error: {e}")),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Add a blocker to the current phase")]
    async fn state_add_blocker(&self, Parameters(args): Parameters<BlockerArgs>) -> String {
        match state::add_blocker(&args.phase, &args.reason) {
            Ok(s) => serde_json::to_string_pretty(&s).unwrap_or_else(|e| format!("Error: {e}")),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Resolve a blocker for a phase")]
    async fn state_resolve_blocker(&self, Parameters(args): Parameters<ResolveBlockerArgs>) -> String {
        match state::resolve_blocker(&args.phase) {
            Ok(s) => serde_json::to_string_pretty(&s).unwrap_or_else(|e| format!("Error: {e}")),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "List plans for a phase with their wave, status, and completion")]
    async fn plans_list(&self, Parameters(args): Parameters<PlansListArgs>) -> String {
        let phase = match args.phase {
            Some(p) => p,
            None => match state::read_state() {
                Ok(s) => s.current.and_then(|c| c.phase).unwrap_or_default(),
                Err(e) => return format!("Error: {e}"),
            },
        };

        let plans_dir = format!(".konductor/phases/{phase}/plans");
        let dir = Path::new(&plans_dir);
        if !dir.exists() {
            return format!("No plans directory found at {plans_dir}");
        }

        let mut plans: Vec<serde_json::Value> = Vec::new();
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => return format!("Error reading plans: {e}"),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

            // Skip summary files
            if name.ends_with("-summary.md") {
                continue;
            }
            if !name.ends_with(".md") {
                continue;
            }

            let content = fs::read_to_string(&path).unwrap_or_default();
            let (wave, title) = parse_plan_frontmatter(&content);

            // Check if summary exists
            let plan_num = name.trim_end_matches(".md");
            let summary_path = dir.join(format!("{plan_num}-summary.md"));
            let complete = summary_path.exists();

            plans.push(serde_json::json!({
                "file": name,
                "wave": wave,
                "title": title,
                "complete": complete,
            }));
        }

        plans.sort_by_key(|p| p["wave"].as_i64().unwrap_or(0));
        serde_json::to_string_pretty(&serde_json::json!({
            "phase": phase,
            "plans": plans,
            "total": plans.len(),
        }))
        .unwrap_or_else(|e| format!("Error: {e}"))
    }

    #[tool(description = "Get a structured status report of the entire project")]
    async fn status(&self) -> String {
        let state = match state::read_state() {
            Ok(s) => s,
            Err(e) => return e,
        };

        let current = state.current.as_ref();
        let progress = state.progress.as_ref();

        let active_blockers: Vec<_> = state
            .blockers
            .iter()
            .filter(|b| b.resolved != Some(true))
            .collect();

        let step = current.and_then(|c| c.step.as_deref()).unwrap_or("unknown");
        let next_suggestion = match step {
            "initialized" => "Say 'next' to start planning, or 'discuss' to set preferences first.",
            "discussed" => "Say 'next' to begin planning.",
            "planned" => "Say 'exec' to execute the plans.",
            "executing" => "Execution in progress. Wait for completion or check logs.",
            "executed" => "Say 'next' to verify the phase.",
            "complete" => "Say 'next' to move to the next phase, or 'ship' if all phases are done.",
            "shipped" => "Project shipped!",
            "blocked" => "Resolve the blocker, then say 'next' to continue.",
            _ => "Say 'status' to check project state.",
        };

        serde_json::to_string_pretty(&serde_json::json!({
            "project": state.project.as_ref().and_then(|p| p.name.as_deref()),
            "current_phase": current.and_then(|c| c.phase.as_deref()),
            "current_step": step,
            "status": current.and_then(|c| c.status.as_deref()),
            "phases_total": progress.and_then(|p| p.phases_total),
            "phases_complete": progress.and_then(|p| p.phases_complete),
            "current_phase_plans": progress.and_then(|p| p.current_phase_plans),
            "current_phase_plans_complete": progress.and_then(|p| p.current_phase_plans_complete),
            "active_blockers": active_blockers.len(),
            "blockers": active_blockers,
            "last_activity": state.metrics.as_ref().and_then(|m| m.last_activity.as_deref()),
            "total_agent_sessions": state.metrics.as_ref().and_then(|m| m.total_agent_sessions),
            "next_suggestion": next_suggestion,
        }))
        .unwrap_or_else(|e| format!("Error: {e}"))
    }
}

fn parse_plan_frontmatter(content: &str) -> (i64, String) {
    let mut wave = 0i64;
    let mut title = String::new();

    let mut in_frontmatter = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "+++" {
            if in_frontmatter {
                break;
            }
            in_frontmatter = true;
            continue;
        }
        if in_frontmatter {
            if let Some(val) = trimmed.strip_prefix("wave") {
                if let Some(val) = val.trim().strip_prefix('=') {
                    wave = val.trim().parse().unwrap_or(0);
                }
            } else if let Some(val) = trimmed.strip_prefix("title") {
                if let Some(val) = val.trim().strip_prefix('=') {
                    title = val.trim().trim_matches('"').to_string();
                }
            }
        }
    }
    (wave, title)
}

// -- ServerHandler wiring --

impl KonductorMcp {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        }
    }
}

#[tool_handler]
#[prompt_handler]
impl ServerHandler for KonductorMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_prompts()
                .enable_tools()
                .build(),
        )
        .with_instructions("Konductor — spec-driven development orchestrator")
    }
}

pub async fn run() {
    let server = KonductorMcp::new();
    let transport = stdio();
    let service = server.serve(transport).await.expect("Failed to start MCP server");
    service.waiting().await.expect("MCP server error");
}
