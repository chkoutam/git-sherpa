use anyhow::{bail, Context, Result};
use colored::Colorize;
use regex::Regex;
use serde::Serialize;
use std::path::Path;

use crate::cli::OutputFormat;
use crate::config::{load_config, Config};
use crate::git;
use crate::sensitive;

#[derive(Debug, Serialize)]
pub struct Report {
    pub branch: BranchReport,
    pub commits: Vec<CommitReport>,
    pub repo: RepoReport,
    pub sensitive: SensitiveReport,
    pub summary: Summary,
}

#[derive(Debug, Serialize)]
pub struct BranchReport {
    pub name: String,
    pub pattern: String,
    pub valid: bool,
}

#[derive(Debug, Serialize)]
pub struct CommitReport {
    pub hash: String,
    pub message: String,
    pub valid: bool,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub total_commits: usize,
    pub invalid_commits: usize,
    pub branch_valid: bool,
    pub worktree_clean: bool,
    pub upstream_set: bool,
    pub sensitive_files: usize,
}

#[derive(Debug, Serialize)]
pub struct RepoReport {
    pub worktree_clean: bool,
    pub upstream_set: bool,
}

#[derive(Debug, Serialize)]
pub struct SensitiveReport {
    pub files: Vec<String>,
}

pub fn check(config_path: &Path, format: OutputFormat, commit_limit: usize) -> Result<()> {
    let config = load_config(config_path)?;
    let report = build_report(&config, commit_limit)?;

    match format {
        OutputFormat::Text => print_text_report(&report),
        OutputFormat::Json => print_json_report(&report)?,
    }

    let has_violations = !report.summary.branch_valid
        || report.summary.invalid_commits > 0
        || !report.summary.worktree_clean
        || !report.summary.upstream_set
        || report.summary.sensitive_files > 0;

    if has_violations {
        std::process::exit(1);
    }

    Ok(())
}

pub fn build_report(config: &Config, commit_limit: usize) -> Result<Report> {
    let branch_name = git::current_branch()?;
    let branch_regex = Regex::new(&config.branches.pattern)
        .with_context(|| format!("invalid branch regex {}", config.branches.pattern))?;
    let branch_valid = branch_regex.is_match(&branch_name);

    let worktree_clean = !config.checks.require_clean_worktree || git::worktree_clean()?;
    let upstream_set = !config.checks.require_upstream || git::has_upstream()?;

    let commit_regex = commit_regex_for(&config.commits.convention)?;
    let commits = git::recent_commits(commit_limit)?;
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

    let staged = git::staged_files().unwrap_or_default();
    let sensitive_files = sensitive::check_sensitive_files(&staged, &config.sensitive.patterns);

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
        sensitive: SensitiveReport {
            files: sensitive_files.clone(),
        },
        summary: Summary {
            total_commits,
            invalid_commits,
            branch_valid,
            worktree_clean,
            upstream_set,
            sensitive_files: sensitive_files.len(),
        },
    })
}

fn print_text_report(report: &Report) {
    let status = |ok: bool| -> String {
        if ok {
            "OK".green().to_string()
        } else {
            "INVALID".red().to_string()
        }
    };

    println!("Branch: {}", report.branch.name);
    println!("Pattern: {}", report.branch.pattern);
    println!("Branch: {}", status(report.branch.valid));

    println!("\nCommits:");
    for commit in &report.commits {
        let tag = if commit.valid {
            "OK".green().to_string()
        } else {
            "INVALID".red().to_string()
        };
        println!("- {} {} [{}]", &commit.hash[..8], commit.message, tag);
    }

    println!(
        "\nRepo: worktree_clean={}, upstream_set={}",
        status(report.repo.worktree_clean),
        status(report.repo.upstream_set)
    );

    if !report.sensitive.files.is_empty() {
        println!("\n{}", "Sensitive files staged:".red().bold());
        for f in &report.sensitive.files {
            println!("  - {}", f.red());
        }
    }

    let all_ok = report.summary.branch_valid
        && report.summary.invalid_commits == 0
        && report.summary.worktree_clean
        && report.summary.upstream_set
        && report.summary.sensitive_files == 0;

    let summary_label = if all_ok {
        "Summary: ALL OK".green().bold().to_string()
    } else {
        format!(
            "Summary: branch_ok={}, invalid_commits={}, sensitive_files={}",
            status(report.summary.branch_valid),
            report.summary.invalid_commits,
            report.summary.sensitive_files
        )
    };
    println!("\n{}", summary_label);
}

fn print_json_report(report: &Report) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    println!("{}", json);
    Ok(())
}

pub(crate) fn commit_regex_for(convention: &str) -> Result<Regex> {
    match convention {
        "conventional" => Regex::new(
            r"^(feat|fix|chore|docs|refactor|test|perf|ci|build)(\([a-z0-9-]+\))?: .+",
        )
        .context("invalid conventional commit regex"),
        _ => bail!("Unsupported commit convention: {}", convention),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_conventional_commits() {
        let re = commit_regex_for("conventional").unwrap();
        assert!(re.is_match("feat: add login"));
        assert!(re.is_match("fix(auth): resolve token issue"));
        assert!(re.is_match("chore: cleanup"));
        assert!(re.is_match("docs: update readme"));
        assert!(re.is_match("refactor(core): simplify logic"));
    }

    #[test]
    fn invalid_conventional_commits() {
        let re = commit_regex_for("conventional").unwrap();
        assert!(!re.is_match("added login"));
        assert!(!re.is_match("Fix bug"));
        assert!(!re.is_match("random message"));
        assert!(!re.is_match(""));
    }

    #[test]
    fn unknown_convention_returns_error() {
        assert!(commit_regex_for("unknown").is_err());
    }
}
