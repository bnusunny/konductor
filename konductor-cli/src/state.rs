use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const STATE_FILE: &str = ".konductor/state.toml";

const VALID_STEPS: &[&str] = &[
    "initialized",
    "discussed",
    "planned",
    "executing",
    "executed",
    "complete",
    "shipped",
    "blocked",
];

const TRANSITIONS: &[(&str, &str)] = &[
    ("initialized", "discussed"),
    ("initialized", "planned"),
    ("discussed", "planned"),
    ("planned", "executing"),
    ("executing", "executed"),
    ("executed", "complete"),
    ("complete", "shipped"),
    // blocked can transition back to idle states
    ("blocked", "initialized"),
    ("blocked", "planned"),
    ("blocked", "executed"),
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KonductorState {
    pub project: Option<Project>,
    pub current: Option<Current>,
    pub progress: Option<Progress>,
    pub metrics: Option<Metrics>,
    #[serde(default)]
    pub blockers: Vec<Blocker>,
    pub release: Option<Release>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub name: Option<String>,
    pub initialized: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Current {
    pub phase: Option<String>,
    pub step: Option<String>,
    pub status: Option<String>,
    pub plan: Option<String>,
    pub wave: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Progress {
    pub phases_total: Option<i64>,
    pub phases_complete: Option<i64>,
    pub current_phase_plans: Option<i64>,
    pub current_phase_plans_complete: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metrics {
    pub last_activity: Option<String>,
    pub total_agent_sessions: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Blocker {
    pub phase: Option<String>,
    pub step: Option<String>,
    pub reason: Option<String>,
    pub timestamp: Option<String>,
    pub resolved: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Release {
    pub version: Option<String>,
    pub shipped: Option<String>,
}

pub fn read_state() -> Result<KonductorState, String> {
    let path = Path::new(STATE_FILE);
    if !path.exists() {
        return Err("No Konductor project found. Say 'initialize my project' to get started.".into());
    }
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read state.toml: {e}"))?;
    toml::from_str(&content).map_err(|e| format!("Failed to parse state.toml: {e}"))
}

pub fn write_state(state: &KonductorState) -> Result<(), String> {
    let content = toml::to_string_pretty(state).map_err(|e| format!("Failed to serialize state: {e}"))?;
    fs::write(STATE_FILE, content).map_err(|e| format!("Failed to write state.toml: {e}"))
}

pub fn validate_transition(from: &str, to: &str) -> Result<(), String> {
    if !VALID_STEPS.contains(&to) {
        return Err(format!("Invalid step: '{to}'. Valid steps: {}", VALID_STEPS.join(", ")));
    }
    if TRANSITIONS.iter().any(|(f, t)| *f == from && *t == to) {
        Ok(())
    } else {
        Err(format!("Invalid transition: '{from}' → '{to}'"))
    }
}

pub fn transition(step: &str, phase: Option<&str>) -> Result<KonductorState, String> {
    let mut state = read_state()?;
    let current = state.current.as_ref().ok_or("No current state found")?;
    let current_step = current.step.as_deref().unwrap_or("unknown");
    validate_transition(current_step, step)?;

    let current = state.current.as_mut().unwrap();
    current.step = Some(step.to_string());
    current.status = Some("idle".to_string());
    if let Some(p) = phase {
        current.phase = Some(p.to_string());
    }

    if let Some(metrics) = state.metrics.as_mut() {
        metrics.last_activity = Some(Utc::now().to_rfc3339());
    }

    write_state(&state)?;
    Ok(state)
}

pub fn add_blocker(phase: &str, reason: &str) -> Result<KonductorState, String> {
    let mut state = read_state()?;

    state.blockers.push(Blocker {
        phase: Some(phase.to_string()),
        step: state.current.as_ref().and_then(|c| c.step.clone()),
        reason: Some(reason.to_string()),
        timestamp: Some(Utc::now().to_rfc3339()),
        resolved: Some(false),
    });

    if let Some(current) = state.current.as_mut() {
        current.status = Some("blocked".to_string());
    }

    write_state(&state)?;
    Ok(state)
}

pub fn resolve_blocker(phase: &str) -> Result<KonductorState, String> {
    let mut state = read_state()?;

    let found = state.blockers.iter_mut().find(|b| {
        b.phase.as_deref() == Some(phase) && b.resolved != Some(true)
    });

    match found {
        Some(blocker) => {
            blocker.resolved = Some(true);
        }
        None => return Err(format!("No active blocker found for phase '{phase}'")),
    }

    let has_active = state.blockers.iter().any(|b| b.resolved != Some(true));
    if !has_active {
        if let Some(current) = state.current.as_mut() {
            current.status = Some("idle".to_string());
        }
    }

    write_state(&state)?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        assert!(validate_transition("initialized", "planned").is_ok());
        assert!(validate_transition("initialized", "discussed").is_ok());
        assert!(validate_transition("planned", "executing").is_ok());
        assert!(validate_transition("executing", "executed").is_ok());
        assert!(validate_transition("executed", "complete").is_ok());
    }

    #[test]
    fn test_invalid_transitions() {
        assert!(validate_transition("initialized", "executed").is_err());
        assert!(validate_transition("planned", "complete").is_err());
        assert!(validate_transition("executed", "planned").is_err());
    }

    #[test]
    fn test_invalid_step() {
        assert!(validate_transition("initialized", "bogus").is_err());
    }
}
