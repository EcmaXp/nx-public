//! Grammar parity: the parse tree, as the classifier sees it.
//!
//! The commands here are authored rather than sampled, so this fixture carries
//! no machine-specific content and can be read by anyone. The expected values
//! were produced by the python implementation this crate replaces, which makes
//! the file a record of the behavior being preserved rather than of the
//! behavior this crate happens to have.
//!
//! What is pinned is deliberately narrow: `has_error`, the mis-parse cut
//! offsets, and the token segments. That triple is the complete image of the
//! tree under classification, and unlike an S-expression it contains nothing
//! that can differ without changing a decision.

use std::process::Command;

use serde_json::Value;

const BIN: &str = env!("CARGO_BIN_EXE_nx-bash-safe");
const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/grammar.jsonl");

/// The environment the fixture was generated under. `$TMPDIR` and
/// `$SCRATCHPAD` are trusted only when they resolve into tmp, so the expected
/// segments depend on their values and the test has to pin them.
fn dump() -> Vec<Value> {
    let output = Command::new(BIN)
        .arg("dump-segments")
        .arg(FIXTURE)
        .env("TMPDIR", "/tmp/claude")
        .env("SCRATCHPAD", "/tmp/claude/scratchpad")
        // Pinned like cases.rs: $HOME folds into the expected segments.
        .env("HOME", "/nonexistent")
        .output()
        .expect("run dump-segments");
    assert!(
        output.stderr.is_empty(),
        "dump-segments wrote stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("utf-8 output")
        .lines()
        .map(|line| serde_json::from_str(line).expect("ndjson row"))
        .collect()
}

fn expected() -> Vec<Value> {
    std::fs::read_to_string(FIXTURE)
        .expect("read fixture")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("fixture row"))
        .collect()
}

#[test]
fn the_tree_matches_the_implementation_being_replaced() {
    let want = expected();
    let got = dump();
    assert_eq!(want.len(), got.len(), "one output row per fixture row");

    let mut failures = Vec::new();
    for (want_row, got_row) in want.iter().zip(&got) {
        let command = want_row["c"].as_str().unwrap_or_default();
        for field in ["e", "cuts", "segs"] {
            if want_row[field] != got_row[field] {
                failures.push(format!(
                    "{command:?}\n    {field}: want {} got {}",
                    want_row[field], got_row[field]
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} divergences:\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}

/// A parse error is the fail-safe gate, so the fixture has to keep exercising
/// it: a tree we could not build is a tree we cannot make claims about.
#[test]
fn the_fixture_still_covers_the_structural_traps() {
    let want = expected();
    let errored = want
        .iter()
        .filter(|row| row["e"] == Value::Bool(true))
        .count();
    let cut = want
        .iter()
        .filter(|row| row["cuts"].as_array().is_some_and(|cuts| !cuts.is_empty()))
        .count();
    assert!(errored > 0, "no parse-error case left in the fixture");
    assert!(cut >= 2, "the mis-parse re-split needs more than one case");
}
