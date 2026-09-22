//! What the classifier does to real traffic, replayed out of session logs.
//!
//! This is the only instrument that exists for this hook. It has no CI and it
//! fails silently by design, so the way a regression shows up is not a crash
//! but a slow drift back toward prompting, and the only way to see that is to
//! measure the allow rate on the traffic that actually happened.
//!
//! Two column choices carry more weight than they look like they do. Rows are
//! ordered by `(-count, first seen)`, not by key, so a week-over-week
//! comparison is not reshuffled by ties. And `prompting` counts the sandbox-off
//! passthrough specifically, because a sandboxed call is auto-allowed before it
//! reaches a prompt: the sandbox-off subset is the surface the user actually
//! sees.
//!
//! Windows are **elapsed time**, not calendar dates: `today` is the last 24
//! hours and `1w` the last 7 days. The implementation this replaces bucketed by
//! local calendar date, which the standard library cannot compute without a
//! timezone database, so `today` here is a rolling window rather than a
//! midnight-to-midnight one. The longer windows agree closely, and the alarms
//! worth watching compare `1w` against `4w` rather than reading `today`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde_json::Value;

use crate::policy::{Policy, Rule};

const WINDOWS: &[&str] = &["today", "1w", "2w", "3w", "4w", "8w", "12w", "all"];
const DAY: u64 = 24 * 60 * 60;

struct Call {
    prefix: String,
    allowed: bool,
    sandbox_off: bool,
}

/// Session logs live under a directory keyed by the workspace path.
fn transcript_dir(workspace: &Path) -> PathBuf {
    let slug = workspace.to_string_lossy().replace('/', "-");
    PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
        .join(".claude/projects")
        .join(slug)
}

fn window_days(window: &str) -> Option<f64> {
    match window {
        "today" => Some(0.0),
        "all" => Some(f64::INFINITY),
        other => {
            let (count, unit) = other.split_at(other.len().checked_sub(1)?);
            let count: f64 = count.parse().ok()?;
            match unit {
                "w" => Some(count * 7.0),
                "d" => Some(count),
                _ => None,
            }
        }
    }
}

/// Bucket transcripts by file mtime, which is what "this week" means here.
fn files_in_window(dir: &Path, window: &str) -> Vec<PathBuf> {
    let Some(days) = window_days(window) else {
        return Vec::new();
    };
    let now = SystemTime::now();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "jsonl"))
        .filter(|path| {
            let Ok(modified) = path.metadata().and_then(|meta| meta.modified()) else {
                return false;
            };
            let age = now.duration_since(modified).unwrap_or(Duration::ZERO);
            if days == 0.0 {
                age.as_secs() < DAY
            } else if days.is_infinite() {
                true
            } else {
                (age.as_secs() as f64) < days * DAY as f64
            }
        })
        .collect();
    files.sort();
    files
}

fn bash_calls(path: &Path, policy: &Policy) -> Vec<Call> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut calls = Vec::new();
    for line in text.lines() {
        if !line.contains("\"Bash\"") {
            continue;
        }
        let Ok(record) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if record.get("type").and_then(Value::as_str) != Some("assistant") {
            continue;
        }
        let Some(content) = record
            .get("message")
            .and_then(|message| message.get("content"))
            .and_then(Value::as_array)
        else {
            continue;
        };
        for item in content {
            if item.get("type").and_then(Value::as_str) != Some("tool_use")
                || item.get("name").and_then(Value::as_str) != Some("Bash")
            {
                continue;
            }
            let input = item.get("input").cloned().unwrap_or(Value::Null);
            let Some(command) = input.get("command").and_then(Value::as_str) else {
                continue;
            };
            calls.push(Call {
                prefix: prefix_of(command, policy),
                allowed: crate::decide(&input, "", policy).is_some(),
                sandbox_off: input
                    .get("dangerouslyDisableSandbox")
                    .is_some_and(|flag| flag == &Value::Bool(true)),
            });
        }
    }
    calls
}

