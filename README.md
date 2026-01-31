<p align="center">
  <img src=".github/logo.svg" alt="git-sherpa" width="480"/>
</p>

<p align="center">
  <a href="https://github.com/chkoutam/git-sherpa/actions/workflows/ci.yml"><img src="https://github.com/chkoutam/git-sherpa/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <img src="https://img.shields.io/badge/language-Rust-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License">
</p>

A Rust CLI to standardize Git hygiene across GitHub and GitLab repositories.

## Features

- **Branch naming enforcement** — validate branches against configurable patterns (e.g. `feat/`, `fix/`, `chore/`)
- **Commit convention checks** — ensure commits follow Conventional Commits
- **Worktree & upstream checks** — detect uncommitted changes and missing upstream branches
- **Auto-fix suggestions** — get actionable commands to rename branches or reword commits
- **Git hooks management** — install pre-commit / pre-push hooks automatically
- **Fully configurable** — single `.gitsherpa.toml` at the repo root

## Installation

```bash
cargo install --path .
```

## Quick start

```bash
# Initialize config and hooks directory
git-sherpa init

# Run all checks on the current branch
git-sherpa check

# Get fix suggestions
git-sherpa fix

# Install git hooks
git-sherpa hooks
```

## Commands

| Command | Description |
|---------|-------------|
| `init`  | Create `.gitsherpa.toml` and `.gitsherpa/` scripts directory |
| `check` | Analyze branch name, recent commits, and repo hygiene |
| `fix`   | Print suggested fixes for invalid branches or commits |
| `hooks` | Manage git hooks (install / uninstall) |

### Output formats

```bash
# Default human-readable output
git-sherpa check

# JSON output for CI integration
git-sherpa check --format json
```

## Configuration

Create a `.gitsherpa.toml` at the root of your repository:

```toml
[branches]
pattern = "^(feat|fix|chore|docs|refactor)/[a-z0-9-]+$"

[commits]
convention = "conventional"

[checks]
require_clean_worktree = true
require_upstream = true
```

## Roadmap

### Done

- [x] Colored terminal output (check & fix)
- [x] `fix --apply` auto-execution for safe fixes (set upstream)
- [x] Sensitive file detection (`.env`, `*.pem`, `*.key`, etc.)
- [x] Enhanced pre-push hook (block force push + protected branches)
- [x] Configurable `[sensitive]` and `[hooks]` sections

### To do

- [ ] **Providers** — GitHub + GitLab API adapters (PR status, merge checks)
- [ ] **PR stacking mode** — manage dependent PRs as a stack
- [ ] **CI integration** — GitHub Actions reusable workflow / GitLab CI template
- [ ] **Interactive fix mode** — prompt-based selection of fixes to apply (`fix -i`)
- [ ] **Config inheritance** — global `~/.gitsherpa.toml` merged with per-repo config
- [ ] **Custom commit conventions** — user-defined regex in config instead of `"conventional"` only
- [ ] **Monorepo support** — per-directory rules and scoped checks
- [ ] **Hook customization** — allow user scripts to run alongside git-sherpa hooks
- [ ] **Publish to crates.io** — `cargo install git-sherpa`
