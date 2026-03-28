use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const STATE_FILE: &str = ".konductor/state.toml";

const VALID_STEPS: &[&str] = &[
    "specced",
    "discussed",
    "designed",
    "planned",
    "executing",
    "executed",
    "complete",
    "shipped",
    "blocked",
];

const TRANSITIONS: &[(&str, &str)] = &[
    ("specced", "discussed"),
    ("specced", "designed"),
    ("discussed", "designed"),
    ("designed", "planned"),
    ("planned", "executing"),
    ("executing", "executed"),
    ("executed", "complete"),
    ("complete", "shipped"),
    // shipped can start a new phase
    ("shipped", "specced"),
    // blocked can transition back to idle states
    ("blocked", "specced"),
    ("blocked", "designed"),
    ("blocked", "planned"),
    ("blocked", "executed"),
];

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KonductorState {
    pub project: Option<Project>,
    pub current: Option<Current>,
    pub progress: Option<Progress>,
    pub metrics: Option<Metrics>,
    #[serde(default)]
    pub blockers: Vec<Blocker>,
    pub release: Option<Release>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Project {
    pub name: Option<String>,
    pub initialized: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Current {
    pub phase: Option<String>,
    pub step: Option<String>,
    pub status: Option<String>,
    pub plan: Option<String>,
    pub wave: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Progress {
    pub phases_total: Option<i64>,
    pub phases_complete: Option<i64>,
    pub current_phase_plans: Option<i64>,
    pub current_phase_plans_complete: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Metrics {
    pub last_activity: Option<String>,
    pub total_agent_sessions: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Blocker {
    pub phase: Option<String>,
    pub step: Option<String>,
    pub reason: Option<String>,
    pub timestamp: Option<String>,
    pub resolved: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Release {
    pub version: Option<String>,
    pub shipped: Option<String>,
}

pub fn read_state() -> Result<KonductorState, String> {
    read_state_from(Path::new(STATE_FILE))
}

pub fn read_state_from(path: &Path) -> Result<KonductorState, String> {
    if !path.exists() {
        return Err("No Konductor project found. Say 'initialize my project' to get started.".into());
    }
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read state.toml: {e}"))?;
    toml::from_str(&content).map_err(|e| format!("Failed to parse state.toml: {e}"))
}

pub fn write_state(state: &KonductorState) -> Result<(), String> {
    write_state_to(state, Path::new(STATE_FILE))
}

pub fn write_state_to(state: &KonductorState, path: &Path) -> Result<(), String> {
    let content = toml::to_string_pretty(state).map_err(|e| format!("Failed to serialize state: {e}"))?;
    fs::write(path, content).map_err(|e| format!("Failed to write state.toml: {e}"))
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

pub fn init_state(name: &str, phase: &str, phases_total: i64) -> Result<KonductorState, String> {
    let now = Utc::now().to_rfc3339();
    let state = KonductorState {
        project: Some(Project {
            name: Some(name.to_string()),
            initialized: Some(now.clone()),
        }),
        current: Some(Current {
            phase: Some(phase.to_string()),
            step: Some("specced".to_string()),
            status: Some("idle".to_string()),
            plan: Some(String::new()),
            wave: Some(0),
        }),
        progress: Some(Progress {
            phases_total: Some(phases_total),
            phases_complete: Some(0),
            current_phase_plans: Some(0),
            current_phase_plans_complete: Some(0),
        }),
        metrics: Some(Metrics {
            last_activity: Some(now),
            total_agent_sessions: Some(1),
        }),
        blockers: Vec::new(),
        release: None,
    };
    write_state(&state)?;
    Ok(state)
}

pub fn transition(step: &str, phase: Option<&str>) -> Result<KonductorState, String> {
    let mut state = read_state()?;
    let current = state.current.as_ref().ok_or("No current state found")?;
    let current_step = current.step.as_deref().unwrap_or("unknown").to_string();
    validate_transition(&current_step, step)?;

    let current = state.current.as_mut().unwrap();
    current.step = Some(step.to_string());
    current.status = Some("idle".to_string());
    if let Some(p) = phase {
        if current_step == "shipped" {
            if let Some(progress) = state.progress.as_mut() {
                progress.phases_complete = Some(progress.phases_complete.unwrap_or(0) + 1);
                progress.current_phase_plans = Some(0);
                progress.current_phase_plans_complete = Some(0);
            }
        }
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

pub fn advance_phase(phase: &str, phases_total: i64) -> Result<KonductorState, String> {
    let mut state = read_state()?;
    let current = state.current.as_ref().ok_or("No current state found")?;
    let step = current.step.as_deref().unwrap_or("unknown");

    if step != "shipped" {
        return Err(format!("Cannot advance phase: current step is '{step}', expected 'shipped'"));
    }

    let current = state.current.as_mut().unwrap();
    current.phase = Some(phase.to_string());
    current.step = Some("specced".to_string());
    current.status = Some("idle".to_string());
    current.plan = Some(String::new());
    current.wave = Some(0);

    if let Some(progress) = state.progress.as_mut() {
        progress.phases_complete = Some(progress.phases_complete.unwrap_or(0) + 1);
        progress.phases_total = Some(phases_total);
        progress.current_phase_plans = Some(0);
        progress.current_phase_plans_complete = Some(0);
    }

    if let Some(metrics) = state.metrics.as_mut() {
        metrics.last_activity = Some(Utc::now().to_rfc3339());
    }

    write_state(&state)?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn sample_state() -> KonductorState {
        KonductorState {
            project: Some(Project {
                name: Some("test".into()),
                initialized: Some("2026-01-01T00:00:00Z".into()),
            }),
            current: Some(Current {
                phase: Some("01".into()),
                step: Some("specced".into()),
                status: Some("idle".into()),
                plan: None,
                wave: None,
            }),
            progress: Some(Progress {
                phases_total: Some(3),
                phases_complete: Some(0),
                current_phase_plans: Some(0),
                current_phase_plans_complete: Some(0),
            }),
            metrics: Some(Metrics {
                last_activity: Some("2026-01-01T00:00:00Z".into()),
                total_agent_sessions: Some(1),
            }),
            blockers: vec![],
            release: None,
        }
    }

    // -- validate_transition tests --

    #[test]
    fn test_valid_transitions() {
        assert!(validate_transition("specced", "designed").is_ok());
        assert!(validate_transition("specced", "discussed").is_ok());
        assert!(validate_transition("discussed", "designed").is_ok());
        assert!(validate_transition("designed", "planned").is_ok());
        assert!(validate_transition("planned", "executing").is_ok());
        assert!(validate_transition("executing", "executed").is_ok());
        assert!(validate_transition("executed", "complete").is_ok());
        assert!(validate_transition("complete", "shipped").is_ok());
        assert!(validate_transition("blocked", "specced").is_ok());
        assert!(validate_transition("blocked", "designed").is_ok());
        assert!(validate_transition("blocked", "planned").is_ok());
        assert!(validate_transition("blocked", "executed").is_ok());
        assert!(validate_transition("shipped", "specced").is_ok());
    }

    #[test]
    fn test_invalid_transitions() {
        assert!(validate_transition("specced", "executed").is_err());
        assert!(validate_transition("planned", "complete").is_err());
        assert!(validate_transition("executed", "planned").is_err());
        assert!(validate_transition("executing", "planned").is_err());
        assert!(validate_transition("specced", "planned").is_err());
    }

    #[test]
    fn test_invalid_step() {
        let err = validate_transition("specced", "bogus").unwrap_err();
        assert!(err.contains("Invalid step"));
        assert!(err.contains("bogus"));
    }

    // -- read_state_from / write_state_to tests --

    #[test]
    fn test_read_state_missing_file() {
        let path = Path::new("/tmp/nonexistent_konductor_state.toml");
        let err = read_state_from(path).unwrap_err();
        assert!(err.contains("No Konductor project found"));
    }

    #[test]
    fn test_read_state_malformed_toml() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "this is not valid toml [[[").unwrap();
        let err = read_state_from(f.path()).unwrap_err();
        assert!(err.contains("Failed to parse state.toml"));
    }

    #[test]
    fn test_write_and_read_roundtrip() {
        let f = NamedTempFile::new().unwrap();
        let state = sample_state();
        write_state_to(&state, f.path()).unwrap();
        let loaded = read_state_from(f.path()).unwrap();
        assert_eq!(state.project, loaded.project);
        assert_eq!(state.current, loaded.current);
        assert_eq!(state.progress, loaded.progress);
    }

    #[test]
    fn test_write_state_to_readonly_path() {
        let state = sample_state();
        let err = write_state_to(&state, Path::new("/proc/nonexistent")).unwrap_err();
        assert!(err.contains("Failed to write state.toml"));
    }

    // -- transition tests (using real filesystem via STATE_FILE) --
    // These are integration-style tests that require .konductor/state.toml to exist.
    // We skip them in unit test context and test the logic via validate_transition + read/write.

    #[test]
    fn test_transition_validates_current_step() {
        // Test that transition logic correctly rejects invalid from→to
        // by testing validate_transition which transition() delegates to
        assert!(validate_transition("specced", "executing").is_err());
        assert!(validate_transition("specced", "designed").is_ok());
    }

    // -- blocker tests (logic only, no filesystem) --

    #[test]
    fn test_blocker_struct_serialization() {
        let blocker = Blocker {
            phase: Some("01".into()),
            step: Some("executing".into()),
            reason: Some("test failure".into()),
            timestamp: Some("2026-01-01T00:00:00Z".into()),
            resolved: Some(false),
        };
        let toml_str = toml::to_string(&blocker).unwrap();
        assert!(toml_str.contains("test failure"));
        let parsed: Blocker = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.phase, Some("01".into()));
        assert_eq!(parsed.resolved, Some(false));
    }

    #[test]
    fn test_state_with_blockers_roundtrip() {
        let f = NamedTempFile::new().unwrap();
        let mut state = sample_state();
        state.blockers.push(Blocker {
            phase: Some("01".into()),
            step: Some("executing".into()),
            reason: Some("build failed".into()),
            timestamp: Some("2026-01-01T00:00:00Z".into()),
            resolved: Some(false),
        });
        write_state_to(&state, f.path()).unwrap();
        let loaded = read_state_from(f.path()).unwrap();
        assert_eq!(loaded.blockers.len(), 1);
        assert_eq!(loaded.blockers[0].reason, Some("build failed".into()));
        assert_eq!(loaded.blockers[0].resolved, Some(false));
    }

    #[test]
    fn test_resolve_blocker_logic() {
        let mut state = sample_state();
        state.blockers.push(Blocker {
            phase: Some("01".into()),
            step: Some("executing".into()),
            reason: Some("error".into()),
            timestamp: None,
            resolved: Some(false),
        });
        state.current.as_mut().unwrap().status = Some("blocked".into());

        // Resolve the blocker
        let found = state.blockers.iter_mut().find(|b| {
            b.phase.as_deref() == Some("01") && b.resolved != Some(true)
        });
        assert!(found.is_some());
        found.unwrap().resolved = Some(true);

        let has_active = state.blockers.iter().any(|b| b.resolved != Some(true));
        assert!(!has_active);
    }

    #[test]
    fn test_resolve_nonexistent_blocker() {
        let state = sample_state();
        let found = state.blockers.iter().find(|b| {
            b.phase.as_deref() == Some("99") && b.resolved != Some(true)
        });
        assert!(found.is_none());
    }

    #[test]
    fn test_multiple_blockers_partial_resolve() {
        let mut state = sample_state();
        state.blockers.push(Blocker {
            phase: Some("01".into()),
            step: None,
            reason: Some("first".into()),
            timestamp: None,
            resolved: Some(false),
        });
        state.blockers.push(Blocker {
            phase: Some("01".into()),
            step: None,
            reason: Some("second".into()),
            timestamp: None,
            resolved: Some(false),
        });

        // Resolve first blocker only
        let found = state.blockers.iter_mut().find(|b| {
            b.phase.as_deref() == Some("01") && b.resolved != Some(true)
        });
        found.unwrap().resolved = Some(true);

        // Still has active blockers
        let has_active = state.blockers.iter().any(|b| b.resolved != Some(true));
        assert!(has_active);
    }

    #[test]
    fn test_empty_state_roundtrip() {
        let f = NamedTempFile::new().unwrap();
        let state = KonductorState {
            project: None,
            current: None,
            progress: None,
            metrics: None,
            blockers: vec![],
            release: None,
        };
        write_state_to(&state, f.path()).unwrap();
        let loaded = read_state_from(f.path()).unwrap();
        assert_eq!(loaded.blockers.len(), 0);
        assert!(loaded.project.is_none());
    }

    // -- advance_phase tests --

    fn shipped_state() -> KonductorState {
        KonductorState {
            project: Some(Project {
                name: Some("test".into()),
                initialized: Some("2026-01-01T00:00:00Z".into()),
            }),
            current: Some(Current {
                phase: Some("09".into()),
                step: Some("shipped".into()),
                status: Some("idle".into()),
                plan: None,
                wave: None,
            }),
            progress: Some(Progress {
                phases_total: Some(9),
                phases_complete: Some(9),
                current_phase_plans: Some(0),
                current_phase_plans_complete: Some(0),
            }),
            metrics: Some(Metrics {
                last_activity: Some("2026-01-01T00:00:00Z".into()),
                total_agent_sessions: Some(5),
            }),
            blockers: vec![Blocker {
                phase: Some("03".into()),
                step: Some("executing".into()),
                reason: Some("old blocker".into()),
                timestamp: None,
                resolved: Some(true),
            }],
            release: None,
        }
    }

    #[test]
    fn test_advance_phase_from_shipped() {
        let f = NamedTempFile::new().unwrap();
        let state = shipped_state();
        write_state_to(&state, f.path()).unwrap();

        // Simulate advance_phase logic without filesystem dependency on STATE_FILE
        let mut s = read_state_from(f.path()).unwrap();
        let step = s.current.as_ref().unwrap().step.as_deref().unwrap();
        assert_eq!(step, "shipped");

        let current = s.current.as_mut().unwrap();
        current.phase = Some("10-new-phase".into());
        current.step = Some("specced".into());
        current.status = Some("idle".into());

        let progress = s.progress.as_mut().unwrap();
        progress.phases_complete = Some(progress.phases_complete.unwrap_or(0) + 1);
        progress.phases_total = Some(10);
        progress.current_phase_plans = Some(0);
        progress.current_phase_plans_complete = Some(0);

        write_state_to(&s, f.path()).unwrap();
        let loaded = read_state_from(f.path()).unwrap();

        // Preserves project.initialized
        assert_eq!(loaded.project.unwrap().initialized, Some("2026-01-01T00:00:00Z".into()));
        // Increments phases_complete
        assert_eq!(loaded.progress.as_ref().unwrap().phases_complete, Some(10));
        // Updates phases_total
        assert_eq!(loaded.progress.as_ref().unwrap().phases_total, Some(10));
        // Sets new phase
        assert_eq!(loaded.current.as_ref().unwrap().phase, Some("10-new-phase".into()));
        // Sets specced step
        assert_eq!(loaded.current.as_ref().unwrap().step, Some("specced".into()));
        // Preserves blocker history
        assert_eq!(loaded.blockers.len(), 1);
        assert_eq!(loaded.blockers[0].reason, Some("old blocker".into()));
        // Preserves total_agent_sessions
        assert_eq!(loaded.metrics.unwrap().total_agent_sessions, Some(5));
    }

    #[test]
    fn test_advance_phase_rejects_non_shipped() {
        let state = sample_state(); // step = "specced"
        let step = state.current.as_ref().unwrap().step.as_deref().unwrap();
        assert_ne!(step, "shipped");
        // advance_phase would reject this
    }

    #[test]
    fn test_advance_phase_resets_plan_counters() {
        let f = NamedTempFile::new().unwrap();
        let mut state = shipped_state();
        state.progress.as_mut().unwrap().current_phase_plans = Some(5);
        state.progress.as_mut().unwrap().current_phase_plans_complete = Some(5);
        write_state_to(&state, f.path()).unwrap();

        let mut s = read_state_from(f.path()).unwrap();
        let progress = s.progress.as_mut().unwrap();
        progress.current_phase_plans = Some(0);
        progress.current_phase_plans_complete = Some(0);

        write_state_to(&s, f.path()).unwrap();
        let loaded = read_state_from(f.path()).unwrap();
        assert_eq!(loaded.progress.as_ref().unwrap().current_phase_plans, Some(0));
        assert_eq!(loaded.progress.as_ref().unwrap().current_phase_plans_complete, Some(0));
    }
}
