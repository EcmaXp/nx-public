//! The checks that cannot be data, selected by name from the data.
//!
//! Each one exists because the tool's read surface is not "a name plus a flag
//! table": `git`'s key depends on its subcommand's own subcommand, `gh api` is
//! safe only for GET, a `sed` script is a small language. Keeping them here and
//! naming them from a policy file is what lets the file stay declarative: an
//! unknown matcher name drops the rule, which narrows the allow surface.

use crate::env;
use crate::flags::{FlagTable, parse_flags};
use crate::pattern;
use crate::policy::{Policy, VerbSpec};

type Tables = [(Vec<String>, FlagTable)];

#[allow(clippy::too_many_arguments)]
pub fn run(
    matcher: &str,
    name: &str,
    args: &[String],
    table: &FlagTable,
    tables: &Tables,
    deny: &[String],
    bool_flags: &[String],
    paths: &[String],
    policy: &Policy,
) -> Option<String> {
    match matcher {
        "git" => git(args, tables),
        "gh" => gh(args, tables),
        "aws" => aws(args, deny, bool_flags),
        "sed" => sed(args, table),
        "jq" => jq(args),
        "ps" => ps(args, table),
        "find" => find(args, deny),
        "yq" => yq(args),
        "treefmt" => treefmt(args),
        "py_compile" => py_compile(args),
        "awk" => awk(args, table),
        "curl" => curl(args, table),
        "read_builtin" => read_builtin(args, table, policy),
        "uv_run_script" => uv_run_script(args, paths),
        // An unknown matcher drops the rule, which can only add prompts.
        _ => {
            let _ = (name, policy);
            None
        }
    }
}

/// Matchers a `paths` rule can fall back to when no path matched.
pub fn run_path_matcher(matcher: &str, name: &str, args: &[String]) -> Option<String> {
    match matcher {
        "asana_read_tool" => {
            let [namespace, verb, tool, ..] = args else {
                return None;
            };
            (namespace == "asana" && verb == "call" && pattern::is_asana_read_tool(tool))
                .then(|| format!("{name} asana call (read)"))
        }
        // Writes only .terraform/ in the cwd; any other init touches a backend.
        "terraform_init_backend_false" => {
            let backendless = args.first().is_some_and(|first| first == "init")
                && args.iter().any(|arg| arg == "-backend=false")
                && args[1..].iter().all(|arg| {
                    matches!(
                        arg.as_str(),
                        "-backend=false" | "-input=false" | "-no-color"
                    )
                });
            backendless.then(|| format!("{name} init -backend=false"))
        }
        _ => None,
    }
}

fn lookup<'a>(tables: &'a Tables, key: &[&str]) -> Option<&'a FlagTable> {
    tables
        .iter()
        .find(|(path, _)| path.len() == key.len() && path.iter().zip(key).all(|(a, b)| a == b))
        .map(|(_, table)| table)
}

