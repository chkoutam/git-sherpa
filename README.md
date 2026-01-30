# git-sherpa

A Rust CLI to standardize Git hygiene across GitHub and GitLab repositories.

## Commands

### init
Creates a default `.gitsherpa.toml` and a `.gitsherpa/` scripts directory.

```bash
git-sherpa init
```

### check
Analyzes the current branch, recent commits, and repo hygiene.

```bash
git-sherpa check
# JSON output
git-sherpa check --format json
```

### fix
Prints suggested fixes for invalid branches or commits.

```bash
git-sherpa fix
```

## Config

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
