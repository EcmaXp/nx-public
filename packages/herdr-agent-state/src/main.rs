//! Claude Code hook body for herdr, replacing the python block in herdr's
//! `herdr-agent-state.sh` asset (integration id `claude`, version 9).
//!
//! Reads a Claude Code hook payload on stdin and reports the session to the
//! herdr socket. Never writes to stdout or stderr, and always exits 0: Claude
//! Code surfaces hook output in the transcript.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value, json};

const SOURCE: &str = "herdr:claude";
const AGENT: &str = "claude";
const METHOD: &str = "pane.report_agent_session";
const TIMEOUT: Duration = Duration::from_millis(500);

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let _ = std::panic::catch_unwind(|| {
        let _ = run();
    });
    std::process::exit(0);
}

fn run() -> Option<()> {
    if std::env::args().nth(1)? != AGENT {
        return None;
    }
    let pane_id = env_var("HERDR_PANE_ID")?;
    let socket_path = env_var("HERDR_SOCKET_PATH")?;

    let mut raw = String::new();
    std::io::stdin().read_to_string(&mut raw).ok()?;
    let hook: Value = serde_json::from_str(raw.trim()).ok()?;

    send(&socket_path, &request(&hook, &pane_id)?)
}

fn env_var(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|value| !value.is_empty())
}

fn request(hook: &Value, pane_id: &str) -> Option<Value> {
    // A subagent turn is not the pane's session.
    if hook.get("agent_id").is_some_and(is_truthy) {
        return None;
    }
    if str_field(hook, "hook_event_name")? != "SessionStart" {
        return None;
    }
    if env_var("CURSOR_VERSION").is_some() || hook.get("cursor_version").is_some() {
        return None;
    }
    let session_id = str_field(hook, "session_id")?;

    let mut params = Map::new();
    params.insert("pane_id".into(), json!(pane_id));
    params.insert("source".into(), json!(SOURCE));
    params.insert("agent".into(), json!(AGENT));
    params.insert("seq".into(), json!(now_nanos()));
    params.insert("agent_session_id".into(), json!(session_id));
    if let Some(path) = str_field(hook, "transcript_path") {
        params.insert("agent_session_path".into(), json!(path));
    }
    if let Some(start_source) = str_field(hook, "source") {
        params.insert("session_start_source".into(), json!(start_source));
    }

    Some(json!({
        "id": request_id(),
        "method": METHOD,
        "params": Value::Object(params),
    }))
}

fn str_field(hook: &Value, key: &str) -> Option<String> {
    hook.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

/// Mirrors python truthiness, which is what the replaced hook body used.
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().is_some_and(|value| value != 0.0),
        Value::String(text) => !text.is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(fields) => !fields.is_empty(),
    }
}

fn now_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos() as u64)
        .unwrap_or(0)
}

fn request_id() -> String {
    let nanos = now_nanos();
    format!("{SOURCE}:{}:{:06}", nanos / 1_000_000, nanos % 1_000_000)
}

fn send(socket_path: &str, request: &Value) -> Option<()> {
    let mut stream = UnixStream::connect(socket_path).ok()?;
    stream.set_write_timeout(Some(TIMEOUT)).ok()?;
    stream.set_read_timeout(Some(TIMEOUT)).ok()?;

    let mut line = serde_json::to_vec(request).ok()?;
    line.push(b'\n');
    stream.write_all(&line).ok()?;
    stream.flush().ok()?;

    let mut response = [0u8; 4096];
    let _ = stream.read(&mut response);
    Some(())
}