/// git, whose read surface is per-subcommand and whose post-conditions are not
/// expressible as flags.
fn git(args: &[String], tables: &Tables) -> Option<String> {
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            // `-C dir` is ours to allow: the worktree layout makes it the norm.
            "-C" => index += 2,
            // `-c` can exec via core.fsmonitor; sandboxed-only via decide().
            "-c" => index += 2,
            "--no-pager" | "-P" => index += 1,
            token if token.starts_with("-c") && token.len() > 2 => index += 1,
            token if token.starts_with('-') => return None,
            _ => break,
        }
    }
    let subcommand = args.get(index)?.clone();
    let mut rest = args[index + 1..].to_vec();
    let mut key = vec![subcommand.clone()];
    let mut config_bare = false;

    match subcommand.as_str() {
        "stash" => {
            let verb = rest.first()?.clone();
            if verb != "list" && verb != "show" {
                return None;
            }
            key.push(verb);
            rest.remove(0);
        }
        "worktree" => {
            if rest.first().is_none_or(|verb| verb != "list") {
                return None;
            }
            key.push("list".to_owned());
            rest.remove(0);
        }
        "remote" => {
            if let Some(verb) = rest.first().filter(|v| *v == "show" || *v == "get-url") {
                key.push(verb.clone());
                rest.remove(0);
            }
        }
        "config" => match rest.first().map(String::as_str) {
            Some("--get" | "--get-all" | "--get-regexp" | "--list" | "-l") => {
                rest.remove(0);
            }
            _ => config_bare = true,
        },
        _ => {}
    }

    let key_refs: Vec<&str> = key.iter().map(String::as_str).collect();
    let table = lookup(tables, &key_refs)?;
    let parsed = parse_flags(&rest, table, true)?;
    let positionals = &parsed.positionals;
    let key_text = key.join(" ");

    match key_text.as_str() {
        "ls-remote" | "remote" if !positionals.is_empty() => return None,
        "fetch" => {
            for word in positionals {
                // The refspec shape alone permits a URL, so exclude one here.
                if word.contains("::") || word.contains("://") || !pattern::is_fetch_ref(word) {
                    return None;
                }
            }
        }
        "remote get-url" => {
            if positionals.len() != 1 || !pattern::is_simple_name(&positionals[0]) {
                return None;
            }
        }
        "remote show" => {
            // Without -n it contacts the remote and can prompt for credentials.
            if !parsed.has("-n")
                || positionals.len() != 1
                || !pattern::is_simple_name(&positionals[0])
            {
                return None;
            }
        }
        "reflog" => {
            if positionals
                .first()
                .is_some_and(|verb| verb != "show" && verb != "list")
            {
                return None;
            }
            if positionals.iter().any(|word| {
                matches!(
                    word.as_str(),
                    "expire" | "delete" | "exists" | "drop" | "write"
                )
            }) {
                return None;
            }
        }
        "tag" | "branch"
            // A positional without --list creates or deletes.
            if !positionals.is_empty() && !parsed.has("-l") && !parsed.has("--list") => {
                return None;
            }
        _ => {}
    }
    if config_bare && positionals.len() != 1 {
        return None;
    }
    Some(format!("git {key_text}"))
}

fn gh(args: &[String], tables: &Tables) -> Option<String> {
    if args.is_empty() {
        return None;
    }
    let key: Vec<&str> = args.iter().take(2).map(String::as_str).collect();
    if let Some(table) = lookup(tables, &key) {
        parse_flags(&args[key.len().min(args.len())..], table, false)?;
        return Some(format!("gh {}", key.join(" ")));
    }
    if args[0] == "api" {
        return api_get(&args[1..]).map(|_| "gh api GET".to_owned());
    }
    None
}

/// A raw-API escape hatch is only a read for GET, and only without fields.
pub fn api_get(rest: &[String]) -> Option<()> {
    let mut index = 0;
    while index < rest.len() {
        let token = rest[index].as_str();
        if matches!(token, "-f" | "-F" | "--field" | "--raw-field" | "--input") {
            return None;
        }
        if ["-f=", "-F=", "--field=", "--raw-field=", "--input="]
            .iter()
            .any(|prefix| token.starts_with(prefix))
        {
            return None;
        }
        if matches!(token, "-X" | "--method") {
            if !rest
                .get(index + 1)
                .is_some_and(|value| value.to_uppercase() == "GET")
            {
                return None;
            }
            index += 2;
            continue;
        }
        if token.starts_with("-X") || token.starts_with("--method=") {
            let value = token
                .split_once('=')
                .map_or_else(|| token[2..].to_owned(), |(_, value)| value.to_owned());
            if value.to_uppercase() != "GET" {
                return None;
            }
        }
        index += 1;
    }
    Some(())
}

