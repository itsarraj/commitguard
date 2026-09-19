use std::collections::BTreeMap;

use crate::parse::ConventionalCommit;

/// Which changelog section a commit type maps to, and the order sections
/// appear in — `feat`/`fix` first since those are what a reader of a
/// changelog actually cares about, everything else after.
fn section_for(commit_type: &str) -> &'static str {
    match commit_type {
        "feat" => "Features",
        "fix" => "Bug Fixes",
        "perf" => "Performance",
        "revert" => "Reverts",
        _ => "Other",
    }
}

const SECTION_ORDER: &[&str] = &["Features", "Bug Fixes", "Performance", "Reverts", "Other"];

/// Groups already-parsed commits by changelog section, in a stable
/// (insertion-order-within-group) order — pure, so it's testable without
/// touching git at all; `gitlog.rs` is what actually feeds it real data.
pub fn group_commits(
    commits: &[ConventionalCommit],
) -> BTreeMap<&'static str, Vec<&ConventionalCommit>> {
    let mut groups: BTreeMap<&'static str, Vec<&ConventionalCommit>> = BTreeMap::new();
    for commit in commits {
        groups
            .entry(section_for(&commit.commit_type))
            .or_default()
            .push(commit);
    }
    groups
}

pub fn render_changelog(commits: &[ConventionalCommit], version_label: &str) -> String {
    let groups = group_commits(commits);
    let breaking: Vec<&ConventionalCommit> = commits.iter().filter(|c| c.breaking).collect();

    let mut out = String::new();
    out.push_str(&format!("## {version_label}\n\n"));

    if !breaking.is_empty() {
        out.push_str("### BREAKING CHANGES\n\n");
        for c in &breaking {
            out.push_str(&format!("- {}\n", describe(c)));
        }
        out.push('\n');
    }

    for section in SECTION_ORDER {
        let Some(entries) = groups.get(section) else {
            continue;
        };
        out.push_str(&format!("### {section}\n\n"));
        for c in entries {
            out.push_str(&format!("- {}\n", describe(c)));
        }
        out.push('\n');
    }

    out
}

fn describe(commit: &ConventionalCommit) -> String {
    match &commit.scope {
        Some(scope) => format!("**{scope}:** {}", commit.description),
        None => commit.description.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(
        commit_type: &str,
        scope: Option<&str>,
        description: &str,
        breaking: bool,
    ) -> ConventionalCommit {
        ConventionalCommit {
            commit_type: commit_type.to_string(),
            scope: scope.map(str::to_string),
            breaking,
            description: description.to_string(),
            body: None,
        }
    }

    #[test]
    fn groups_by_section_correctly() {
        let commits = vec![
            commit("feat", None, "add search", false),
            commit("fix", None, "correct typo", false),
            commit("chore", None, "bump deps", false),
        ];
        let groups = group_commits(&commits);
        assert_eq!(groups["Features"].len(), 1);
        assert_eq!(groups["Bug Fixes"].len(), 1);
        assert_eq!(groups["Other"].len(), 1);
    }

    #[test]
    fn breaking_commits_get_their_own_section_in_the_render() {
        let commits = vec![commit("feat", Some("api"), "remove v1 endpoints", true)];
        let text = render_changelog(&commits, "v2.0.0");
        assert!(text.contains("### BREAKING CHANGES"));
        assert!(text.contains("**api:** remove v1 endpoints"));
    }

    #[test]
    fn sections_appear_in_a_stable_reader_friendly_order() {
        let commits = vec![
            commit("chore", None, "bump deps", false),
            commit("fix", None, "correct bug", false),
            commit("feat", None, "add thing", false),
        ];
        let text = render_changelog(&commits, "v1.0.0");
        let features_pos = text.find("### Features").unwrap();
        let fixes_pos = text.find("### Bug Fixes").unwrap();
        let other_pos = text.find("### Other").unwrap();
        assert!(features_pos < fixes_pos);
        assert!(fixes_pos < other_pos);
    }

    #[test]
    fn empty_commit_list_still_renders_a_header_without_panicking() {
        let text = render_changelog(&[], "v0.1.0");
        assert!(text.starts_with("## v0.1.0"));
    }
}
