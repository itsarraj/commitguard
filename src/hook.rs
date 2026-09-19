use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// `commit-msg` (not `pre-commit`) is the right hook for this: git passes
/// it the path to a temp file holding the message being committed, and a
/// non-zero exit aborts the commit with the hook's own output shown to
/// the user.
const HOOK_SCRIPT: &str =
    "#!/bin/sh\n# Installed by `commitguard install-hook`.\nexec commitguard lint \"$1\"\n";

pub fn install_hook(repo_dir: &Path) -> Result<PathBuf> {
    let hooks_dir = repo_dir.join(".git").join("hooks");
    fs::create_dir_all(&hooks_dir).with_context(|| format!("creating {}", hooks_dir.display()))?;
    let hook_path = hooks_dir.join("commit-msg");
    fs::write(&hook_path, HOOK_SCRIPT)
        .with_context(|| format!("writing {}", hook_path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755))?;
    }

    Ok(hook_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_an_executable_commit_msg_hook() {
        let repo =
            std::env::temp_dir().join(format!("commitguard-test-hook-{}", std::process::id()));
        fs::create_dir_all(&repo).unwrap();

        let hook_path = install_hook(&repo).unwrap();
        assert_eq!(hook_path.file_name().unwrap(), "commit-msg");
        let contents = fs::read_to_string(&hook_path).unwrap();
        assert!(contents.contains("commitguard lint"));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&hook_path).unwrap().permissions().mode();
            assert!(mode & 0o111 != 0);
        }

        fs::remove_dir_all(&repo).ok();
    }
}