fn aws(args: &[String], deny: &[String], bool_flags: &[String]) -> Option<String> {
    // An endpoint override sends the call, and any credential in it, elsewhere.
    if args
        .iter()
        .any(|arg| arg.split_once('=').map_or(arg.as_str(), |(head, _)| head) == "--endpoint-url")
    {
        return None;
    }
    let mut words: Vec<&str> = Vec::new();
    let mut index = 0;
    while index < args.len() && words.len() < 2 {
        let token = args[index].as_str();
        if token.starts_with('-') {
            index += if token.contains('=') || bool_flags.iter().any(|flag| flag == token) {
                1
            } else {
                2
            };
            continue;
        }
        words.push(token);
        index += 1;
    }
    let [service, action] = words[..] else {
        return None;
    };
    if service == "s3" {
        return (action == "ls").then(|| "aws s3 ls".to_owned());
    }
    if service == "configure" {
        return matches!(action, "list" | "get" | "list-profiles")
            .then(|| "aws configure read".to_owned());
    }
    if deny.iter().any(|denied| denied == action) {
        return None;
    }
    const READ_VERBS: &[&str] = &[
        "get-",
        "list-",
        "describe-",
        "head-",
        "select-",
        "search-",
        "lookup-",
    ];
    READ_VERBS
        .iter()
        .any(|verb| action.starts_with(verb))
        .then(|| format!("aws {service} {action}"))
}

/// A sed script is a language: `w` writes a file, `e` executes, `r` reads one.
fn sed(args: &[String], table: &FlagTable) -> Option<String> {
    let mut scripts: Vec<String> = Vec::new();
    let mut positionals: Vec<String> = Vec::new();
    let mut index = 0;

    while index < args.len() {
        let token = args[index].as_str();
        if token == "--" {
            positionals.extend(args[index + 1..].iter().cloned());
            break;
        }
        if token.starts_with('-') && token != "-" {
            let base = token.split_once('=').map_or(token, |(head, _)| head);
            if base == "-e" || base == "--expression" {
                match token.split_once('=') {
                    Some((_, script)) => scripts.push(script.to_owned()),
                    None => {
                        if let Some(script) = args.get(index + 1) {
                            scripts.push(script.clone());
                            index += 1;
                        }
                    }
                }
            } else if let Some(takes_arg) = table.arity(base) {
                if takes_arg && !token.contains('=') {
                    index += 1;
                }
            } else if pattern::is_combined_short(token)
                && token
                    .chars()
                    .skip(1)
                    .all(|letter| table.arity(&format!("-{letter}")) == Some(false))
            {
                // a cluster of no-argument short flags
            } else {
                return None;
            }
        } else {
            positionals.push(token.to_owned());
        }
        index += 1;
    }

    if scripts.is_empty() {
        if positionals.is_empty() {
            return Some("sed".to_owned());
        }
        scripts.push(positionals.remove(0));
    }

    for script in &scripts {
        // Whole script first: a `;` may be literal inside a bracket class.
        if part_ok(script) {
            continue;
        }
        if script.split([';', '\n']).all(part_ok) {
            continue;
        }
        return None;
    }
    Some("sed read-only".to_owned())
}

fn part_ok(part: &str) -> bool {
    let part = part.trim();
    if part.is_empty() {
        return true;
    }
    let body = &part[pattern::sed_address_prefix_len(part)..];
    matches!(body, "p" | "d" | "q" | "=") || pattern::is_sed_substitution(body)
}

fn jq(args: &[String]) -> Option<String> {
    const UNSAFE_LONG: &[&str] = &[
        "--from-file",
        "--rawfile",
        "--slurpfile",
        "--run-tests",
        "--library-path",
    ];
    if pattern::has_jq_unsafe_word(&args.join(" ")) {
        return None;
    }
    for arg in args {
        let base = arg.split_once('=').map_or(arg.as_str(), |(head, _)| head);
        if UNSAFE_LONG.contains(&base) {
            return None;
        }
        if arg.starts_with('-') && !arg.starts_with("--") && pattern::is_jq_unsafe_short(arg) {
            return None;
        }
    }
    Some("jq".to_owned())
}

