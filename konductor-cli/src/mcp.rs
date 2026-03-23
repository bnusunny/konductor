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
use crate::config;

fn tool_error(code: &str, message: &str) -> String {
    serde_json::json!({
        "error": true,
        "code": code,
        "message": message,
    }).to_string()
}

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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct InitArgs {
    #[schemars(description = "Project name")]
    pub name: String,
    #[schemars(description = "First phase identifier (e.g. '01' or '01-auth-system')")]
    pub phase: String,
    #[schemars(description = "Total number of phases in the roadmap")]
    pub phases_total: i64,
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
            "Initialize this project with Konductor. Detect existing codebase if present, discover project goals through interactive questions, generate spec documents (project.md, requirements.md, roadmap.md), and set up the pipeline. Run the konductor-init skill.",
        )]
    }

    #[prompt(name = "k-plan", description = "Plan a phase for execution")]
    async fn prompt_plan(&self, Parameters(args): Parameters<PhaseArgs>) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!("Plan phase {} for execution. Research the ecosystem, create execution plans with tasks and acceptance criteria, validate them, and run design review if enabled. Run the konductor-plan skill.", args.phase),
        )]
    }

    #[prompt(name = "k-exec", description = "Execute the plans for the current phase")]
    async fn prompt_exec(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "Execute the plans for the current phase. Read state to confirm the phase is planned, spawn executor subagents for each plan by wave, and run code review. Run the konductor-exec skill.",
        )]
    }

    #[prompt(name = "k-verify", description = "Verify the current phase after execution")]
    async fn prompt_verify(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "Verify the current phase after execution. Read state to confirm the phase is executed, then validate that phase goals were achieved using the 3-level verification framework (Exists, Substantive, Wired). Run the konductor-verify skill.",
        )]
    }

    #[prompt(name = "k-ship", description = "Ship and finalize the project after all phases are complete")]
    async fn prompt_ship(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "Ship and finalize the project. Verify all phases are complete, run cross-phase integration checks, create release branch and PR, and archive project state. Run the konductor-ship skill.",
        )]
    }

    #[prompt(name = "k-next", description = "Determine and execute the next pipeline step")]
    async fn prompt_next(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "Determine what needs to happen next based on current state and execute it — planning, execution, verification, or phase advancement. Read state first. Run the konductor-next skill.",
        )]
    }

    #[prompt(name = "k-status", description = "Show project status and progress")]
    async fn prompt_status(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "Show current Konductor project status. Call the status MCP tool, then display phase progress, current step, blockers, and next steps. Run the konductor-status skill.",
        )]
    }

    #[prompt(name = "k-discuss", description = "Discuss and set context for a phase before planning")]
    async fn prompt_discuss(&self, Parameters(args): Parameters<PhaseArgs>) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!("Discuss and set context for phase {} before planning. Have a conversation about approach preferences, library choices, architectural trade-offs, and constraints. Decisions are saved to context.md for the planner. Run the konductor-discuss skill.", args.phase),
        )]
    }

    #[prompt(name = "k-map", description = "Analyze and map the existing codebase structure and tech stack")]
    async fn prompt_map(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "Analyze and map the existing codebase. Scan file structure, tech stack, architecture patterns, testing setup, and integrations. Results are written to .kiro/steering/structure.md and .kiro/steering/tech.md. Run the konductor-map-codebase skill.",
        )]
    }
}

// -- Tools --

#[tool_router]
impl KonductorMcp {
    #[tool(description = "Get current pipeline state from .konductor/state.toml")]
    async fn state_get(&self) -> String {
        match state::read_state() {
            Ok(s) => serde_json::to_string_pretty(&s).unwrap_or_else(|e| tool_error("SERIALIZE_ERROR", &e.to_string())),
            Err(e) => tool_error("STATE_NOT_FOUND", &e),
        }
    }

    #[tool(description = "Initialize a new project state. Creates .konductor/state.toml with project name, first phase, and phase count.")]
    async fn state_init(&self, Parameters(args): Parameters<InitArgs>) -> String {
        match state::init_state(&args.name, &args.phase, args.phases_total) {
            Ok(s) => serde_json::to_string_pretty(&s).unwrap_or_else(|e| tool_error("SERIALIZE_ERROR", &e.to_string())),
            Err(e) => tool_error("INIT_ERROR", &e),
        }
    }

