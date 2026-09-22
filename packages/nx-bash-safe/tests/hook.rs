//! The hook contract, asserted before any policy exists to satisfy it.
//!
//! What is pinned here is not "the right commands are allowed" (that is the
//! case fixture's job) but the surrounding discipline: an unparseable, hostile,
//! or simply irrelevant payload must produce no stdout, no stderr, and exit 0,
//! because Claude Code surfaces hook output in the transcript and a hook that
//! fails loudly is worse than one that stays quiet.

use std::io::Write;
use std::process::{Command, Output, Stdio};

use nx_bash_safe::hook::allow_json;
use serde_json::{Value, json};

const BIN: &str = env!("CARGO_BIN_EXE_nx-bash-safe");

fn run(stdin: &str) -> Output {
    let mut child = Command::new(BIN)
        .arg("hook")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(stdin.as_bytes())
        .expect("write");
    child.wait_with_output().expect("wait")
}

fn assert_silent(output: &Output, what: &str) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{what}: exit code must be 0, hook failures surface to the user"
    );
    assert!(
        output.stdout.is_empty(),
        "{what}: expected no stdout, got {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        output.stderr.is_empty(),
        "{what}: expected no stderr, got {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn empty_stdin_is_silent() {
    assert_silent(&run(""), "empty stdin");
}

#[test]
fn malformed_json_is_silent() {
    assert_silent(&run("{not json"), "malformed json");
    assert_silent(&run("[]"), "wrong toplevel type");
    assert_silent(&run("null"), "null payload");
}

#[test]
fn embedded_nul_is_silent() {
    assert_silent(&run("{\"tool_name\":\"Bash\0\"}"), "embedded NUL");
}

#[test]
fn a_payload_for_another_tool_is_silent() {
    let payload = json!({
        "tool_name": "Read",
        "tool_input": {"file_path": "/etc/passwd"},
    });
    assert_silent(&run(&payload.to_string()), "non-Bash tool");
}

#[test]
fn a_bash_payload_without_a_command_is_silent() {
    let payload = json!({"tool_name": "Bash", "tool_input": {}});
    assert_silent(&run(&payload.to_string()), "no command");
}

/// Size is the point here, not the verdict, so the command is one that
/// prompts: a huge *allowed* command would pass this by printing, and prove
/// nothing about surviving the input.
#[test]
fn an_oversized_payload_is_silent() {
    let payload = json!({
        "tool_name": "Bash",
        "tool_input": {"command": "git push origin ".repeat(20_000)},
    });
    assert_silent(&run(&payload.to_string()), "300KB payload");
}

/// The hook does its job when a policy is loaded: this is the other half of
/// the contract, and without it the silence assertions above would also pass
/// on a binary that never allows anything.
#[test]
fn a_read_only_command_is_allowed() {
    let payload = json!({
        "tool_name": "Bash",
        "tool_input": {"command": "git status && rg -n foo src/"},
    });
    let output = run(&payload.to_string());
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let body: Value =
        serde_json::from_slice(&output.stdout).expect("an allow decision is one JSON line");
    assert_eq!(
        body["hookSpecificOutput"]["permissionDecisionReason"],
        Value::String("safe command: git status, rg".to_owned())
    );
}

#[test]
fn an_unknown_verb_fails_loudly_but_hook_never_does() {
    let output = Command::new(BIN).arg("frobnicate").output().expect("run");
    assert_eq!(output.status.code(), Some(2));
    assert!(!output.stderr.is_empty());
}

/// The exact bytes an allow decision produces. Claude Code parses this, so the
/// assertion is on the parsed value, but the "safe command: " prefix is part of
/// what the user sees and what the recorded oracle contains.
#[test]
fn the_allow_body_matches_the_recorded_shape() {
    let body: Value = allow_json("git status, rg");
    assert_eq!(
        body,
        json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "permissionDecisionReason": "safe command: git status, rg",
            }
        })
    );
}
