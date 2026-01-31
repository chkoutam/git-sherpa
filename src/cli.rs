use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub const DEFAULT_CONFIG_PATH: &str = ".gitsherpa.toml";

#[derive(Parser)]
#[command(name = "git-sherpa", version, about = "Git hygiene assistant")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize repository configuration and scripts
    Init {
        #[arg(long, default_value = DEFAULT_CONFIG_PATH)]
        config: PathBuf,
    },
    /// Analyze repo branches and commits
    Check {
        #[arg(long, default_value = DEFAULT_CONFIG_PATH)]
        config: PathBuf,
        #[arg(long, default_value = "text")]
        format: OutputFormat,
        #[arg(long, default_value_t = 20)]
        commit_limit: usize,
    },
    /// Propose fixes for issues
    Fix {
        #[arg(long, default_value = DEFAULT_CONFIG_PATH)]
        config: PathBuf,
        #[arg(long, default_value_t = 20)]
        commit_limit: usize,
        /// Automatically apply safe fixes (e.g. set upstream)
        #[arg(long)]
        apply: bool,
    },
    /// Manage git hooks
    Hooks {
        #[command(subcommand)]
        action: HooksAction,
    },
}

#[derive(Subcommand)]
pub enum HooksAction {
    /// Install pre-commit and pre-push hooks
    Install {
        /// Overwrite existing hooks
        #[arg(long)]
        force: bool,
    },
    /// Remove hooks installed by git-sherpa
    Uninstall,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}
