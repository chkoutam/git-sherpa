use anyhow::{Context, Result};
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::git;

const HOOK_MARKER: &str = "# git-sherpa";

pub(crate) fn hook_content() -> String {
    format!("#!/bin/sh\n{}\nexec git-sherpa check\n", HOOK_MARKER)
}

pub(crate) fn pre_push_hook_content(protected_branches: &[String]) -> String {
    let branches_list = protected_branches.join("|");
    format!(
        r#"#!/bin/sh
{marker}

# Block force push
for arg in "$@"; do
    case "$arg" in
        --force|-f|--force-with-lease)
            echo "git-sherpa: force push is blocked."
            exit 1
            ;;
    esac
done

# Block push to protected branches
current_branch=$(git rev-parse --abbrev-ref HEAD)
case "$current_branch" in
    {branches})
        echo "git-sherpa: direct push to '$current_branch' is blocked. Use a pull request."
        exit 1
        ;;
esac

exec git-sherpa check
"#,
        marker = HOOK_MARKER,
        branches = branches_list,
    )
}

pub fn install_with_config(force: bool, protected_branches: &[String]) -> Result<()> {
    let hooks_dir = git::hooks_dir()?;
    fs::create_dir_all(&hooks_dir)?;

    let pre_commit_content = hook_content();
    let pre_push_content = pre_push_hook_content(protected_branches);

    let hooks: [(&str, &str); 2] = [
        ("pre-commit", &pre_commit_content),
        ("pre-push", &pre_push_content),
    ];

    for (name, content) in &hooks {
        let path = hooks_dir.join(name);
        if path.exists() && !force {
            eprintln!(
                "Warning: {} already exists, skipping (use --force to overwrite)",
                path.display()
            );
            continue;
        }
        fs::write(&path, content)
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

pub fn uninstall() -> Result<()> {
    let hooks_dir = git::hooks_dir()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_content_has_shebang() {
        assert!(hook_content().starts_with("#!/bin/sh\n"));
    }

    #[test]
    fn hook_content_has_marker() {
        assert!(hook_content().contains("# git-sherpa"));
    }

    #[test]
    fn hook_content_has_exec() {
        assert!(hook_content().contains("exec git-sherpa check"));
    }

    #[test]
    fn pre_push_blocks_protected_branches() {
        let content = pre_push_hook_content(&["main".into(), "master".into()]);
        assert!(content.contains("main|master"));
        assert!(content.contains("force push is blocked"));
        assert!(content.contains("direct push to"));
    }

    #[test]
    fn pre_push_has_marker() {
        let content = pre_push_hook_content(&["main".into()]);
        assert!(content.contains(HOOK_MARKER));
    }
}
