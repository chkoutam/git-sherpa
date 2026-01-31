use anyhow::Result;
use std::path::Path;

use crate::check::build_report;
use crate::config::load_config;

pub fn fix(config_path: &Path, commit_limit: usize) -> Result<()> {
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
