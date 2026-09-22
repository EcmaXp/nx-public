//! The PreToolUse hook itself: payload in, allow decision or silence out.
//!
//! Contract, inherited from the python implementation and load-bearing:
//! print an allow decision or nothing at all, never deny, never write stderr,
//! always exit 0. Anything unprovable falls through to the normal permission
//! flow, so silence is the safe direction and every error path takes it.

use std::io::Read;

use serde_json::{Value, json};

use crate::policy::Policy;

/// Claude Code shows this prefix to the user, so it is part of the output the
/// oracle recorded, not decoration.
const REASON_PREFIX: &str = "safe command: ";

/// The exact JSON body an allow decision produces.
///
/// Kept separate from stdout so the wire contract can be asserted directly,
/// before any policy exists to produce a reason.
pub fn allow_json(reason: &str) -> Value {
    json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "permissionDecisionReason": format!("{REASON_PREFIX}{reason}"),
        }
    })
}

/// Decide on an already-parsed payload. Returns the line to print, if any.
pub fn decide_payload(payload: &Value, policy: &Policy) -> Option<String> {
    if payload.get("tool_name").and_then(Value::as_str) != Some("Bash") {
        return None;
    }
    let tool_input = payload.get("tool_input")?;
    let mode = payload
        .get("permission_mode")
        .and_then(Value::as_str)
        .unwrap_or("");
    let reason = crate::decide(tool_input, mode, policy)?;
    Some(allow_json(&reason).to_string())
}

/// Read a payload on stdin and print an allow decision, or stay silent.
///
/// Returns `None` on every failure, which the caller turns into silence: a
/// malformed payload, an unreadable stdin, and an unprovable command are all
/// the same outcome to the user.
pub fn run(policy: &Policy) -> Option<()> {
    let mut raw = String::new();
    std::io::stdin().read_to_string(&mut raw).ok()?;
    let payload: Value = serde_json::from_str(raw.trim()).ok()?;
    let line = decide_payload(&payload, policy)?;
    println!("{line}");
    Some(())
}
