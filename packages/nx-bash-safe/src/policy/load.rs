//! Loading policy, and the trust tiers that make loading it safe.
//!
//! Three sources, in order: a **baseline** shipped read-only beside the binary,
//! an **overlay** at a fixed user-level path, and **explicit** files named on
//! the command line. Two invariants make a writable policy file no worse than
//! the permission rules that already live in a writable settings file:
//!
//! - **Additive only.** An overlay may introduce a command the baseline does
//!   not define. A name collision is rejected rather than merged, because
//!   merging would let an overlay redefine `git` as a reader and neuter the
//!   classifier.
//! - **Monotone.** Dropping any file, row, or field can only produce more
//!   prompts, never fewer. That is what makes "discard what does not parse"
//!   identical to failing closed, and it is why the hook can stay silent about
//!   a malformed overlay instead of refusing to run.
//!
//! There is deliberately **no ambient discovery**: nothing is loaded relative
//! to the project directory, so opening a hostile repository cannot hand it a
//! rule that auto-allows its own commands.

use std::path::{Path, PathBuf};

use toml::Value;

use crate::flags::FlagTable;
use crate::policy::{Kind, Policy, Positional, Rule, VerbSpec};

/// Baked in by the packaging so the baseline can be found in the store.
const BAKED_POLICY_DIR: Option<&str> = option_env!("NX_BASH_SAFE_POLICY_DIR");

/// The layer order, declared by the `home.layers` nix option: one absolute
/// directory per line, lowest precedence first.
const MANIFEST_SUBPATH: &str = ".nx/manifest/layers/bash-safe";

/// Caps, so a runaway or hostile file cannot turn the hook into a hang.
const MAX_FILES: usize = 16;
const MAX_BYTES: u64 = 256 * 1024;
const MAX_RULES: usize = 512;

/// Where the list of directories came from. Worth reporting: an unexpected
/// `Baseline` is how "the workspace layer is missing" looks from the outside,
/// and that failure is otherwise indistinguishable from a policy that simply
/// never had those rules.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Manifest,
    #[default]
    Baseline,
}

#[derive(Debug, Default)]
pub struct Report {
    pub files: Vec<PathBuf>,
    pub rules: usize,
    pub source: Source,
    /// Everything discarded, with a reason. Silent in hook mode, printed by
    /// `validate`, which is the only way to see an overlay typo.
    pub problems: Vec<String>,
}

/// Keep the first occurrence of each file, comparing resolved paths.
///
/// Naming the same directory twice is not hypothetical: it is what `--policy`
/// against the baseline does while comparing two policies. Without this, every
/// rule in the second copy is reported as a collision and `validate` fails on a
/// perfectly good policy.
fn dedup(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen: Vec<PathBuf> = Vec::with_capacity(paths.len());
    let mut out = Vec::with_capacity(paths.len());
    for path in paths {
        let key = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        out.push(path);
    }
    out
}

/// Where the baseline policy lives: next to the binary, or wherever packaging
/// put it.
fn baseline_dir() -> Option<PathBuf> {
    // Existence is checked, not assumed: during a packaged build the tests run
    // before the install step that creates this path, and a baked path that is
    // not there yet must fall through rather than silently disable the policy.
    if let Some(dir) = BAKED_POLICY_DIR
        .map(PathBuf::from)
        .filter(|dir| dir.is_dir())
    {
        return Some(dir);
    }
    let exe = std::env::current_exe().ok()?;
    let root = exe.parent()?.parent()?;
    let shipped = root.join("share/nx-bash-safe/policy");
    if shipped.is_dir() {
        return Some(shipped);
    }
    // Running from a checkout.
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("policy");
    source.is_dir().then_some(source)
}

/// The declared layers, in load order.
///
/// `None` means there is no manifest at all, which is what a packaged build and
/// a checkout that has not been switched both look like. That is why
/// `baseline_dir` still exists: without it, the tests that run before the
/// install step would classify against an empty policy.
fn manifest_dirs() -> Option<Vec<PathBuf>> {
    let path = match std::env::var_os("NX_BASH_SAFE_LAYERS") {
        Some(value) => PathBuf::from(value),
        None => PathBuf::from(std::env::var_os("HOME")?).join(MANIFEST_SUBPATH),
    };
    let text = std::fs::read_to_string(&path).ok()?;
    Some(
        text.lines()
            .map(str::trim)
            .filter(|line| line.starts_with('/'))
            .map(PathBuf::from)
            .filter(|dir| dir.is_dir())
            .collect(),
    )
}

fn toml_files(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_owned()];
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
        .collect();
    files.sort();
    files
}

/// Load the baseline, then the overlay, then anything named explicitly.
pub fn load(explicit: &[String]) -> (Policy, Report) {
    let (mut sources, from_manifest) = match manifest_dirs() {
        Some(dirs) if !dirs.is_empty() => (dirs, true),
        _ => (baseline_dir().into_iter().collect(), false),
    };
    if let Some(from_env) = std::env::var_os("NX_BASH_SAFE_POLICY") {
        sources.extend(
            from_env
                .to_string_lossy()
                .split(':')
                .filter(|part| !part.is_empty())
                .map(PathBuf::from),
        );
    }
    sources.extend(explicit.iter().map(PathBuf::from));

    let mut policy = Policy::empty();
    let mut report = Report::default();
    report.source = if from_manifest {
        Source::Manifest
    } else {
        Source::Baseline
    };
    let mut files = dedup(sources.iter().flat_map(|path| toml_files(path)).collect());
    if files.len() > MAX_FILES {
        // Truncating in silence would look exactly like a policy that never
        // mentioned those commands, so say it rather than imply it.
        report.problems.push(format!(
            "{} policy files, more than {MAX_FILES}; the rest were ignored",
            files.len()
        ));
        files.truncate(MAX_FILES);
    }

    for path in files {
        match std::fs::metadata(&path) {
            Ok(meta) if meta.len() > MAX_BYTES => {
                report
                    .problems
                    .push(format!("{}: larger than {MAX_BYTES} bytes", path.display()));
                continue;
            }
            Ok(_) => {}
            Err(error) => {
                report.problems.push(format!("{}: {error}", path.display()));
                continue;
            }
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            report
                .problems
                .push(format!("{}: unreadable", path.display()));
            continue;
        };
        merge(&mut policy, &mut report, &path, &text);
        report.files.push(path);
    }
    report.rules = policy.rules.len();
    (policy, report)
}

