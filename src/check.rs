use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::Serialize;
use std::path::Path;

use crate::cli::OutputFormat;
use crate::config::{load_config, Config};
use crate::git;

#[derive(Debug, Serialize)]
pub struct Report {
    pub branch: BranchReport,
    pub commits: Vec<CommitReport>,
    pub repo: RepoReport,
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
}

#[derive(Debug, Serialize)]
pub struct RepoReport {
    pub worktree_clean: bool,
    pub upstream_set: bool,
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
        || !report.summary.upstream_set;

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

fn commit_regex_for(convention: &str) -> Result<Regex> {
    match convention {
        "conventional" => Regex::new(
            r"^(feat|fix|chore|docs|refactor|test|perf|ci|build)(\([a-z0-9-]+\))?: .+",
        )
        .context("invalid conventional commit regex"),
        _ => bail!("Unsupported commit convention: {}", convention),
    }
}