    #[tool(description = "Transition pipeline to a new step. Validates the transition is allowed.")]
    async fn state_transition(&self, Parameters(args): Parameters<TransitionArgs>) -> String {
        match state::transition(&args.step, args.phase.as_deref()) {
            Ok(s) => serde_json::to_string_pretty(&s).unwrap_or_else(|e| tool_error("SERIALIZE_ERROR", &e.to_string())),
            Err(e) => tool_error("INVALID_TRANSITION", &e),
        }
    }

    #[tool(description = "Add a blocker to the current phase")]
    async fn state_add_blocker(&self, Parameters(args): Parameters<BlockerArgs>) -> String {
        match state::add_blocker(&args.phase, &args.reason) {
            Ok(s) => serde_json::to_string_pretty(&s).unwrap_or_else(|e| tool_error("SERIALIZE_ERROR", &e.to_string())),
            Err(e) => tool_error("STATE_NOT_FOUND", &e),
        }
    }

    #[tool(description = "Resolve a blocker for a phase")]
    async fn state_resolve_blocker(&self, Parameters(args): Parameters<ResolveBlockerArgs>) -> String {
        match state::resolve_blocker(&args.phase) {
            Ok(s) => serde_json::to_string_pretty(&s).unwrap_or_else(|e| tool_error("SERIALIZE_ERROR", &e.to_string())),
            Err(e) => tool_error("STATE_NOT_FOUND", &e),
        }
    }

    #[tool(description = "List plans for a phase with their wave, status, and completion")]
    async fn plans_list(&self, Parameters(args): Parameters<PlansListArgs>) -> String {
        let phase = match args.phase {
            Some(p) => p,
            None => match state::read_state() {
                Ok(s) => s.current.and_then(|c| c.phase).unwrap_or_default(),
                Err(e) => return tool_error("STATE_NOT_FOUND", &e),
            },
        };

        let plans_dir = format!(".konductor/phases/{phase}/plans");
        let dir = Path::new(&plans_dir);
        if !dir.exists() {
            return tool_error("NO_PLANS", &format!("No plans directory found at {plans_dir}"));
        }

        let mut plans: Vec<serde_json::Value> = Vec::new();
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => return tool_error("IO_ERROR", &format!("Error reading plans: {e}")),
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
            let fm = parse_plan_frontmatter(&content);

            // Check if summary exists
            let plan_num = name.trim_end_matches(".md");
            let summary_path = dir.join(format!("{plan_num}-summary.md"));
            let complete = summary_path.exists();

            plans.push(serde_json::json!({
                "file": name,
                "plan": fm.plan,
                "wave": fm.wave,
                "title": fm.title,
                "type": fm.plan_type,
                "depends_on": fm.depends_on,
                "requirements": fm.requirements,
                "complete": complete,
            }));
        }

