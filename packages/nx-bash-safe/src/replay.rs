//! Batch modes that make this implementation checkable against another.
//!
//! Both are NDJSON in, NDJSON out, one row per input row **in input order**.
//! Order is the pairing key: a fixture holds one row per distinct
//! `(command, sandbox_off, mode)`, so the same command text legitimately
//! appears twice and its hash does not identify a row. The hash is still
//! emitted, so `diff` on two streams is meaningful and neither artifact carries
//! a command line.
//!
//! Everything here runs in one process. Spawning per command would cost over an
//! hour on the frozen corpus, which is the difference between a gate that runs
//! on every change and one that runs once.

use std::io::{BufRead, BufWriter, Write};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::parse::{self, OPAQUE, OPAQUE_LABEL};
use crate::policy::Policy;

fn row_hash(command: &str) -> String {
    let digest = Sha256::digest(command.as_bytes());
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

struct Row {
    command: String,
    sandbox_off: bool,
    mode: String,
}

fn read_rows(path: &str) -> std::io::Result<Vec<Row>> {
    let file = std::fs::File::open(path)?;
    let mut rows = Vec::new();
    for line in std::io::BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line)?;
        let Some(command) = value.get("c").and_then(Value::as_str) else {
            continue;
        };
        rows.push(Row {
            command: command.to_owned(),
            sandbox_off: value.get("s").and_then(Value::as_bool).unwrap_or(false),
            mode: value
                .get("m")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
        });
    }
    Ok(rows)
}

/// Classify a whole fixture, emitting the decision for each row.
pub fn replay(path: &str, policy: &Policy) -> std::io::Result<()> {
    let rows = read_rows(path)?;
    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    for row in rows {
        let mut tool_input = json!({"command": row.command});
        if row.sandbox_off {
            tool_input["dangerouslyDisableSandbox"] = Value::Bool(true);
        }
        let reason = crate::decide(&tool_input, &row.mode, policy);
        let record = json!({
            "h": row_hash(&row.command),
            "a": reason.is_some(),
            "r": reason,
        });
        writeln!(out, "{record}")?;
    }
    Ok(())
}

/// Project each fixture command down to what classification would see.
pub fn dump_segments(path: &str, policy: &Policy) -> std::io::Result<()> {
    let rows = read_rows(path)?;
    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    for row in rows {
        let dump = parse::dump_source(row.command.as_bytes(), policy);
        let segments: Vec<Vec<String>> = dump
            .segments
            .into_iter()
            .map(|segment| {
                segment
                    .into_iter()
                    .map(|token| token.replace(OPAQUE, OPAQUE_LABEL))
                    .collect()
            })
            .collect();
        let record = json!({
            "h": row_hash(&row.command),
            "e": dump.errored,
            "cuts": dump.cuts,
            "segs": segments,
        });
        writeln!(out, "{record}")?;
    }
    Ok(())
}
