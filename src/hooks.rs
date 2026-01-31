use anyhow::{Context, Result};
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::git;

const HOOK_MARKER: &str = "# git-sherpa";

fn hook_content() -> String {
    format!("#!/bin/sh\n{}\nexec git-sherpa check\n", HOOK_MARKER)
}

pub fn install(force: bool) -> Result<()> {
    let hooks_dir = git::hooks_dir()?;
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
