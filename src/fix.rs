use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use crate::check::build_report;
use crate::config::load_config;
use crate::git;

pub fn fix(config_path: &Path, commit_limit: usize, apply: bool) -> Result<()> {
    let config = load_config(config_path)?;
    let report = build_report(&config, commit_limit)?;

    println!("{}", "Suggested fixes:".yellow().bold());

    let mut has_fixes = false;

    if !report.branch.valid {
        has_fixes = true;
        println!(
            "\n{}",
            "Branch name does not match pattern:".yellow().bold()
        );
        println!(
            "  {}",
            format!(
                "git branch -m {} <new-name-matching:{}>",
                report.branch.name, report.branch.pattern
            )
            .cyan()
        );
    }

    if !report.repo.worktree_clean {
        has_fixes = true;
        println!("\n{}", "Working tree is dirty:".yellow().bold());
        println!(
            "  {}",
            "git stash  or  git add . && git commit".cyan()
        );
    }

    if !report.repo.upstream_set {
        has_fixes = true;
        if apply {
            println!("\n{}", "Setting upstream...".yellow().bold());
            git::push_set_upstream(&report.branch.name)?;
            println!("  {}", "Upstream set successfully.".green());
        } else {
            println!("\n{}", "No upstream tracking branch:".yellow().bold());
            println!(
                "  {}",
                format!("git push -u origin {}", report.branch.name).cyan()
            );
            println!(
                "  {}",
                "(use --apply to execute this automatically)".dimmed()
            );
        }
    }

    for commit in report.commits.iter().filter(|c| !c.valid) {
        has_fixes = true;
        println!(
            "\n{}",
            format!("Invalid commit {}:", &commit.hash[..8])
                .yellow()
                .bold()
        );
        println!(
            "  {}",
            format!("git rebase -i --reword {}^", commit.hash).cyan()
        );
    }

    if !report.sensitive.files.is_empty() {
        has_fixes = true;
        println!("\n{}", "Sensitive files staged:".red().bold());
        for f in &report.sensitive.files {
            println!("  {}", format!("git reset HEAD {}", f).cyan());
        }
    }

    if !has_fixes {
        println!(
            "\n{}",
            "No fixes needed. You're good to go!".green().bold()
        );
    }

    Ok(())
}