/// The group label: the first command that is not a `cd`.
fn prefix_of(command: &str, policy: &Policy) -> String {
    // A display preference, not policy: which tools are worth one word deeper
    // in the table. Not derivable from the rules, because grouping `jq` or
    // `sed` by their second word puts arbitrary user data in the label, and
    // `uv` has no rule at all. A namespaced dispatcher is the exception, and it
    // says so itself through `alias_prefix`.
    const GROUPED: &[&str] = &[
        "git",
        "gh",
        "aws",
        "terraform",
        "kubectl",
        "uv",
        "herdr",
        "pup",
        "mise",
    ];
    let dump = crate::parse::dump_source(command.as_bytes(), policy);
    let mut tokens: Vec<String> = Vec::new();
    for segment in &dump.segments {
        let stripped = crate::policy::strip_wrappers(segment, policy);
        if stripped.is_empty() {
            continue;
        }
        let is_cd = stripped[0] == "cd";
        tokens = stripped;
        if !is_cd {
            break;
        }
    }
    if tokens.is_empty() {
        // A command that did not parse still has to land somewhere, or the
        // table would quietly under-report the traffic it cannot read.
        let first_line = command.trim().lines().next().unwrap_or_default();
        tokens = crate::policy::strip_wrappers(
            &first_line
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>(),
            policy,
        );
    }
    let Some(head) = tokens.first() else {
        return "(unparsed)".to_owned();
    };
    let name = head.rsplit('/').next().unwrap_or(head);
    let second = tokens
        .get(1)
        .filter(|word| !word.starts_with('-'))
        .map(String::as_str);

    // `tool-ns args` and `tool ns args` are the same dispatch, so both label
    // from the rule name.
    if let Some((rule, namespace)) = policy.aliased(name) {
        return match second {
            Some(word) => format!("{} {namespace} {word}", rule.name),
            None => format!("{} {namespace}", rule.name),
        };
    }
    let grouped =
        GROUPED.contains(&name) || policy.rule(name).and_then(Rule::alias_prefix).is_some();
    match second {
        Some(word) if grouped => format!("{name} {word}"),
        _ => name.to_owned(),
    }
}

/// Insertion order breaks ties, so equal counts keep a stable order between runs.
fn ranked(counts: &HashMap<String, (usize, usize)>, limit: usize) -> Vec<(String, usize, usize)> {
    let mut rows: Vec<(String, usize, usize, usize)> = counts
        .iter()
        .map(|(name, (count, first_seen))| (name.clone(), *count, *first_seen, 0))
        .collect();
    rows.sort_by(|left, right| right.1.cmp(&left.1).then(left.2.cmp(&right.2)));
    rows.into_iter()
        .take(limit)
        .map(|(name, count, _, extra)| (name, count, extra))
        .collect()
}

fn print_toon(table: &str, header: &str, rows: &[Vec<String>]) {
    println!("{table}[{}]{{{header}}}:", rows.len());
    for row in rows {
        let cells: Vec<String> = row
            .iter()
            .map(|cell| {
                if cell.contains(',') {
                    format!("\"{cell}\"")
                } else {
                    cell.clone()
                }
            })
            .collect();
        println!("  {}", cells.join(","));
    }
}

fn percent(part: usize, total: usize) -> String {
    if total == 0 {
        return "n/a".to_owned();
    }
    format!("{:.0}%", 100.0 * part as f64 / total as f64)
}

