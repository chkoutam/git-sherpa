use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub branches: BranchConfig,
    pub commits: CommitConfig,
    pub checks: CheckConfig,
    #[serde(default)]
    pub sensitive: SensitiveConfig,
    #[serde(default)]
    pub hooks: HooksConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BranchConfig {
    pub pattern: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitConfig {
    pub convention: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckConfig {
    pub require_clean_worktree: bool,
    pub require_upstream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SensitiveConfig {
    pub patterns: Vec<String>,
}

impl Default for SensitiveConfig {
    fn default() -> Self {
        Self {
            patterns: crate::sensitive::default_patterns(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HooksConfig {
    pub protected_branches: Vec<String>,
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            protected_branches: vec!["main".to_string(), "master".to_string()],
        }
    }
}

pub fn load_config(path: &Path) -> Result<Config> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("read config at {}", path.display()))?;
    let config: Config = toml::from_str(&contents).context("parse config")?;
    Ok(config)
}

pub fn default_config() -> Config {
    Config {
        branches: BranchConfig {
            pattern: "^(feat|fix|chore|docs|refactor)/[a-z0-9-]+$".to_string(),
        },
        commits: CommitConfig {
            convention: "conventional".to_string(),
        },
        checks: CheckConfig {
            require_clean_worktree: true,
            require_upstream: true,
        },
        sensitive: SensitiveConfig::default(),
        hooks: HooksConfig::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_config();
        assert_eq!(cfg.commits.convention, "conventional");
        assert!(cfg.checks.require_clean_worktree);
        assert!(cfg.checks.require_upstream);
        assert!(!cfg.branches.pattern.is_empty());
        assert!(!cfg.sensitive.patterns.is_empty());
        assert!(cfg.hooks.protected_branches.contains(&"main".to_string()));
    }

    #[test]
    fn valid_toml_parses() {
        let toml_str = r#"
[branches]
pattern = "^main$"

[commits]
convention = "conventional"

[checks]
require_clean_worktree = false
require_upstream = false
"#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.branches.pattern, "^main$");
        assert_eq!(cfg.commits.convention, "conventional");
        assert!(!cfg.checks.require_clean_worktree);
        assert!(!cfg.checks.require_upstream);
        // defaults kick in
        assert!(!cfg.sensitive.patterns.is_empty());
        assert!(!cfg.hooks.protected_branches.is_empty());
    }

    #[test]
    fn custom_sensitive_patterns() {
        let toml_str = r#"
[branches]
pattern = "^main$"

[commits]
convention = "conventional"

[checks]
require_clean_worktree = false
require_upstream = false

[sensitive]
patterns = ["*.secret"]
"#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.sensitive.patterns, vec!["*.secret"]);
    }

    #[test]
    fn invalid_toml_returns_error() {
        let bad = "not valid toml [[[";
        assert!(toml::from_str::<Config>(bad).is_err());
    }
}
