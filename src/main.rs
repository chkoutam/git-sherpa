mod check;
mod cli;
mod config;
mod fix;
mod git;
mod hooks;
mod sensitive;

use anyhow::{bail, Context, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;

use cli::{Cli, Commands, HooksAction};
use config::default_config;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { config } => init(&config),
        Commands::Check {
            config,
            format,
            commit_limit,
        } => check::check(&config, format, commit_limit),
        Commands::Fix {
            config,
            commit_limit,
            apply,
        } => fix::fix(&config, commit_limit, apply),
        Commands::Hooks { action } => match action {
            HooksAction::Install { force } => {
                let config_path = std::path::Path::new(cli::DEFAULT_CONFIG_PATH);
                let cfg = if config_path.exists() {
                    config::load_config(config_path).unwrap_or_else(|_| default_config())
                } else {
                    default_config()
                };
                hooks::install_with_config(force, &cfg.hooks.protected_branches)
            }
            HooksAction::Uninstall => hooks::uninstall(),
        },
    }
}

fn init(config_path: &std::path::Path) -> Result<()> {
    if config_path.exists() {
        bail!("Config already exists at {}", config_path.display());
    }

    let config = default_config();
    let toml = toml::to_string_pretty(&config).context("serialize config")?;
    fs::write(config_path, toml).with_context(|| format!("write {}", config_path.display()))?;

    let scripts_dir = PathBuf::from(".gitsherpa");
    fs::create_dir_all(&scripts_dir)?;
    fs::write(
        scripts_dir.join("README.md"),
        "# git-sherpa scripts\n\nAdd custom hooks or scripts for your repo here.\n",
    )?;

    println!(
        "Initialized git-sherpa config at {}",
        config_path.display()
    );
    Ok(())
}