pub fn audit(workspace: &Path, windows: &[String], policy: &Policy) -> i32 {
    let dir = transcript_dir(workspace);
    if !dir.is_dir() {
        eprintln!("no transcripts under {}", dir.display());
        return 1;
    }

    let mut widest: Vec<Call> = Vec::new();
    let mut rows = Vec::new();
    for window in windows {
        let files = files_in_window(&dir, window);
        let calls: Vec<Call> = files
            .iter()
            .flat_map(|path| bash_calls(path, policy))
            .collect();
        let allowed = calls.iter().filter(|call| call.allowed).count();
        let prompting = calls
            .iter()
            .filter(|call| !call.allowed && call.sandbox_off)
            .count();
        rows.push(vec![
            window.clone(),
            files.len().to_string(),
            calls.len().to_string(),
            allowed.to_string(),
            (calls.len() - allowed).to_string(),
            prompting.to_string(),
            percent(allowed, calls.len()),
        ]);
        if calls.len() >= widest.len() {
            widest = calls;
        }
    }
    print_toon(
        "windows",
        "window,sessions,calls,allow,passthrough,prompting,allow_pct",
        &rows,
    );

    let mut allow_counts: HashMap<String, (usize, usize)> = HashMap::new();
    let mut pass_counts: HashMap<String, (usize, usize)> = HashMap::new();
    let mut sandbox_off: HashMap<String, usize> = HashMap::new();
    for (order, call) in widest.iter().enumerate() {
        let target = if call.allowed {
            &mut allow_counts
        } else {
            &mut pass_counts
        };
        let entry = target.entry(call.prefix.clone()).or_insert((0, order));
        entry.0 += 1;
        if !call.allowed && call.sandbox_off {
            *sandbox_off.entry(call.prefix.clone()).or_insert(0) += 1;
        }
    }

    println!();
    print_toon(
        "totals",
        "bucket,calls,distinct_prefixes",
        &[
            vec![
                "allow".to_owned(),
                widest
                    .iter()
                    .filter(|call| call.allowed)
                    .count()
                    .to_string(),
                allow_counts.len().to_string(),
            ],
            vec![
                "passthrough".to_owned(),
                widest
                    .iter()
                    .filter(|call| !call.allowed)
                    .count()
                    .to_string(),
                pass_counts.len().to_string(),
            ],
        ],
    );

    println!();
    print_toon(
        "passthrough_top",
        "prefix,calls,sandbox_off",
        &ranked(&pass_counts, 20)
            .into_iter()
            .map(|(name, count, _)| {
                let off = sandbox_off.get(&name).copied().unwrap_or(0);
                vec![name, count.to_string(), off.to_string()]
            })
            .collect::<Vec<_>>(),
    );

    println!();
    print_toon(
        "allow_top",
        "prefix,calls",
        &ranked(&allow_counts, 20)
            .into_iter()
            .map(|(name, count, _)| vec![name, count.to_string()])
            .collect::<Vec<_>>(),
    );
    0
}

pub fn windows_from(args: &[String]) -> Vec<String> {
    if args.iter().any(|arg| arg == "--sweep") {
        return WINDOWS.iter().map(|window| (*window).to_owned()).collect();
    }
    let named: Vec<String> = args
        .iter()
        .filter(|arg| !arg.starts_with('-') && window_days(arg).is_some())
        .cloned()
        .collect();
    if named.is_empty() {
        vec!["today".to_owned()]
    } else {
        named
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::prefix_of;
    use crate::policy::Policy;
    use crate::policy::load::{Report, merge};

    /// A name no baseline rule uses, so this pins the mechanism, not the data.
    const OVERLAY: &str = r#"
version = 1

[[command]]
name = "tool"
kind = "paths"
alias_prefix = "tool-"
help_tail = true
paths = ["ns verb"]
"#;

    fn overlay() -> Policy {
        let mut policy = Policy::empty();
        let mut report = Report::default();
        merge(&mut policy, &mut report, Path::new("test.toml"), OVERLAY);
        assert!(report.problems.is_empty(), "{:?}", report.problems);
        policy
    }

    /// Both the alias label and the grouping come from the rule's own
    /// `alias_prefix`, so no tool name has to be compiled in.
    #[test]
    fn an_alias_prefix_rule_groups_by_namespace() {
        let policy = overlay();
        for (command, label) in [
            ("tool ns verb", "tool ns"),
            ("tool-ns verb x", "tool ns verb"),
            ("./bin/tool-ns verb -w", "tool ns verb"),
            ("timeout 90 tool ns verb", "tool ns"),
            ("cd x && tool ns verb", "tool ns"),
            ("tool", "tool"),
            ("tool --help", "tool"),
        ] {
            assert_eq!(prefix_of(command, &policy), label, "{command}");
        }
    }

    /// The label must not survive the policy going away: this is what fails if
    /// someone hardcodes a tool name back into `GROUPED`.
    #[test]
    fn grouping_a_namespace_needs_the_rule_that_declares_it() {
        let bare = Policy::empty();
        assert_eq!(prefix_of("tool ns verb", &bare), "tool");
        assert_eq!(prefix_of("tool-ns verb", &bare), "tool-ns");
    }
}
