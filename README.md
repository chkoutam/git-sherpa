# git-sherpa

[![CI](https://github.com/chkoutam/git-sherpa/actions/workflows/ci.yml/badge.svg)](https://github.com/chkoutam/git-sherpa/actions/workflows/ci.yml)

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

- Providers: GitHub + GitLab adapters
- PR stacking mode
- CI integration