        plans.sort_by_key(|p| p["wave"].as_i64().unwrap_or(0));
        serde_json::to_string_pretty(&serde_json::json!({
            "phase": phase,
            "plans": plans,
            "total": plans.len(),
        }))
        .unwrap_or_else(|e| tool_error("SERIALIZE_ERROR", &e.to_string()))
    }

    #[tool(description = "Get a structured status report of the entire project")]
    async fn status(&self) -> String {
        let state = match state::read_state() {
            Ok(s) => s,
            Err(e) => return tool_error("STATE_NOT_FOUND", &e),
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

        let config_summary = config::read_config().ok();

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
            "config": config_summary,
            "next_suggestion": next_suggestion,
        }))
        .unwrap_or_else(|e| tool_error("SERIALIZE_ERROR", &e.to_string()))
    }

    #[tool(description = "Get current configuration from .konductor/config.toml")]
    async fn config_get(&self) -> String {
        match config::read_config() {
            Ok(c) => serde_json::to_string_pretty(&c).unwrap_or_else(|e| tool_error("SERIALIZE_ERROR", &e.to_string())),
            Err(e) => tool_error("INVALID_CONFIG", &e),
        }
    }

    #[tool(description = "Initialize configuration with defaults. Creates .konductor/config.toml if it doesn't exist. Idempotent — returns current config if file already exists.")]
    async fn config_init(&self) -> String {
        match config::read_config() {
            Ok(c) => {
                // Write (or re-write) with defaults applied for any missing fields
                if let Err(e) = config::write_config(&c) {
                    return tool_error("IO_ERROR", &e);
                }
                serde_json::to_string_pretty(&c).unwrap_or_else(|e| tool_error("SERIALIZE_ERROR", &e.to_string()))
            }
            Err(e) => tool_error("INVALID_CONFIG", &e),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct MustHaves {
    #[serde(default)]
    pub truths: Vec<String>,
    #[serde(default)]
    pub artifacts: Vec<String>,
    #[serde(default)]
    pub key_links: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct PlanFrontmatter {
    #[serde(default)]
    pub phase: String,
    #[serde(default)]
    pub plan: i64,
    #[serde(default)]
    pub wave: i64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub depends_on: Vec<i64>,
    #[serde(default, rename = "type")]
    pub plan_type: String,
    #[serde(default)]
    pub autonomous: bool,
    #[serde(default)]
    pub requirements: Vec<String>,
    #[serde(default)]
    pub files_modified: Vec<String>,
    #[serde(default)]
    pub must_haves: Option<MustHaves>,
}

fn parse_plan_frontmatter(content: &str) -> PlanFrontmatter {
    let mut frontmatter = String::new();
    let mut found_start = false;

    for line in content.lines() {
        if line.trim() == "+++" {
            if found_start {
                break;
            }
            found_start = true;
            continue;
        }
        if found_start {
            frontmatter.push_str(line);
            frontmatter.push('\n');
        }
    }

    if frontmatter.is_empty() {
        return PlanFrontmatter::default();
    }

    toml::from_str(&frontmatter).unwrap_or_default()
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
    let service = match server.serve(transport).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("konductor: MCP server failed to initialize: {e}");
            return;
        }
    };
    if let Err(e) = service.waiting().await {
        eprintln!("konductor: MCP server stopped: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_complete_frontmatter() {
        let content = r#"+++
phase = "01"
plan = 1
wave = 2
title = "User Model"
depends_on = [1, 2]
type = "tdd"
autonomous = true
requirements = ["REQ-01", "REQ-02"]
files_modified = ["src/models/user.rs"]

[must_haves]
truths = ["Users can register"]
artifacts = ["src/models/user.rs"]
key_links = ["User imported by auth"]
+++

# Plan body here
"#;
        let fm = parse_plan_frontmatter(content);
        assert_eq!(fm.phase, "01");
        assert_eq!(fm.plan, 1);
        assert_eq!(fm.wave, 2);
        assert_eq!(fm.title, "User Model");
        assert_eq!(fm.depends_on, vec![1, 2]);
        assert_eq!(fm.plan_type, "tdd");
        assert!(fm.autonomous);
        assert_eq!(fm.requirements, vec!["REQ-01", "REQ-02"]);
        assert_eq!(fm.files_modified, vec!["src/models/user.rs"]);
        let mh = fm.must_haves.unwrap();
        assert_eq!(mh.truths, vec!["Users can register"]);
        assert_eq!(mh.artifacts, vec!["src/models/user.rs"]);
        assert_eq!(mh.key_links, vec!["User imported by auth"]);
    }

    #[test]
    fn test_parse_minimal_frontmatter() {
        let content = "+++\nwave = 1\n+++\n# Body\n";
        let fm = parse_plan_frontmatter(content);
        assert_eq!(fm.wave, 1);
        assert_eq!(fm.phase, "");
        assert_eq!(fm.plan, 0);
        assert!(fm.depends_on.is_empty());
        assert!(fm.must_haves.is_none());
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "# Just a markdown file\nNo frontmatter here.";
        let fm = parse_plan_frontmatter(content);
        assert_eq!(fm.wave, 0);
        assert_eq!(fm.title, "");
    }

    #[test]
    fn test_parse_empty_content() {
        let fm = parse_plan_frontmatter("");
        assert_eq!(fm, PlanFrontmatter::default());
    }

    #[test]
    fn test_parse_frontmatter_with_empty_depends() {
        let content = "+++\nwave = 1\ndepends_on = []\n+++\n";
        let fm = parse_plan_frontmatter(content);
        assert!(fm.depends_on.is_empty());
    }

    #[test]
    fn test_parse_frontmatter_must_haves_partial() {
        let content = "+++\nwave = 1\n\n[must_haves]\ntruths = [\"a\"]\n+++\n";
        let fm = parse_plan_frontmatter(content);
        let mh = fm.must_haves.unwrap();
        assert_eq!(mh.truths, vec!["a"]);
        assert!(mh.artifacts.is_empty());
        assert!(mh.key_links.is_empty());
    }
}
