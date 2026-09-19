use crate::parse::{self, ConventionalCommit};

pub const ALLOWED_TYPES: &[&str] = &[
    "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert",
];

const MAX_SUBJECT_LEN: usize = 100;

#[derive(Debug, Clone, PartialEq)]
pub struct LintIssue {
    pub rule: &'static str,
    pub message: String,
}

/// Lints a full commit message. If it doesn't even parse as a
/// Conventional Commit, that's the one issue reported (the specific rule
/// checks below all assume a successful parse, so there's nothing more
/// useful to say). Otherwise runs every rule against the parsed result —
/// a message can fail more than one rule at once.
pub fn lint(message: &str) -> Vec<LintIssue> {
    let commit = match parse::parse(message) {
        Ok(c) => c,
        Err(reason) => {
            return vec![LintIssue {
                rule: "format",
                message: reason,
            }]
        }
    };

    let mut issues = Vec::new();
    check_type(&commit, &mut issues);
    check_description_case(&commit, &mut issues);
    check_description_no_trailing_period(&commit, &mut issues);
    check_subject_length(message, &mut issues);
    issues
}

fn check_type(commit: &ConventionalCommit, issues: &mut Vec<LintIssue>) {
    if !ALLOWED_TYPES.contains(&commit.commit_type.as_str()) {
        issues.push(LintIssue {
            rule: "type-enum",
            message: format!(
                "'{}' is not an allowed type — expected one of: {}",
                commit.commit_type,
                ALLOWED_TYPES.join(", ")
            ),
        });
    }
}

fn check_description_case(commit: &ConventionalCommit, issues: &mut Vec<LintIssue>) {
    if let Some(first) = commit.description.chars().next() {
        if first.is_uppercase() {
            issues.push(LintIssue {
                rule: "description-case",
                message: "description should start with a lowercase letter".to_string(),
            });
        }
    }
}

fn check_description_no_trailing_period(commit: &ConventionalCommit, issues: &mut Vec<LintIssue>) {
    if commit.description.trim_end().ends_with('.') {
        issues.push(LintIssue {
            rule: "description-full-stop",
            message: "description should not end with a period".to_string(),
        });
    }
}

fn check_subject_length(message: &str, issues: &mut Vec<LintIssue>) {
    let subject = message.lines().next().unwrap_or("");
    if subject.chars().count() > MAX_SUBJECT_LEN {
        issues.push(LintIssue {
            rule: "subject-max-length",
            message: format!(
                "subject line is {} characters, max is {MAX_SUBJECT_LEN}",
                subject.chars().count()
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_commit_has_no_issues() {
        assert!(lint("fix: correct pagination off-by-one").is_empty());
    }

    #[test]
    fn unparseable_subject_yields_exactly_one_format_issue() {
        let issues = lint("fixed a thing");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule, "format");
    }

    #[test]
    fn unknown_type_is_flagged() {
        let issues = lint("feet: typo in the type itself");
        assert!(issues.iter().any(|i| i.rule == "type-enum"));
    }

    #[test]
    fn uppercase_description_is_flagged() {
        let issues = lint("fix: Correct the bug");
        assert!(issues.iter().any(|i| i.rule == "description-case"));
    }

    #[test]
    fn trailing_period_is_flagged() {
        let issues = lint("fix: correct the bug.");
        assert!(issues.iter().any(|i| i.rule == "description-full-stop"));
    }

    #[test]
    fn overlong_subject_is_flagged() {
        let long_desc = "a".repeat(120);
        let issues = lint(&format!("feat: {long_desc}"));
        assert!(issues.iter().any(|i| i.rule == "subject-max-length"));
    }

    #[test]
    fn a_message_can_fail_multiple_rules_at_once() {
        let issues = lint("feet: Fix the thing.");
        let rules: Vec<&str> = issues.iter().map(|i| i.rule).collect();
        assert!(rules.contains(&"type-enum"));
        assert!(rules.contains(&"description-case"));
        assert!(rules.contains(&"description-full-stop"));
    }
}