fn ps(args: &[String], table: &FlagTable) -> Option<String> {
    let parsed = parse_flags(args, table, false)?;
    // BSD `e` shows other processes' environments, which leaks their secrets.
    if parsed
        .positionals
        .iter()
        .any(|word| pattern::is_ps_env_cluster(word))
    {
        return None;
    }
    Some("ps".to_owned())
}

fn find(args: &[String], deny: &[String]) -> Option<String> {
    let blocked = args
        .iter()
        .any(|arg| deny.iter().any(|denied| denied == arg) || arg.starts_with("-fprint"));
    (!blocked).then(|| "find".to_owned())
}

fn yq(args: &[String]) -> Option<String> {
    let in_place = args.iter().any(|arg| arg == "-i" || arg == "--inplace");
    (!in_place).then(|| "yq".to_owned())
}

/// Rewrites files in place, so it needs explicit paths and is suppressed under
/// plan mode. The reason text carries `<paths>` because that suppression
/// matches on the reason string.
fn treefmt(args: &[String]) -> Option<String> {
    args.iter()
        .any(|arg| !arg.starts_with('-'))
        .then(|| "treefmt <paths>".to_owned())
}

/// An awk program free of system/getline/`>`/`|` is a pure filter; `>` as a
/// comparison is indistinguishable from a redirect, so only `>=` passes.
fn awk(args: &[String], _table: &FlagTable) -> Option<String> {
    // `-F,` and `-vX=Y` attach their value, which parse_flags does not model.
    let mut program = None;
    let mut index = 0;
    while index < args.len() {
        let token = args[index].as_str();
        if matches!(token, "-F" | "-v") {
            index += 2;
            continue;
        }
        if token.len() > 2 && (token.starts_with("-F") || token.starts_with("-v")) {
            index += 1;
            continue;
        }
        if token.starts_with('-') && token != "-" && token != "--" {
            return None;
        }
        if token != "--" {
            program = Some(&args[index]);
            break;
        }
        index += 1;
    }
    let program = program?;
    if program.contains("system") || program.contains("getline") {
        return None;
    }
    let bytes = program.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        let next = bytes.get(index + 1);
        let prev = index.checked_sub(1).map(|at| &bytes[at]);
        match byte {
            b'>' if next != Some(&b'=') => return None,
            b'|' if next != Some(&b'|') && prev != Some(&b'|') => return None,
            _ => {}
        }
    }
    Some("awk read-only".to_owned())
}

/// GET-only; `-o` may land in tmp, the py_compile contained-write class.
fn curl(args: &[String], table: &FlagTable) -> Option<String> {
    let mut index = 0;
    while index < args.len() {
        let token = args[index].as_str();
        if !token.starts_with('-') || token == "-" {
            index += 1;
            continue;
        }
        let base = token.split_once('=').map_or(token, |(head, _)| head);
        let inline = token.split_once('=').map(|(_, value)| value.to_owned());
        match base {
            "-X" | "--request" => {
                let value = inline.or_else(|| args.get(index + 1).cloned())?;
                if value.to_uppercase() != "GET" {
                    return None;
                }
                index += if token.contains('=') { 1 } else { 2 };
            }
            "-o" | "--output" => {
                let value = inline.or_else(|| args.get(index + 1).cloned())?;
                if value != "/dev/null" && !env::scratch_path(&value) {
                    return None;
                }
                index += if token.contains('=') { 1 } else { 2 };
            }
            _ => {
                let takes_arg = table.arity(base)?;
                index += if takes_arg && !token.contains('=') {
                    2
                } else {
                    1
                };
            }
        }
    }
    Some("curl GET".to_owned())
}

/// A shell-critical target (IFS) would let stdin data steer the line.
fn read_builtin(args: &[String], table: &FlagTable, policy: &Policy) -> Option<String> {
    let parsed = parse_flags(args, table, false)?;
    let plain = parsed.positionals.iter().all(|word| {
        !policy.is_shell_critical(word)
            && word
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    });
    plain.then(|| "read".to_owned())
}

