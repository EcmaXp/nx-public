//! The invariants that make a writable policy file safe to load.
//!
//! A policy file grants auto-approval, so it is worth being precise about what
//! it can and cannot do. It is not a new class of risk: the permission rules in
//! a settings file already grant the same thing from a writable path. What
//! keeps it bounded is that an overlay may only *add*, and that anything
//! unparseable is dropped rather than guessed at.

use std::path::Path;

use nx_bash_safe::policy::Policy;
use nx_bash_safe::policy::load::{Report, merge};

const BASELINE: &str = r#"
version = 1
readers = ["cat"]
help_flags = ["--help"]

[[command]]
name = "git"
kind = "native"
matcher = "git"

[command.tables]
status = "--short -s --branch -b"
"#;

fn load(documents: &[(&str, &str)]) -> (Policy, Report) {
    let mut policy = Policy::empty();
    let mut report = Report::default();
    for (name, text) in documents {
        merge(&mut policy, &mut report, Path::new(name), text);
    }
    (policy, report)
}

fn classify(policy: &Policy, command: &str) -> Option<String> {
    nx_bash_safe::classify(command, policy)
}

#[test]
fn an_overlay_may_add_a_command_the_baseline_does_not_define() {
    let overlay = r#"
version = 1
[[command]]
name = "mytool"
kind = "paths"
paths = ["show"]
"#;
    let (policy, report) = load(&[("baseline", BASELINE), ("overlay", overlay)]);
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    assert_eq!(
        classify(&policy, "mytool show x").as_deref(),
        Some("mytool show")
    );
    assert_eq!(classify(&policy, "mytool delete x"), None);
}

/// The invariant that matters most. Merging instead of rejecting would let an
/// overlay restate `git` as a reader, which accepts every argument, and the
/// flag validation that makes `git` safe would simply stop happening.
#[test]
fn an_overlay_cannot_redefine_a_compiled_in_command() {
    let hostile = r#"
version = 1
[[command]]
name = "git"
kind = "reader"
"#;
    let (policy, report) = load(&[("baseline", BASELINE), ("hostile", hostile)]);
    assert!(
        report
            .problems
            .iter()
            .any(|problem| problem.contains("git")),
        "the collision must be reported: {:?}",
        report.problems
    );
    // Still the compiled-in rule: a push is not a read.
    assert_eq!(
        classify(&policy, "git status").as_deref(),
        Some("git status")
    );
    assert_eq!(classify(&policy, "git push origin main"), None);
}

#[test]
fn a_malformed_row_is_dropped_without_taking_its_neighbours() {
    let overlay = r#"
version = 1

[[command]]
name = "good"
kind = "reader"

[[command]]
name = "nokind"

[[command]]
name = "badkind"
kind = "wishful"

[[command]]
name = "nomatcher"
kind = "native"

[[command]]
name = "alsogood"
kind = "reader"
"#;
    let (policy, report) = load(&[("baseline", BASELINE), ("overlay", overlay)]);
    assert_eq!(report.problems.len(), 3, "{:?}", report.problems);
    assert_eq!(classify(&policy, "good x").as_deref(), Some("good"));
    assert_eq!(classify(&policy, "alsogood x").as_deref(), Some("alsogood"));
    assert_eq!(classify(&policy, "nokind x"), None);
    assert_eq!(classify(&policy, "badkind x"), None);
    assert_eq!(classify(&policy, "nomatcher x"), None);
}

/// Naming a matcher that does not exist has to narrow, not widen: the rule is
/// kept but can never say yes, so a typo costs prompts rather than safety.
#[test]
fn an_unknown_matcher_never_allows() {
    let overlay = r#"
version = 1
[[command]]
name = "mytool"
kind = "native"
matcher = "does_not_exist"
"#;
    let (policy, _) = load(&[("baseline", BASELINE), ("overlay", overlay)]);
    assert_eq!(classify(&policy, "mytool anything"), None);
}

#[test]
fn a_document_that_does_not_parse_leaves_the_policy_intact() {
    let (policy, report) = load(&[("baseline", BASELINE), ("broken", "[[command]\nname =")]);
    assert_eq!(report.problems.len(), 1);
    assert_eq!(
        classify(&policy, "git status").as_deref(),
        Some("git status")
    );
}

/// Dropping everything is the same as allowing nothing, which is what makes a
/// missing or unreadable policy safe to ignore instead of fatal.
#[test]
fn an_empty_policy_allows_nothing() {
    let policy = Policy::empty();
    for command in ["git status", "ls", "cat f", "echo hi"] {
        assert_eq!(classify(&policy, command), None, "{command}");
    }
}
