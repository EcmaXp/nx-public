//! Verb dispatch.
//!
//! `hook` is the one that runs on every Bash tool call, so it is the one with
//! the strict contract: silence on any failure, nothing on stderr, exit 0
//! always. The other verbs are development tools and are free to be loud.
//!
//! No argument-parsing crate: four verbs with a handful of flags do not justify
//! adding a dependency and its startup cost to a binary this hot.

use nx_bash_safe::hook;
use nx_bash_safe::policy::Policy;

const USAGE: &str = "\
usage: nx-bash-safe <verb> [options]

  hook                  read a PreToolUse payload on stdin, print an allow
                        decision or nothing; always exits 0
  explain <command>     say why a command is allowed, or where the proof ran out
  validate              load the policy loudly, reporting anything discarded
  audit [window|--sweep]  replay session logs and report the allow rate;
                        windows are elapsed time (today = last 24h, 1w = 7d)
  replay <fixture>      classify a whole NDJSON fixture, one row per input row
  dump-segments <f>     project each fixture command down to has_error, the
                        mis-parse cut offsets, and the token segments
  version               print the version

options:
  --policy <path>       load policy from a file or directory (repeatable)
  --cases <path>        validate: also run a command<TAB>allow|prompt file
  --tree                explain: also show has_error and the cut offsets
";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let verb = args.first().map(String::as_str).unwrap_or("");

    if verb == "hook" {
        // Everything past this point is best-effort. A panic here would
        // otherwise surface in the user's transcript as a hook failure.
        std::panic::set_hook(Box::new(|_| {}));
        let _ = std::panic::catch_unwind(|| {
            let policy = load_policy(&args[1..]).unwrap_or_default();
            let _ = hook::run(&policy);
        });
        std::process::exit(0);
    }

    let code = match verb {
        "replay" | "dump-segments" => {
            let Some(fixture) = args.get(1) else {
                eprintln!("usage: nx-bash-safe {verb} <fixture.jsonl>");
                std::process::exit(2);
            };
            let policy = load_policy(&args[1..]).unwrap_or_default();
            let result = if verb == "replay" {
                nx_bash_safe::replay::replay(fixture, &policy)
            } else {
                nx_bash_safe::replay::dump_segments(fixture, &policy)
            };
            match result {
                Ok(()) => 0,
                Err(error) => {
                    eprintln!("{verb}: {error}");
                    1
                }
            }
        }
        "explain" => {
            let policy = load_policy(&args[1..]).unwrap_or_default();
            let show_tree = args.iter().any(|arg| arg == "--tree");
            let command: Vec<&str> = args[1..]
                .iter()
                .filter(|arg| *arg != "--tree")
                .map(String::as_str)
                .collect();
            let command = command.join(" ");
            if command.trim().is_empty() {
                eprintln!("usage: nx-bash-safe explain <command>");
                std::process::exit(2);
            }
            nx_bash_safe::explain::explain(&command, &policy, show_tree)
        }
        "audit" => {
            let policy = load_policy(&args[1..]).unwrap_or_default();
            let workspace = option_value(&args, "--workspace")
                .map(std::path::PathBuf::from)
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_default();
            let windows = nx_bash_safe::audit::windows_from(&args[1..]);
            nx_bash_safe::audit::audit(&workspace, &windows, &policy)
        }
        "validate" => {
            let cases = option_value(&args, "--cases");
            nx_bash_safe::validate::run(&policy_paths(&args[1..]), cases.as_deref())
        }
        "version" => {
            println!("nx-bash-safe {}", env!("CARGO_PKG_VERSION"));
            0
        }
        "" => {
            print!("{USAGE}");
            0
        }
        other => {
            eprintln!("unknown verb: {other}\n\n{USAGE}");
            2
        }
    };
    std::process::exit(code);
}

/// Options are parsed leniently in hook mode on purpose: a malformed `--policy`
/// yields a narrower policy, which means more prompts, never fewer.
fn load_policy(args: &[String]) -> Option<Policy> {
    Some(nx_bash_safe::policy::load::load(&policy_paths(args)).0)
}

fn policy_paths(args: &[String]) -> Vec<String> {
    let mut paths = Vec::new();
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        if arg == "--policy" {
            paths.extend(rest.next().cloned());
        }
    }
    paths
}

fn option_value(args: &[String], name: &str) -> Option<String> {
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        if arg == name {
            return rest.next().cloned();
        }
    }
    None
}
