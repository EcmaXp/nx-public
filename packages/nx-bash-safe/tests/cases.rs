//! The classification cases, shared with the implementation being replaced.
//!
//! One copy of the data, read by both, so the two cannot drift. Nearly half the
//! cases expect a *rejection*, which is the half that matters: they record
//! specific ways a command can look read-only and not be, and several were
//! found by comparing against the classifier embedded in Claude Code itself
//! (`git -c core.fsmonitor=...`, `fd -x`, `sort -o`, a sed script's `w`).
//!
//! The `r` column is newer than the cases. The original suite asserted only
//! allow-or-prompt, which left the reason string, the thing plan mode parses
//! back apart, with no coverage at all.
//!
//! Only the cases the **baseline** decides are here. The ones that depend on
//! the workspace overlay live beside that overlay, because a test that needs a
//! file this crate does not ship is a test that passes on one machine and
//! fails in a packaged build. Every rejection case is in this half.

use std::process::Command;

use serde_json::{Value, json};

const BIN: &str = env!("CARGO_BIN_EXE_nx-bash-safe");
const CASES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cases.jsonl");
const DECIDE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/decide.jsonl");

fn rows(path: &str) -> Vec<Value> {
    std::fs::read_to_string(path)
        .expect("read fixture")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("fixture row"))
        .collect()
}

/// Replay a fixture through the real binary, under the environment the
/// expectations were recorded in: `$TMPDIR` and `$SCRATCHPAD` are trusted only
/// when they resolve into tmp, so they decide some of these cases.
fn replay(rows: &[Value]) -> Vec<Value> {
    let input: String = rows
        .iter()
        .map(|row| format!("{}\n", json!({"c": row["c"], "s": row["s"], "m": row["m"]})))
        .collect();
    let scratch =
        std::env::temp_dir().join(format!("nx-bash-safe-cases-{}.jsonl", std::process::id()));
    std::fs::write(&scratch, input).expect("write scratch fixture");
    let output = Command::new(BIN)
        .arg("replay")
        .arg(&scratch)
        .env("TMPDIR", "/tmp/claude")
        .env("SCRATCHPAD", "/tmp/claude/scratchpad")
        // No HOME means no overlay, so these run against the baseline alone.
        // That is what keeps this suite hermetic and identical under a packaged
        // build, where the overlay does not exist. The cases the overlay
        // decides live beside the overlay.
        .env("HOME", "/nonexistent")
        .output()
        .expect("run replay");
    let _ = std::fs::remove_file(&scratch);
    assert!(
        output.stderr.is_empty(),
        "replay wrote stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("utf-8")
        .lines()
        .map(|line| serde_json::from_str(line).expect("ndjson row"))
        .collect()
}

#[test]
fn classification_matches_the_recorded_expectations() {
    let want = rows(CASES);
    let got = replay(&want);
    assert_eq!(want.len(), got.len());

    let mut failures = Vec::new();
    for (want_row, got_row) in want.iter().zip(&got) {
        let command = want_row["c"].as_str().unwrap_or_default();
        if want_row["allow"] != got_row["a"] {
            failures.push(format!(
                "{command:?}: want {}, got {}",
                want_row["allow"], got_row["a"]
            ));
        } else if want_row["r"] != got_row["r"] {
            failures.push(format!(
                "{command:?}: reason want {} got {}",
                want_row["r"], got_row["r"]
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} cases failed:\n  {}",
        failures.len(),
        want.len(),
        failures.join("\n  ")
    );
}

#[test]
fn the_suite_still_spends_most_of_itself_on_rejections() {
    let cases = rows(CASES);
    let rejects = cases.iter().filter(|row| row["allow"] == false).count();
    assert!(
        rejects * 2 >= cases.len(),
        "only {rejects} of {} cases expect a rejection; the suite is drifting \
         toward testing what works instead of what must not",
        cases.len()
    );
}

/// Plan mode and the sandbox annotation live in `decide`, not `classify`, and
/// the corpus cannot cover them: a suppressed call prints nothing, and nothing
/// is what the transcripts record.
#[test]
fn plan_mode_and_the_sandbox_flag_match_the_recorded_expectations() {
    let want = rows(DECIDE);
    let input: String = want
        .iter()
        .map(|row| {
            let tool_input = &row["input"];
            format!(
                "{}\n",
                json!({
                    "c": tool_input["command"],
                    "s": tool_input.get("dangerouslyDisableSandbox").is_some_and(|flag| flag == true),
                    "m": row["m"],
                })
            )
        })
        .collect();
    let scratch =
        std::env::temp_dir().join(format!("nx-bash-safe-decide-{}.jsonl", std::process::id()));
    std::fs::write(&scratch, input).expect("write scratch fixture");
    let output = Command::new(BIN)
        .arg("replay")
        .arg(&scratch)
        .env("TMPDIR", "/tmp/claude")
        .env("SCRATCHPAD", "/tmp/claude/scratchpad")
        // No HOME means no overlay, so these run against the baseline alone.
        // That is what keeps this suite hermetic and identical under a packaged
        // build, where the overlay does not exist. The cases the overlay
        // decides live beside the overlay.
        .env("HOME", "/nonexistent")
        .output()
        .expect("run replay");
    let _ = std::fs::remove_file(&scratch);
    let got: Vec<Value> = String::from_utf8(output.stdout)
        .expect("utf-8")
        .lines()
        .map(|line| serde_json::from_str(line).expect("ndjson row"))
        .collect();

    let mut failures = Vec::new();
    for (want_row, got_row) in want.iter().zip(&got) {
        if want_row["allow"] != got_row["a"] || want_row["r"] != got_row["r"] {
            failures.push(format!(
                "{} in mode {}: want {} / {}, got {} / {}",
                want_row["input"]["command"],
                want_row["m"],
                want_row["allow"],
                want_row["r"],
                got_row["a"],
                got_row["r"],
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