/// Parse one document into the policy, recording whatever it had to discard.
pub fn merge(policy: &mut Policy, report: &mut Report, path: &Path, text: &str) {
    let name = path.display().to_string();
    let document: Value = match toml::from_str(text) {
        Ok(value) => value,
        Err(error) => {
            report.problems.push(format!("{name}: {error}"));
            return;
        }
    };

    for (key, target) in [
        ("readers", &mut policy.readers),
        ("reader_builtins", &mut policy.reader_builtins),
        ("help_flags", &mut policy.help_flags),
        ("plan_mode_block", &mut policy.plan_mode_block),
        ("safe_assign_vars", &mut policy.safe_assign_vars),
        ("shell_critical_vars", &mut policy.shell_critical_vars),
        ("sandbox_off_ok", &mut policy.sandbox_off_ok),
    ] {
        for item in strings(document.get(key)) {
            if !target.contains(&item) {
                target.push(item);
            }
        }
    }

    let Some(commands) = document.get("command").and_then(Value::as_array) else {
        return;
    };
    for entry in commands {
        if policy.rules.len() >= MAX_RULES {
            report
                .problems
                .push(format!("{name}: more than {MAX_RULES} rules"));
            return;
        }
        match rule_from(entry) {
            Err(problem) => report.problems.push(format!("{name}: {problem}")),
            Ok(rule) => {
                if policy
                    .rules
                    .iter()
                    .any(|existing| existing.name == rule.name)
                {
                    // Additive only: shadowing a compiled-in command is how an
                    // overlay would turn a flag-validated tool into a reader.
                    report
                        .problems
                        .push(format!("{name}: {} is already defined", rule.name));
                    continue;
                }
                policy.rules.push(rule);
            }
        }
    }
}

fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn words(value: Option<&Value>) -> Vec<Vec<String>> {
    strings(value)
        .into_iter()
        .map(|path| path.split_whitespace().map(str::to_owned).collect())
        .filter(|path: &Vec<String>| !path.is_empty())
        .collect()
}

fn flag_tables(entry: &Value) -> Vec<(Vec<String>, FlagTable)> {
    entry
        .get("tables")
        .and_then(Value::as_table)
        .map(|table| {
            table
                .iter()
                .filter_map(|(key, spec)| {
                    let path: Vec<String> = key.split_whitespace().map(str::to_owned).collect();
                    let spec = spec.as_str()?;
                    (!path.is_empty()).then(|| (path, FlagTable::parse(spec)))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn boolean(entry: &Value, key: &str) -> bool {
    entry.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn number(entry: &Value, key: &str) -> usize {
    entry
        .get(key)
        .and_then(Value::as_integer)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(0)
}

fn rule_from(entry: &Value) -> Result<Rule, String> {
    let name = entry
        .get("name")
        .and_then(Value::as_str)
        .ok_or("a command has no name")?
        .to_owned();
    let kind_name = entry
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{name} has no kind"))?;
    let table = FlagTable::parse(entry.get("table").and_then(Value::as_str).unwrap_or(""));

    let kind = match kind_name {
        "reader" => Kind::Reader,
        "flags" => Kind::Flags {
            table,
            positionals: match entry.get("positionals").and_then(Value::as_str) {
                None | Some("any") => Positional::Any,
                Some("plus_prefixed") => Positional::PlusPrefixed,
                Some("at_least_one") => Positional::AtLeastOne,
                Some(other) => return Err(format!("{name}: unknown positionals {other}")),
            },
        },
        "paths" => Kind::Paths {
            paths: words(entry.get("paths")),
            skip_flags: boolean(entry, "skip_flags"),
            skip_value_flags: strings(entry.get("skip_value_flags")),
            help_tail: boolean(entry, "help_tail"),
            alias_prefix: entry
                .get("alias_prefix")
                .and_then(Value::as_str)
                .map(str::to_owned),
            matchers: strings(entry.get("matchers")),
        },
        "path_flags" => Kind::PathFlags {
            tables: flag_tables(entry),
            help_anywhere: boolean(entry, "help_anywhere"),
        },
        "verbs" => Kind::Verbs(VerbSpec {
            verbs: strings(entry.get("verbs")),
            trailing: entry.get("verb_position").and_then(Value::as_str) == Some("trailing"),
            min_words: number(entry, "min_words"),
            max_words: number(entry, "max_words"),
            bool_flags: strings(entry.get("bool_flags")),
            value_flags: strings(entry.get("value_flags")),
            help_flags: boolean(entry, "help_flags"),
            allow_paths: strings(entry.get("allow_paths")),
            matchers: strings(entry.get("matchers")),
        }),
        "native" => Kind::Native {
            matcher: entry
                .get("matcher")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("{name} is native with no matcher"))?
                .to_owned(),
            table,
            tables: flag_tables(entry),
            deny: strings(entry.get("deny")),
            bool_flags: strings(entry.get("bool_flags")),
            paths: strings(entry.get("paths")),
        },
        other => return Err(format!("{name}: unknown kind {other}")),
    };
    Ok(Rule { name, kind })
}
