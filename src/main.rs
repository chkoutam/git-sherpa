use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_CONFIG_PATH: &str = ".gitsherpa.toml";

#[derive(Parser)]
#[command(name = "git-sherpa", version, about = "Git hygiene assistant")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
    },
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    branches: BranchConfig,
    commits: CommitConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct BranchConfig {
    pattern: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommitConfig {
    convention: String,
}

#[derive(Debug, Serialize)]
struct Report {
    branch: BranchReport,
    commits: Vec<CommitReport>,
    summary: Summary,
}

#[derive(Debug, Serialize)]
struct BranchReport {
    name: String,
    pattern: String,
    valid: bool,
}

#[derive(Debug, Serialize)]
struct CommitReport {
    hash: String,
    message: String,
    valid: bool,
}

#[derive(Debug, Serialize)]
struct Summary {
    total_commits: usize,
    invalid_commits: usize,
    branch_valid: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { config } => init(&config),
        Commands::Check {
            config,
            format,
            commit_limit,
        } => check(&config, format, commit_limit),
        Commands::Fix {
            config,
            commit_limit,
        } => fix(&config, commit_limit),
    }
}

fn init(config_path: &Path) -> Result<()> {
    if config_path.exists() {
        bail!("Config already exists at {}", config_path.display());
    }

    let config = default_config();
    let toml = toml::to_string_pretty(&config).context("serialize config")?;
    fs::write(config_path, toml).with_context(|| format!("write {}", config_path.display()))?;

    let scripts_dir = PathBuf::from(".gitsherpa");
    fs::create_dir_all(&scripts_dir)?;
    fs::write(scripts_dir.join("README.md"), scripts_readme())?;

    println!("Initialized git-sherpa config at {}", config_path.display());
    Ok(())
}

fn check(config_path: &Path, format: OutputFormat, commit_limit: usize) -> Result<()> {
    let config = load_config(config_path)?;
    let report = build_report(&config, commit_limit)?;

    match format {
        OutputFormat::Text => print_text_report(&report),
        OutputFormat::Json => print_json_report(&report)?,
    }

    Ok(())
}

fn fix(config_path: &Path, commit_limit: usize) -> Result<()> {
    let config = load_config(config_path)?;
    let report = build_report(&config, commit_limit)?;

    println!("Suggested fixes:");

    if !report.branch.valid {
        println!(
            "- Rename branch: git branch -m {} <new-name-matching:{}>",
            report.branch.name, report.branch.pattern
        );
    }

    for commit in report.commits.iter().filter(|c| !c.valid) {
        println!(
            "- Fix commit {}: git rebase -i --reword {}^",
            commit.hash, commit.hash
        );
    }

    if report.branch.valid && report.commits.iter().all(|c| c.valid) {
        println!("- No fixes needed. You're good to go!");
    }

    Ok(())
}

fn build_report(config: &Config, commit_limit: usize) -> Result<Report> {
    let branch_name = git_current_branch()?;
    let branch_regex = Regex::new(&config.branches.pattern)
        .with_context(|| format!("invalid branch regex {}", config.branches.pattern))?;
    let branch_valid = branch_regex.is_match(&branch_name);

    let commit_regex = commit_regex_for(&config.commits.convention)?;
    let commits = git_recent_commits(commit_limit)?;
    let commit_reports: Vec<CommitReport> = commits
        .into_iter()
        .map(|(hash, message)| CommitReport {
            valid: commit_regex.is_match(&message),
            hash,
            message,
        })
        .collect();

    let invalid_commits = commit_reports.iter().filter(|c| !c.valid).count();

    Ok(Report {
        branch: BranchReport {
            name: branch_name,
            pattern: config.branches.pattern.clone(),
            valid: branch_valid,
        },
        commits: commit_reports,
        summary: Summary {
            total_commits: commit_reports.len(),
            invalid_commits,
            branch_valid,
        },
    })
}

fn print_text_report(report: &Report) {
    println!("Branch: {}", report.branch.name);
    println!("Pattern: {}", report.branch.pattern);
    println!("Branch OK: {}", report.branch.valid);

    println!("\nCommits:");
    for commit in &report.commits {
        println!(
            "- {} {} [{}]",
            commit.hash,
            commit.message,
            if commit.valid { "OK" } else { "INVALID" }
        );
    }

    println!(
        "\nSummary: branch_ok={}, invalid_commits={}",
        report.summary.branch_valid, report.summary.invalid_commits
    );
}

fn print_json_report(report: &Report) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    println!("{}", json);
    Ok(())
}

fn load_config(path: &Path) -> Result<Config> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("read config at {}", path.display()))?;
    let config: Config = toml::from_str(&contents).context("parse config")?;
    Ok(config)
}

fn default_config() -> Config {
    Config {
        branches: BranchConfig {
            pattern: "^(feat|fix|chore|docs|refactor)/[a-z0-9-]+$".to_string(),
        },
        commits: CommitConfig {
            convention: "conventional".to_string(),
        },
    }
}

fn scripts_readme() -> String {
    "# git-sherpa scripts\n\nAdd custom hooks or scripts for your repo here.\n".to_string()
}

fn commit_regex_for(convention: &str) -> Result<Regex> {
    match convention {
        "conventional" => Regex::new(
            r"^(feat|fix|chore|docs|refactor|test|perf|ci|build)(\([a-z0-9-]+\))?: .+",
        )
        .context("invalid conventional commit regex"),
        _ => bail!("Unsupported commit convention: {}", convention),
    }
}

fn git_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("git rev-parse")?;
    if !output.status.success() {
        bail!("Not a git repository or failed to get branch name");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_recent_commits(limit: usize) -> Result<Vec<(String, String)>> {
    let output = Command::new("git")
        .args([
            "log",
            &format!("-n{}", limit),
            "--pretty=format:%H:::%s",
        ])
        .output()
        .context("git log")?;

    if !output.status.success() {
        bail!("Failed to read git log");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits = stdout
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, ":::");
            let hash = parts.next()?.to_string();
            let message = parts.next()?.to_string();
            Some((hash, message))
        })
        .collect();

    Ok(commits)
}
