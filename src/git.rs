use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

pub fn current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("git rev-parse")?;
    if !output.status.success() {
        bail!("Not a git repository or failed to get branch name");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn recent_commits(limit: usize) -> Result<Vec<(String, String)>> {
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

pub fn worktree_clean() -> Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("git status")?;
    if !output.status.success() {
        bail!("Failed to read git status");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

pub fn has_upstream() -> Result<bool> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
        .output()
        .context("git upstream")?;
    Ok(output.status.success())
}

pub fn hooks_dir() -> Result<PathBuf> {
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

pub fn staged_files() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
        .context("git diff --cached")?;
    if !output.status.success() {
        bail!("Failed to list staged files");
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().map(|l| l.to_string()).collect())
}

pub fn push_set_upstream(branch: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["push", "-u", "origin", branch])
        .status()
        .context("git push -u origin")?;
    if !status.success() {
        bail!("Failed to push and set upstream for branch '{}'", branch);
    }
    Ok(())
}
