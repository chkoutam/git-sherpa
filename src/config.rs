use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub branches: BranchConfig,
    pub commits: CommitConfig,
    pub checks: CheckConfig,
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
    }

    #[test]
    fn invalid_toml_returns_error() {
        let bad = "not valid toml [[[";
        assert!(toml::from_str::<Config>(bad).is_err());
    }
}
