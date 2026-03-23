use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const CONFIG_FILE: &str = ".konductor/config.toml";

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct Config {
    #[serde(default = "GeneralConfig::default")]
    pub general: GeneralConfig,
    #[serde(default = "ExecutionConfig::default")]
    pub execution: ExecutionConfig,
    #[serde(default = "GitConfig::default")]
    pub git: GitConfig,
    #[serde(default = "FeaturesConfig::default")]
    pub features: FeaturesConfig,
    #[serde(default = "HooksConfig::default")]
    pub hooks: HooksConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GeneralConfig {
    #[serde(default = "default_model")]
    pub default_model: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self { default_model: default_model() }
    }
}

fn default_model() -> String { "claude-sonnet-4".into() }

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ExecutionConfig {
    #[serde(default = "default_parallelism")]
    pub max_wave_parallelism: i64,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self { max_wave_parallelism: default_parallelism() }
    }
}

fn default_parallelism() -> i64 { 4 }

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GitConfig {
    #[serde(default = "default_true")]
    pub auto_commit: bool,
    #[serde(default = "default_branching")]
    pub branching_strategy: String,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            auto_commit: true,
            branching_strategy: default_branching(),
        }
    }
}

fn default_true() -> bool { true }
fn default_branching() -> String { "none".into() }

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FeaturesConfig {
    #[serde(default = "default_true")]
    pub research: bool,
    #[serde(default = "default_true")]
    pub plan_checker: bool,
    #[serde(default = "default_true")]
    pub verifier: bool,
    #[serde(default = "default_true")]
    pub design_review: bool,
    #[serde(default = "default_true")]
    pub code_review: bool,
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            research: true,
            plan_checker: true,
            verifier: true,
            design_review: true,
            code_review: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct HooksConfig {
    #[serde(default = "default_test_patterns")]
    pub test_patterns: Vec<String>,
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self { test_patterns: default_test_patterns() }
    }
}

fn default_test_patterns() -> Vec<String> {
    [
        "test", "pytest", "cargo test", "npm test", "npx test",
        "go test", "mvn test", "gradle test", "gradlew test",
        "jest", "vitest", "mocha", "junit",
        "ruff", "lint", "eslint", "golangci-lint", "checkstyle", "clippy",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

pub fn read_config() -> Result<Config, String> {
    read_config_from(Path::new(CONFIG_FILE))
}

pub fn read_config_from(path: &Path) -> Result<Config, String> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config.toml: {e}"))?;
    toml::from_str(&content)
        .map_err(|e| format!("Failed to parse config.toml: {e}"))
}

pub fn write_config(config: &Config) -> Result<(), String> {
    write_config_to(config, Path::new(CONFIG_FILE))
}

pub fn write_config_to(config: &Config, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {e}"))?;
    }
    let content = toml::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    fs::write(path, content)
        .map_err(|e| format!("Failed to write config.toml: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_defaults_when_missing() {
        let path = Path::new("/tmp/nonexistent_konductor_config.toml");
        let config = read_config_from(path).unwrap();
        assert_eq!(config.general.default_model, "claude-sonnet-4");
        assert_eq!(config.execution.max_wave_parallelism, 4);
        assert!(config.git.auto_commit);
        assert_eq!(config.git.branching_strategy, "none");
        assert!(config.features.research);
        assert!(config.features.plan_checker);
        assert!(config.features.verifier);
        assert!(config.features.design_review);
        assert!(config.features.code_review);
    }

    #[test]
    fn test_parse_valid_config() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, r#"
[general]
default_model = "gpt-4"

[execution]
max_wave_parallelism = 2

[git]
auto_commit = false
branching_strategy = "feature"

[features]
research = false
plan_checker = true
verifier = false
design_review = false
code_review = false
"#).unwrap();
        let config = read_config_from(f.path()).unwrap();
        assert_eq!(config.general.default_model, "gpt-4");
        assert_eq!(config.execution.max_wave_parallelism, 2);
        assert!(!config.git.auto_commit);
        assert_eq!(config.git.branching_strategy, "feature");
        assert!(!config.features.research);
        assert!(config.features.plan_checker);
        assert!(!config.features.verifier);
        assert!(!config.features.design_review);
        assert!(!config.features.code_review);
    }

    #[test]
    fn test_partial_config_uses_defaults() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "[general]\ndefault_model = \"custom\"\n").unwrap();
        let config = read_config_from(f.path()).unwrap();
        assert_eq!(config.general.default_model, "custom");
        // Missing sections use defaults
        assert_eq!(config.execution.max_wave_parallelism, 4);
        assert!(config.git.auto_commit);
        assert!(config.features.research);
    }

    #[test]
    fn test_empty_file_uses_defaults() {
        let f = NamedTempFile::new().unwrap();
        let config = read_config_from(f.path()).unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_malformed_config() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "not valid toml [[[").unwrap();
        let err = read_config_from(f.path()).unwrap_err();
        assert!(err.contains("Failed to parse config.toml"));
    }
}