/// `uv run` of a `paths` basename; `--with` can execute at install time.
fn uv_run_script(args: &[String], paths: &[String]) -> Option<String> {
    let mut rest = args.iter();
    if rest.next()? != "run" {
        return None;
    }
    for token in rest {
        if !token.starts_with('-') {
            let script = token.rsplit('/').next().unwrap_or(token);
            return paths
                .iter()
                .any(|allowed| allowed == script)
                .then(|| format!("uv run {script}"));
        }
        if !matches!(
            token.as_str(),
            "-q" | "--quiet" | "--no-project" | "--script" | "--frozen" | "--offline" | "--no-sync"
        ) {
            return None;
        }
    }
    None
}

fn py_compile(args: &[String]) -> Option<String> {
    let targets: &[String] = match args {
        [first, second, rest @ ..] if first == "-m" && second == "py_compile" => rest,
        [first, rest @ ..] if first == "-mpy_compile" => rest,
        _ => return None,
    };
    let files: Vec<&String> = targets.iter().filter(|arg| *arg != "-q").collect();
    // __pycache__ lands beside the source, so a tmp source keeps the write in tmp.
    (!files.is_empty() && files.iter().all(|file| env::scratch_path(file)))
        .then(|| "py_compile in tmp".to_owned())
}

/// The longest allowed path that is a prefix of these words.
fn matching_prefix(paths: &[String], words: &[&str]) -> Option<String> {
    paths
        .iter()
        .filter(|path| {
            let wanted: Vec<&str> = path.split_whitespace().collect();
            words.len() >= wanted.len() && words[..wanted.len()] == wanted[..]
        })
        .max_by_key(|path| path.len())
        .cloned()
}

/// Tools whose command path ends in a read verb.
pub fn classify_verbs(
    name: &str,
    args: &[String],
    spec: &VerbSpec,
    policy: &Policy,
) -> Option<String> {
    let mut words: Vec<&str> = Vec::new();
    let mut index = 0;

    while index < args.len() {
        let token = args[index].as_str();
        if spec.help_flags && policy.is_help_flag(token) {
            return Some(format!("{name} help"));
        }
        if token.starts_with('-') {
            let base = token.split_once('=').map_or(token, |(head, _)| head);
            if spec.bool_flags.iter().any(|flag| flag == token) {
                index += 1;
            } else if spec.value_flags.iter().any(|flag| flag == base) {
                index += if token.contains('=') { 1 } else { 2 };
            } else {
                return None;
            }
            continue;
        }
        words.push(token);

        if !spec.trailing {
            // The verb decides as soon as it appears, so an allowed path or the
            // raw-API hatch has to be recognized at the same point.
            let joined = words.join(" ");
            if spec.allow_paths.contains(&joined) {
                return Some(format!("{name} {joined}"));
            }
            if spec.matchers.iter().any(|matcher| matcher == "api_get") && joined == "api" {
                return api_get(&args[index + 1..]).map(|_| format!("{name} api GET"));
            }
            if spec.verbs.iter().any(|verb| verb == token) {
                return (words.len() >= spec.min_words).then(|| format!("{name} {joined}"));
            }
            if spec.max_words > 0 && words.len() >= spec.max_words {
                return None;
            }
        }
        index += 1;
    }

    if !spec.trailing {
        return None;
    }
    // Prefix, not equality: an allowed path names an operation, and its own
    // arguments follow it (`schema <method>`).
    if let Some(path) = matching_prefix(&spec.allow_paths, &words) {
        return Some(format!("{name} {path}"));
    }
    let last = words.last()?;
    (words.len() >= spec.min_words && spec.verbs.iter().any(|verb| verb == last))
        .then(|| format!("{name} {} {last}", words[0]))
}
