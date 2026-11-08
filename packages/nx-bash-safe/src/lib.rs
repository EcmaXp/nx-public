//! Decide whether a Bash command is provably read-only.
//!
//! The shape of the argument, in order:
//!
//! 1. **Parse** the command with tree-sitter-bash. Every node is consumed by an
//!    explicit branch and an unrecognized one aborts, because the grammar nests
//!    trailing structure in surprising places: in `cat <<'PY' | uv run python -`
//!    the whole trailing pipeline hangs off the heredoc node, so a walker that
//!    reads only the children it expects would classify it as a bare `cat`.
//! 2. **Project** the tree down to flat token segments, one per command.
//! 3. **Classify** each segment against the policy, which is data: a rule
//!    matches the command name and then validates its flags against a
//!    default-deny table. Anything unmatched aborts.
//!
//! Every abort means "prompt", never "deny". The asymmetry is deliberate: an
//! over-allow silently deletes a decision the user would have been asked to
//! make, while an under-allow costs one prompt.

pub mod audit;
pub mod env;
pub mod explain;
pub mod flags;
pub mod hook;
pub mod parse;
pub mod pattern;
pub mod policy;
pub mod prompt;
pub mod replay;
pub mod validate;

use serde_json::Value;

use crate::policy::Policy;

/// Joins per-segment reasons. Plan-mode suppression splits on it, so it is an
/// interface, not formatting.
pub const REASON_JOIN: &str = ", ";

/// Classify a whole command line: `Some(reason)` to allow, `None` to prompt.
pub fn classify(command: &str, policy: &Policy) -> Option<String> {
    if command.trim().is_empty() {
        return None;
    }
    parse::classify_source(command.as_bytes(), policy, false)
}

/// Allow-reason for a Bash `tool_input`, or `None` to pass through.
///
/// The sandbox is the trust boundary: sandbox-off keeps its allow only for
/// entitled reasons, so a call that could have run sandboxed goes back
/// through the permission gate. Plan mode suppresses mutating reasons.
pub fn decide(tool_input: &Value, permission_mode: &str, policy: &Policy) -> Option<String> {
    let command = tool_input.get("command")?.as_str()?;
    let mut reason = classify(command, policy)?;
    if permission_mode == "plan"
        && reason
            .split(REASON_JOIN)
            .any(|segment| policy.blocks_in_plan_mode(segment))
    {
        return None;
    }
    if truthy(tool_input.get("dangerouslyDisableSandbox")) {
        // One entitled segment required, mutating segments (the plan-mode
        // block list) forbidden; pure reads riding the pipeline stay allowed.
        let mut entitled = false;
        for segment in reason.split(REASON_JOIN) {
            if policy.blocks_in_plan_mode(segment) {
                return None;
            }
            entitled |= policy.sandbox_off_reason_ok(segment);
        }
        if !entitled {
            return None;
        }
        reason.push_str(" (sandbox-off)");
    }
    Some(reason)
}

/// Mirrors python truthiness, which is what the payload producer assumes.
fn truthy(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => false,
        Some(Value::Bool(flag)) => *flag,
        Some(Value::Number(number)) => number.as_f64().is_some_and(|value| value != 0.0),
        Some(Value::String(text)) => !text.is_empty(),
        Some(Value::Array(items)) => !items.is_empty(),
        Some(Value::Object(fields)) => !fields.is_empty(),
    }
}
