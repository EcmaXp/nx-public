//! Check a policy, loudly.
//!
//! `nix build` covers the baseline, but nothing covers a freshly edited overlay,
//! because the overlay needs no rebuild. This is what closes that gap: edit,
//! validate, done, in seconds and without a build.
//!
//! It is also the only place a discarded row is ever mentioned. The hook stays
//! silent about one on purpose, since discarding can only add prompts, but
//! silence is a poor way to learn you mistyped a table.

use crate::policy::Policy;
use crate::policy::load::{self, Report};

/// A case file: one `command<TAB>allow|prompt` per line, `#` for a comment.
fn run_cases(path: &str, policy: &Policy) -> Result<(usize, Vec<String>), std::io::Error> {
    let text = std::fs::read_to_string(path)?;
    let mut checked = 0;
    let mut failures = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (command, expected) = match line.rsplit_once('\t') {
            Some(parts) => parts,
            None => {
                failures.push(format!("{line}: no tab separating the expectation"));
                continue;
            }
        };
        let want_allow = match expected.trim() {
            "allow" => true,
            "prompt" => false,
            other => {
                failures.push(format!("{command}: unknown expectation {other}"));
                continue;
            }
        };
        checked += 1;
        let got = crate::classify(command, policy);
        if got.is_some() != want_allow {
            failures.push(format!(
                "{command}: want {expected}, got {}",
                got.unwrap_or_else(|| "prompt".to_owned())
            ));
        }
    }
    Ok((checked, failures))
}

pub fn validate(policy: &Policy, report: &Report, cases: Option<&str>) -> i32 {
    let source = match report.source {
        load::Source::Manifest => "manifest",
        load::Source::Baseline => "baseline fallback, no manifest",
    };
    println!("files: {} ({source})", report.files.len());
    for path in &report.files {
        println!("  {}", path.display());
    }
    println!("rules: {}", report.rules);
    println!("readers: {}", policy.readers.len());

    let mut code = 0;
    if report.problems.is_empty() {
        println!("discarded: none");
    } else {
        println!("discarded: {}", report.problems.len());
        for problem in &report.problems {
            println!("  {problem}");
        }
        code = 1;
    }

    if let Some(path) = cases {
        match run_cases(path, policy) {
            Err(error) => {
                eprintln!("{path}: {error}");
                code = 1;
            }
            Ok((checked, failures)) => {
                println!("cases: {checked} checked, {} failed", failures.len());
                for failure in &failures {
                    println!("  {failure}");
                }
                if !failures.is_empty() {
                    code = 1;
                }
            }
        }
    }
    code
}

/// Load and report in one step, for the CLI.
pub fn run(explicit: &[String], cases: Option<&str>) -> i32 {
    let (policy, report) = load::load(explicit);
    validate(&policy, &report, cases)
}
