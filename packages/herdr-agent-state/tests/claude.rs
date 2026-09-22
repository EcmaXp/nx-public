//! Socket-level fixtures: what the hook binary puts on the wire for each
//! Claude Code payload, and what it stays silent about.

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use serde_json::Value;

const BIN: &str = env!("CARGO_BIN_EXE_herdr-agent-state");
const PANE_ID: &str = "w1:pC0";
const ACCEPT_WAIT: Duration = Duration::from_millis(2000);

static COUNTER: AtomicU32 = AtomicU32::new(0);

struct Harness {
    listener: UnixListener,
    path: PathBuf,
}

impl Harness {
    fn new() -> Self {
        let unique = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path =
            std::env::temp_dir().join(format!("herdr-hook-{}-{unique}.sock", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).expect("bind test socket");
        listener
            .set_nonblocking(true)
            .expect("nonblocking listener");
        Self { listener, path }
    }

    /// Runs the hook and returns the reported line, plus the process output.
    fn report(
        &self,
        args: &[&str],
        stdin: &str,
        pane_id: Option<&str>,
    ) -> (Option<String>, Output) {
        let mut child = Command::new(BIN)
            .args(args)
            .env("HERDR_SOCKET_PATH", &self.path)
            .env_remove("HERDR_PANE_ID")
            .envs(pane_id.map(|id| ("HERDR_PANE_ID", id)))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn hook");
        child
            .stdin
            .take()
            .expect("stdin")
            .write_all(stdin.as_bytes())
            .expect("write payload");

        let line = self.accept_line(&mut child);
        let output = child.wait_with_output().expect("wait for hook");
        (line, output)
    }

    fn accept_line(&self, child: &mut Child) -> Option<String> {
        let deadline = Instant::now() + ACCEPT_WAIT;
        let stream = loop {
            match self.listener.accept() {
                Ok((stream, _)) => break stream,
                Err(_) if child.try_wait().expect("poll hook").is_some() => {
                    break self.listener.accept().ok()?.0;
                }
                Err(_) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(5)),
                Err(_) => return None,
            }
        };

        stream.set_nonblocking(false).expect("blocking stream");
        stream
            .set_read_timeout(Some(deadline - Instant::now()))
            .expect("read timeout");
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).expect("read reported line");
        // Answer so the hook does not sit out its own response timeout.
        reader
            .get_mut()
            .write_all(b"{\"id\":\"test\",\"result\":{}}\n")
            .expect("write response");
        Some(line)
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn reported(stdin: &str) -> Option<Value> {
    let harness = Harness::new();
    let (line, output) = harness.report(&["claude"], stdin, Some(PANE_ID));
    assert_silent(&output);
    line.map(|line| serde_json::from_str(&line).expect("reported line is JSON"))
}

fn assert_silent(output: &Output) {
    assert_eq!(output.status.code(), Some(0), "hook must always exit 0");
    assert!(output.stdout.is_empty(), "hook must not write stdout");
    assert!(output.stderr.is_empty(), "hook must not write stderr");
}

#[test]
fn session_start_reports_identity_and_start_source() {
    let value = reported(
        r#"{"hook_event_name":"SessionStart","source":"startup","session_id":"abc-123","transcript_path":"/tmp/t.jsonl"}"#,
    )
    .expect("SessionStart reports");

    assert_eq!(value["method"], "pane.report_agent_session");
    assert!(
        value["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("herdr:claude:"))
    );
    let params = &value["params"];
    assert_eq!(params["pane_id"], PANE_ID);
    assert_eq!(params["source"], "herdr:claude");
    assert_eq!(params["agent"], "claude");
    assert_eq!(params["agent_session_id"], "abc-123");
    assert_eq!(params["agent_session_path"], "/tmp/t.jsonl");
    assert_eq!(params["session_start_source"], "startup");
    assert!(params["seq"].as_u64().is_some_and(|seq| seq > 0));
}

#[test]
fn only_session_start_reports() {
    for event in ["Stop", "SubagentStop", "UserPromptSubmit", "SessionEnd"] {
        let payload =
            format!(r#"{{"hook_event_name":"{event}","source":"startup","session_id":"abc-123"}}"#);
        assert!(reported(&payload).is_none(), "{event} must stay silent");
    }
    assert!(reported(r#"{"session_id":"abc-123"}"#).is_none());
}

#[test]
fn cursor_sessions_are_ignored() {
    assert!(
        reported(
            r#"{"hook_event_name":"SessionStart","session_id":"abc-123","cursor_version":"1.2"}"#
        )
        .is_none()
    );

    let harness = Harness::new();
    let mut child = Command::new(BIN)
        .arg("claude")
        .env("HERDR_SOCKET_PATH", &harness.path)
        .env("HERDR_PANE_ID", PANE_ID)
        .env("CURSOR_VERSION", "1.2")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn hook");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(br#"{"hook_event_name":"SessionStart","session_id":"abc-123"}"#)
        .expect("write payload");
    assert!(
        harness.accept_line(&mut child).is_none(),
        "cursor env must not report"
    );
    assert_silent(&child.wait_with_output().expect("wait for hook"));
}

#[test]
fn missing_transcript_path_is_omitted() {
    let value = reported(r#"{"hook_event_name":"SessionStart","session_id":"abc-123"}"#)
        .expect("payload without transcript reports");

    assert!(value["params"].get("agent_session_path").is_none());
    assert!(value["params"].get("session_start_source").is_none());
}

#[test]
fn subagent_payload_is_ignored() {
    assert!(
        reported(r#"{"hook_event_name":"Stop","agent_id":"sub-1","session_id":"abc-123"}"#)
            .is_none()
    );
}

#[test]
fn payload_without_session_id_is_ignored() {
    assert!(reported(r#"{"hook_event_name":"SessionStart","source":"startup"}"#).is_none());
    assert!(reported(r#"{"hook_event_name":"SessionStart","session_id":""}"#).is_none());
}

#[test]
fn malformed_and_empty_payloads_are_ignored() {
    assert!(reported("not json").is_none());
    assert!(reported("").is_none());
    assert!(reported("   \n").is_none());
}

#[test]
fn other_targets_and_missing_pane_are_ignored() {
    let harness = Harness::new();
    let payload = r#"{"hook_event_name":"SessionStart","session_id":"abc-123"}"#;

    let (line, output) = harness.report(&["codex"], payload, Some(PANE_ID));
    assert_silent(&output);
    assert!(line.is_none(), "unknown target must not report");

    let (line, output) = harness.report(&["claude"], payload, None);
    assert_silent(&output);
    assert!(line.is_none(), "missing pane id must not report");
}

#[test]
fn unreachable_socket_stays_silent() {
    let missing = std::env::temp_dir().join("herdr-hook-does-not-exist.sock");
    let _ = std::fs::remove_file(&missing);

    let mut child = Command::new(BIN)
        .arg("claude")
        .env("HERDR_SOCKET_PATH", &missing)
        .env("HERDR_PANE_ID", PANE_ID)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn hook");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(br#"{"hook_event_name":"SessionStart","session_id":"abc-123"}"#)
        .expect("write payload");

    assert_silent(&child.wait_with_output().expect("wait for hook"));
}

/// The shipped v10 asset and this binary must put the same fields on the wire.
#[test]
fn matches_the_shipped_python_hook() {
    let payload = r#"{"hook_event_name":"SessionStart","source":"resume","session_id":"abc-123","transcript_path":"/tmp/t.jsonl"}"#;
    let ours = reported(payload).expect("binary reports");

    let baseline = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("baseline/claude-v10.sh");
    let harness = Harness::new();
    let mut child = Command::new("bash")
        .arg(&baseline)
        .arg("session")
        .env("HERDR_ENV", "1")
        .env("HERDR_SOCKET_PATH", &harness.path)
        .env("HERDR_PANE_ID", PANE_ID)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn baseline hook");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(payload.as_bytes())
        .expect("write payload");
    let line = harness.accept_line(&mut child).expect("baseline reports");
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("stderr")
        .read_to_string(&mut stderr)
        .expect("read stderr");
    child.wait().expect("wait for baseline hook");
    let theirs: Value = serde_json::from_str(&line).expect("baseline line is JSON");

    assert_eq!(ours["method"], theirs["method"]);
    assert_eq!(
        without_seq(&ours["params"]),
        without_seq(&theirs["params"]),
        "stderr: {stderr}"
    );
    assert!(theirs["params"]["seq"].as_u64().is_some());
}

/// `seq` is a timestamp, so parity is about every other field.
fn without_seq(params: &Value) -> Value {
    let mut params = params.clone();
    params.as_object_mut().expect("params object").remove("seq");
    params
}
