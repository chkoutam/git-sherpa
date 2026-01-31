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
