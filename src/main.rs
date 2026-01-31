use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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
    /// Manage git hooks
    Hooks {
        #[command(subcommand)]
        action: HooksAction,
    },
}

#[derive(Subcommand)]
enum HooksAction {
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
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    branches: BranchConfig,
    commits: CommitConfig,
    checks: CheckConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct BranchConfig {
    pattern: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommitConfig {
    convention: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CheckConfig {
    require_clean_worktree: bool,
    require_upstream: bool,
}

#[derive(Debug, Serialize)]
struct Report {
    branch: BranchReport,
    commits: Vec<CommitReport>,
    repo: RepoReport,
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
    worktree_clean: bool,
    upstream_set: bool,
}

#[derive(Debug, Serialize)]
struct RepoReport {
    worktree_clean: bool,
    upstream_set: bool,
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
        Commands::Hooks { action } => match action {
            HooksAction::Install { force } => hooks_install(force),
            HooksAction::Uninstall => hooks_uninstall(),
        },
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

    let has_violations = !report.summary.branch_valid
        || report.summary.invalid_commits > 0
        || !report.summary.worktree_clean
        || !report.summary.upstream_set;

    if has_violations {
        std::process::exit(1);
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

    if !report.repo.worktree_clean {
        println!("- Clean working tree: git status (commit or stash changes)");
    }

    if !report.repo.upstream_set {
        println!("- Set upstream: git push -u origin {}", report.branch.name);
    }

    for commit in report.commits.iter().filter(|c| !c.valid) {
        println!(
            "- Fix commit {}: git rebase -i --reword {}^",
            commit.hash, commit.hash
        );
    }

    if report.branch.valid
        && report.commits.iter().all(|c| c.valid)
        && report.repo.worktree_clean
        && report.repo.upstream_set
    {
        println!("- No fixes needed. You're good to go!");
    }

    Ok(())
}

fn build_report(config: &Config, commit_limit: usize) -> Result<Report> {
    let branch_name = git_current_branch()?;
    let branch_regex = Regex::new(&config.branches.pattern)
        .with_context(|| format!("invalid branch regex {}", config.branches.pattern))?;
    let branch_valid = branch_regex.is_match(&branch_name);

    let worktree_clean = !config.checks.require_clean_worktree || git_worktree_clean()?;
    let upstream_set = !config.checks.require_upstream || git_has_upstream()?;

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
    let total_commits = commit_reports.len();

    Ok(Report {
        branch: BranchReport {
            name: branch_name,
            pattern: config.branches.pattern.clone(),
            valid: branch_valid,
        },
        commits: commit_reports,
        repo: RepoReport {
            worktree_clean,
            upstream_set,
        },
        summary: Summary {
            total_commits,
            invalid_commits,
            branch_valid,
            worktree_clean,
            upstream_set,
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
        "\nRepo: worktree_clean={}, upstream_set={}",
        report.repo.worktree_clean, report.repo.upstream_set
    );

    println!(
        "Summary: branch_ok={}, invalid_commits={}",
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
        checks: CheckConfig {
            require_clean_worktree: true,
            require_upstream: true,
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

fn git_worktree_clean() -> Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("git status")?;
    if !output.status.success() {
        bail!("Failed to read git status");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

fn git_has_upstream() -> Result<bool> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
        .output()
        .context("git upstream")?;
    Ok(output.status.success())
}

const HOOK_MARKER: &str = "# git-sherpa";

fn hook_content() -> String {
    format!(
        "#!/bin/sh\n{}\nexec git-sherpa check\n",
        HOOK_MARKER
    )
}

fn git_hooks_dir() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .context("git rev-parse --git-dir")?;
    if !output.status.success() {
        bail!("Not a git repository");
    }
    let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(git_dir).join("hooks"))
}

fn hooks_install(force: bool) -> Result<()> {
    let hooks_dir = git_hooks_dir()?;
    fs::create_dir_all(&hooks_dir)?;

    let content = hook_content();
    let hook_names = ["pre-commit", "pre-push"];

    for name in &hook_names {
        let path = hooks_dir.join(name);
        if path.exists() && !force {
            eprintln!(
                "Warning: {} already exists, skipping (use --force to overwrite)",
                path.display()
            );
            continue;
        }
        fs::write(&path, &content)
            .with_context(|| format!("write hook {}", path.display()))?;
        #[cfg(unix)]
        {
            let perms = fs::Permissions::from_mode(0o755);
            fs::set_permissions(&path, perms)
                .with_context(|| format!("chmod {}", path.display()))?;
        }
        println!("Installed {}", path.display());
    }

    Ok(())
}

fn hooks_uninstall() -> Result<()> {
    let hooks_dir = git_hooks_dir()?;
    let hook_names = ["pre-commit", "pre-push"];

    for name in &hook_names {
        let path = hooks_dir.join(name);
        if !path.exists() {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap_or_default();
        if !content.contains(HOOK_MARKER) {
            eprintln!(
                "Warning: {} was not installed by git-sherpa, skipping",
                path.display()
            );
            continue;
        }
        fs::remove_file(&path)
            .with_context(|| format!("remove hook {}", path.display()))?;
        println!("Removed {}", path.display());
    }

    Ok(())
}
