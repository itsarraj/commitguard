use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

/// Full commit messages (subject + body) for `range` (any `git log`
/// revision range, e.g. `"v1.0.0..HEAD"`; `None` means the whole history),
/// oldest-first (`--reverse`, so a changelog reads in the order things
/// actually happened). Uses `\x1e` (ASCII record separator) between
/// messages rather than a blank line — a commit body can itself contain
/// blank lines, which would otherwise corrupt the split.
pub fn commit_messages(repo_dir: &Path, range: Option<&str>) -> Result<Vec<String>> {
    let mut args = vec![
        "log".to_string(),
        "--reverse".to_string(),
        "--pretty=format:%B\x1e".to_string(),
    ];
    if let Some(r) = range {
        args.push(r.to_string());
    }

    let output = Command::new("git")
        .args(&args)
        .current_dir(repo_dir)
        .output()
        .context("running git log")?;
    if !output.status.success() {
        anyhow::bail!(
            "git log failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text
        .split('\u{1e}')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_repo(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "commitguard-test-git-{}-{}",
            std::process::id(),
            name
        ));
        fs::create_dir_all(&dir).unwrap();
        let run = |args: &[&str]| {
            let status = Command::new("git")
                .args(args)
                .current_dir(&dir)
                .status()
                .unwrap();
            assert!(status.success(), "git {args:?} failed");
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "test@example.com"]);
        run(&["config", "user.name", "test"]);
        dir
    }

    fn commit(dir: &Path, filename: &str, message: &str) {
        fs::write(dir.join(filename), "content").unwrap();
        Command::new("git")
            .args(["add", filename])
            .current_dir(dir)
            .status()
            .unwrap();
        Command::new("git")
            .args(["commit", "-q", "-m", message])
            .current_dir(dir)
            .status()
            .unwrap();
    }

    #[test]
    fn retrieves_full_messages_in_chronological_order() {
        let repo = temp_repo("order");
        commit(&repo, "a.txt", "feat: add first thing");
        commit(&repo, "b.txt", "fix: correct second thing");

        let messages = commit_messages(&repo, None).unwrap();
        assert_eq!(
            messages,
            vec![
                "feat: add first thing".to_string(),
                "fix: correct second thing".to_string()
            ]
        );

        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn a_multiline_body_with_blank_lines_is_captured_whole() {
        let repo = temp_repo("multiline");
        commit(
            &repo,
            "a.txt",
            "feat: add thing\n\nFirst paragraph.\n\nSecond paragraph.",
        );

        let messages = commit_messages(&repo, None).unwrap();
        assert_eq!(
            messages.len(),
            1,
            "the blank lines inside the body must not split it into multiple entries"
        );
        assert!(messages[0].contains("First paragraph."));
        assert!(messages[0].contains("Second paragraph."));

        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn revision_range_limits_which_commits_come_back() {
        let repo = temp_repo("range");
        commit(&repo, "a.txt", "feat: first");
        let tag_status = Command::new("git")
            .args(["tag", "v1.0.0"])
            .current_dir(&repo)
            .status()
            .unwrap();
        assert!(tag_status.success());
        commit(&repo, "b.txt", "feat: second, after the tag");

        let messages = commit_messages(&repo, Some("v1.0.0..HEAD")).unwrap();
        assert_eq!(messages, vec!["feat: second, after the tag".to_string()]);

        fs::remove_dir_all(&repo).ok();
    }
}
