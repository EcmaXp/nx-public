//! Policy: the data half of the classifier.
//!
//! The engine knows *kinds* of rule, not tools. A tool is a row of data naming
//! its kind, its flag table, and its allowed subcommands, so adding one is an
//! edit to a TOML file. The handful of checks that genuinely cannot be data (a
//! sed script's grammar, `gh api` being GET-only) stay compiled and are
//! selected *by name* from the data, so a policy file contains no logic.
//!
//! The dispatch order in `classify_segment` is behavior, not tidiness. It is
//! written out flat, in the original's order, because three of its steps are
//! load-bearing in ways a cleaner arrangement would silently change.

pub mod load;
pub mod natives;

use crate::flags::{FlagTable, parse_flags};
use crate::pattern;

/// What a rule's positional arguments are allowed to look like.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Positional {
    #[default]
    Any,
    /// Only `+FORMAT`, so `date` cannot be used to set the clock.
    PlusPrefixed,
    /// At least one, so an interactive REPL cannot be entered.
    AtLeastOne,
}

#[derive(Debug, Default, Clone)]
pub struct VerbSpec {
    pub verbs: Vec<String>,
    pub trailing: bool,
    pub min_words: usize,
    pub max_words: usize,
    pub bool_flags: Vec<String>,
    pub value_flags: Vec<String>,
    pub help_flags: bool,
    pub allow_paths: Vec<String>,
    pub matchers: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Kind {
    /// Arguments are data. An opaque expansion may reach these, because there
    /// is no flag table for one to smuggle a flag past.
    Reader,
    Flags {
        table: FlagTable,
        positionals: Positional,
    },
    /// The leading words must match an allowed subcommand path.
    Paths {
        paths: Vec<Vec<String>>,
        skip_flags: bool,
        /// Global flags that consume the next token (`--context prod get`).
        skip_value_flags: Vec<String>,
        help_tail: bool,
        alias_prefix: Option<String>,
        matchers: Vec<String>,
    },
    /// Subcommand path, then that path's own flag table.
    PathFlags {
        tables: Vec<(Vec<String>, FlagTable)>,
        help_anywhere: bool,
    },
    Verbs(VerbSpec),
    /// A compiled check, named from the data.
    Native {
        matcher: String,
        table: FlagTable,
        tables: Vec<(Vec<String>, FlagTable)>,
        deny: Vec<String>,
        bool_flags: Vec<String>,
        paths: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub kind: Kind,
}

impl Rule {
    pub(crate) fn alias_prefix(&self) -> Option<&str> {
        match &self.kind {
            Kind::Paths { alias_prefix, .. } => alias_prefix.as_deref(),
            _ => None,
        }
    }

    fn runs_before_path_rejection(&self) -> bool {
        matches!(
            &self.kind,
            Kind::Paths {
                help_tail: true,
                ..
            }
        )
    }
}

/// Everything the classifier is allowed to say yes to.
#[derive(Debug, Default)]
pub struct Policy {
    pub readers: Vec<String>,
    pub reader_builtins: Vec<String>,
    pub help_flags: Vec<String>,
    pub plan_mode_block: Vec<String>,
    pub safe_assign_vars: Vec<String>,
    pub shell_critical_vars: Vec<String>,
    /// Reason segments entitled to keep their allow under sandbox-off.
    pub sandbox_off_ok: Vec<String>,
    pub rules: Vec<Rule>,
}

impl Policy {
    /// An empty policy allows nothing, which is the safe starting point: with
    /// no rules loaded, every command prompts.
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn blocks_in_plan_mode(&self, reason_segment: &str) -> bool {
        contains(&self.plan_mode_block, reason_segment)
    }

    /// Whether an opaque expansion may reach this command as data.
    pub fn is_simple_reader(&self, name: &str) -> bool {
        contains(&self.readers, name) || contains(&self.reader_builtins, name)
    }

    pub fn is_help_flag(&self, token: &str) -> bool {
        contains(&self.help_flags, token)
    }

    pub fn is_safe_assign(&self, name: &str) -> bool {
        contains(&self.safe_assign_vars, name)
    }

    pub fn is_shell_critical(&self, name: &str) -> bool {
        contains(&self.shell_critical_vars, name)
    }

    pub fn sandbox_off_reason_ok(&self, segment: &str) -> bool {
        self.sandbox_off_ok.iter().any(|entry| {
            segment == entry
                || (segment.starts_with(entry.as_str())
                    && segment.as_bytes().get(entry.len()) == Some(&b' '))
        })
    }

    pub fn rule(&self, name: &str) -> Option<&Rule> {
        self.rules.iter().find(|rule| rule.name == name)
    }

    /// A rule reached through its alias form, e.g. `tool-ns` for `tool`, with
    /// the suffix put back as the first word.
    pub(crate) fn aliased(&self, name: &str) -> Option<(&Rule, String)> {
        self.rules.iter().find_map(|rule| {
            let prefix = rule.alias_prefix()?;
            let suffix = name.strip_prefix(prefix)?;
            (!suffix.is_empty()).then(|| (rule, suffix.to_owned()))
        })
    }
}

fn contains(haystack: &[String], needle: &str) -> bool {
    haystack.iter().any(|item| item == needle)
}

fn basename(token: &str) -> &str {
    token.rsplit('/').next().unwrap_or(token)
}

/// Sentinels that no rule name can equal, so they classify as nothing.
const TIMEOUT_MALFORMED: &str = "\u{0}timeout\u{0}";
const ENV_UNSAFE: &str = "\u{0}env-unsafe\u{0}";

/// Strip leading assignments and `env`/`timeout`-style wrappers.
///
/// Returns the command the segment actually runs, or a sentinel when the
/// wrapper itself is the problem: an exported assignment to a name outside the
/// allowlist can turn a read-only command into an exec vector, so it has to
/// poison the segment rather than be skipped.
pub fn strip_wrappers(tokens: &[String], policy: &Policy) -> Vec<String> {
    let mut assigned: Vec<&str> = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        let token = tokens[index].as_str();
        if pattern::is_assignment(token) {
            assigned.push(token.split_once('=').map_or(token, |(name, _)| name));
            index += 1;
        } else if matches!(token, "env" | "time" | "nice") {
            index += 1;
        } else if token == "command" {
            // `command -v name` is a lookup for the `command` rule, not a wrapper.
            if tokens
                .get(index + 1)
                .is_some_and(|next| next.starts_with('-'))
            {
                break;
            }
            index += 1;
        } else if token == "direnv"
            && tokens.get(index + 1).is_some_and(|next| next == "exec")
            && tokens.get(index + 2).is_some_and(|dir| {
                !dir.is_empty() && !dir.starts_with('-') && *dir != crate::parse::OPAQUE
            })
        {
            index += 3;
        } else if token == "rtk" {
            index += 1;
            if tokens.get(index).is_some_and(|next| next == "proxy") {
                index += 1;
            }
        } else if token == "timeout" {
            index += 1;
            while index < tokens.len() {
                let token = tokens[index].as_str();
                if matches!(token, "-k" | "--kill-after" | "-s" | "--signal") {
                    index += 2;
                } else if token.starts_with('-') {
                    index += 1;
                } else if pattern::is_timeout_duration(token) {
                    index += 1;
                    break;
                } else {
                    return vec![TIMEOUT_MALFORMED.to_owned()];
                }
            }
        } else {
            break;
        }
    }

    let rest: Vec<String> = tokens[index.min(tokens.len())..].to_vec();
    if !rest.is_empty() {
        // Prefix assignments are exported into the child's environment.
        if assigned.iter().any(|name| !policy.is_safe_assign(name)) {
            return vec![ENV_UNSAFE.to_owned()];
        }
    } else if assigned.iter().any(|name| policy.is_shell_critical(name)) {
        return vec![ENV_UNSAFE.to_owned()];
    }
    rest
}

/// The reason this segment is safe, or `None` to prompt.
pub fn classify_segment(tokens: &[String], policy: &Policy) -> Option<String> {
    let rest = strip_wrappers(tokens, policy);
    let Some(head) = rest.first() else {
        // A bare wrapper with nothing to run is just an environment change.
        return (!tokens.is_empty()).then(|| "env".to_owned());
    };
    let args = &rest[1..];

    if head == "cd" || head == "unset" {
        return Some(head.clone());
    }
    if head == "export" {
        return classify_export(args, policy);
    }

    let name = basename(head);

    if name == "xargs" {
        return classify_xargs(args, policy);
    }

    // Before the path rejection below, because these are invoked by path in
    // real traffic (`./bin/tool-ns review`).
    if let Some(rule) = policy
        .rule(name)
        .filter(|rule| rule.runs_before_path_rejection())
    {
        return classify_rule(rule, name, args, policy);
    }
    if let Some((rule, suffix)) = policy.aliased(name) {
        let mut words = vec![suffix];
        words.extend(args.iter().cloned());
        return classify_rule(rule, &rule.name.clone(), &words, policy);
    }

    // A trailing help flag is a read for any command, known or not.
    if rest.len() == 2 && policy.is_help_flag(&rest[1]) {
        return Some(format!("{name} {}", rest[1]));
    }
    if name == "uv"
        && args.first().is_some_and(|word| word == "run")
        && args.last().is_some_and(|word| policy.is_help_flag(word))
    {
        return Some("uv run help".to_owned());
    }

    // Anything else invoked by path is refused: the name is not enough to know
    // what the binary is.
    if head.contains('/') {
        return None;
    }

    if contains(&policy.readers, name) {
        return Some(name.to_owned());
    }
    let rule = policy.rule(name)?;
    classify_rule(rule, name, args, policy)
}

/// xargs appends data-derived arguments at runtime: simple readers only.
fn classify_xargs(args: &[String], policy: &Policy) -> Option<String> {
    let mut index = 0;
    while index < args.len() {
        let token = args[index].as_str();
        if !token.starts_with('-') {
            break;
        }
        match token.split_once('=').map_or(token, |(head, _)| head) {
            "-0" | "--null" | "-r" | "--no-run-if-empty" | "-t" | "--verbose" | "-x" | "--exit" => {
                index += 1
            }
            "-n" | "--max-args" | "-I" | "-i" | "--replace" | "-P" | "--max-procs" | "-d"
            | "--delimiter" | "-L" | "--max-lines" | "-s" | "--max-chars" => {
                index += if token.contains('=') { 1 } else { 2 };
            }
            _ => return None,
        }
    }
    let inner = &args[index.min(args.len())..];
    let Some(head) = inner.first() else {
        return Some("xargs".to_owned()); // the default command is echo
    };
    if !policy.is_simple_reader(basename(head)) {
        return None;
    }
    classify_segment(inner, policy).map(|reason| format!("xargs {reason}"))
}

fn classify_export(args: &[String], policy: &Policy) -> Option<String> {
    for arg in args {
        let name = arg.split_once('=').map_or(arg.as_str(), |(name, _)| name);
        if arg.starts_with('-') || !policy.is_safe_assign(name) {
            return None;
        }
    }
    Some("export".to_owned())
}

fn classify_rule(rule: &Rule, name: &str, args: &[String], policy: &Policy) -> Option<String> {
    match &rule.kind {
        Kind::Reader => Some(name.to_owned()),
        Kind::Flags { table, positionals } => {
            let parsed = parse_flags(args, table, false)?;
            let ok = match positionals {
                Positional::Any => true,
                Positional::PlusPrefixed => {
                    parsed.positionals.iter().all(|word| word.starts_with('+'))
                }
                Positional::AtLeastOne => !parsed.positionals.is_empty(),
            };
            ok.then(|| name.to_owned())
        }
        Kind::Paths {
            paths,
            skip_flags,
            skip_value_flags,
            help_tail,
            matchers,
            ..
        } => classify_paths(
            name,
            args,
            paths,
            *skip_flags,
            skip_value_flags,
            *help_tail,
            matchers,
            policy,
        ),
        Kind::PathFlags {
            tables,
            help_anywhere,
        } => {
            if *help_anywhere && args.iter().any(|arg| policy.is_help_flag(arg)) {
                return Some(format!("{name} help"));
            }
            let (path, table) = longest_table_match(tables, args)?;
            parse_flags(&args[path.len()..], table, false)?;
            Some(format!("{name} {}", path.join(" ")))
        }
        Kind::Verbs(spec) => natives::classify_verbs(name, args, spec, policy),
        Kind::Native {
            matcher,
            table,
            tables,
            deny,
            bool_flags,
            paths,
        } => natives::run(
            matcher, name, args, table, tables, deny, bool_flags, paths, policy,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn classify_paths(
    name: &str,
    args: &[String],
    paths: &[Vec<String>],
    skip_flags: bool,
    skip_value_flags: &[String],
    help_tail: bool,
    matchers: &[String],
    policy: &Policy,
) -> Option<String> {
    // A help flag on any subcommand, even one that does not exist. It needs a
    // subcommand though: `tool --help` alone has no namespace to name, and the
    // original reached that case with an empty argument list and fell through.
    if help_tail && args.len() >= 2 && args.last().is_some_and(|last| policy.is_help_flag(last)) {
        return args.first().map(|first| format!("{name} {first} help"));
    }
    let words: Vec<String> = if skip_flags {
        // The subcommand may sit behind global flags (`terraform -chdir=x show`).
        let mut words = Vec::new();
        let mut index = 0;
        while index < args.len() && words.is_empty() {
            let arg = args[index].as_str();
            if !arg.starts_with('-') {
                words.push(arg.to_owned());
                continue;
            }
            let base = arg.split_once('=').map_or(arg, |(head, _)| head);
            index += if !arg.contains('=') && contains(skip_value_flags, base) {
                2
            } else {
                1
            };
        }
        words
    } else {
        args.to_vec()
    };
    if let Some(path) = longest_path_match(paths, &words) {
        return Some(format!("{name} {}", path.join(" ")));
    }
    for matcher in matchers {
        if let Some(reason) = natives::run_path_matcher(matcher, name, args) {
            return Some(reason);
        }
    }
    None
}

/// Longest match wins, so `stash list` is not shadowed by a bare `stash`.
fn longest_path_match<'a>(paths: &'a [Vec<String>], words: &[String]) -> Option<&'a Vec<String>> {
    paths
        .iter()
        .filter(|path| words.len() >= path.len() && words[..path.len()] == path[..])
        .max_by_key(|path| path.len())
}

fn longest_table_match<'a>(
    tables: &'a [(Vec<String>, FlagTable)],
    words: &[String],
) -> Option<(&'a Vec<String>, &'a FlagTable)> {
    tables
        .iter()
        .filter(|(path, _)| words.len() >= path.len() && words[..path.len()] == path[..])
        .max_by_key(|(path, _)| path.len())
        .map(|(path, table)| (path, table))
}
