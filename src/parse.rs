use std::sync::OnceLock;

use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub struct ConventionalCommit {
    pub commit_type: String,
    pub scope: Option<String>,
    pub breaking: bool,
    pub description: String,
    pub body: Option<String>,
}

fn subject_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"^(?P<type>[a-zA-Z]+)(\((?P<scope>[^)]+)\))?(?P<breaking>!)?: (?P<description>.+)$",
        )
        .unwrap()
    })
}

/// Parses a full commit message (subject + optional blank line + body)
/// against the Conventional Commits spec's subject-line grammar:
/// `type(scope)!: description`. Returns `Err` with a human-readable reason
/// rather than `None` — a hook or CI check wants to say *why* it failed,
/// not just that it did.
pub fn parse(message: &str) -> Result<ConventionalCommit, String> {
    let mut lines = message.lines();
    let subject = lines.next().unwrap_or("").trim();
    if subject.is_empty() {
        return Err("commit message is empty".to_string());
    }

    let caps = subject_re().captures(subject).ok_or_else(|| {
        format!("subject line doesn't match 'type(scope)!: description' — got: {subject:?}")
    })?;

    let commit_type = caps.name("type").unwrap().as_str().to_string();
    let scope = caps.name("scope").map(|m| m.as_str().to_string());
    let breaking = caps.name("breaking").is_some();
    let description = caps.name("description").unwrap().as_str().to_string();

    let rest: Vec<&str> = lines.collect();
    let body_text = rest.join("\n");
    let body = if body_text.trim().is_empty() {
        None
    } else {
        Some(body_text.trim().to_string())
    };
    let breaking = breaking
        || body
            .as_deref()
            .is_some_and(|b| b.contains("BREAKING CHANGE"));

    Ok(ConventionalCommit {
        commit_type,
        scope,
        breaking,
        description,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_basic_conventional_commit() {
        let c = parse("feat: add dark mode toggle").unwrap();
        assert_eq!(c.commit_type, "feat");
        assert_eq!(c.scope, None);
        assert!(!c.breaking);
        assert_eq!(c.description, "add dark mode toggle");
    }

    #[test]
    fn parses_scope_and_breaking_marker() {
        let c = parse("feat(api)!: remove deprecated v1 endpoints").unwrap();
        assert_eq!(c.commit_type, "feat");
        assert_eq!(c.scope.as_deref(), Some("api"));
        assert!(c.breaking);
    }

    #[test]
    fn breaking_change_footer_also_marks_breaking_even_without_the_bang() {
        let msg =
            "feat: change response shape\n\nBREAKING CHANGE: `id` is now a string, not a number";
        let c = parse(msg).unwrap();
        assert!(
            c.breaking,
            "a BREAKING CHANGE footer must count even without '!' in the subject"
        );
    }

    #[test]
    fn body_is_captured_when_present() {
        let msg = "fix: correct off-by-one in pagination\n\nThe last page was being dropped.";
        let c = parse(msg).unwrap();
        assert_eq!(c.body.as_deref(), Some("The last page was being dropped."));
    }

    #[test]
    fn non_conventional_subject_is_a_descriptive_error_not_a_panic() {
        let result = parse("fixed the bug");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("fixed the bug"));
    }

    #[test]
    fn empty_message_is_a_clean_error() {
        assert!(parse("").is_err());
    }
}
