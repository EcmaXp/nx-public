//! Default-deny flag validation.
//!
//! A command is allowed as "this name, and only these flags". The table says
//! which flags exist and which consume the next token; anything absent from it
//! prompts. That direction is the whole point: a new flag on a familiar tool is
//! unknown until someone decides it is a read, and `git -c` proves why (it can
//! set `core.fsmonitor` to an arbitrary binary and turn `git status` into exec).

use crate::pattern;

/// One command's allowed flags. `true` means the flag consumes the next token.
#[derive(Debug, Default, Clone)]
pub struct FlagTable(Vec<(String, bool)>);

impl FlagTable {
    /// Parse a spec: space-separated flags, `:a` for one that takes an argument.
    pub fn parse(spec: &str) -> Self {
        Self(
            spec.split_whitespace()
                .map(|item| match item.split_once(':') {
                    Some((flag, "a")) => (flag.to_owned(), true),
                    Some((flag, _)) => (flag.to_owned(), false),
                    None => (item.to_owned(), false),
                })
                .collect(),
        )
    }

    /// `Some(takes_argument)` if the flag is allowed at all.
    pub fn arity(&self, flag: &str) -> Option<bool> {
        self.0
            .iter()
            .find(|(name, _)| name == flag)
            .map(|(_, takes_arg)| *takes_arg)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, bool)> {
        self.0.iter().map(|(name, arg)| (name.as_str(), *arg))
    }
}

#[derive(Debug, Default)]
pub struct Parsed {
    pub flags: Vec<String>,
    pub positionals: Vec<String>,
}

impl Parsed {
    pub fn has(&self, flag: &str) -> bool {
        self.flags.iter().any(|name| name == flag)
    }
}

/// Validate every flag against the table, or `None` if any is unknown.
pub fn parse_flags(args: &[String], table: &FlagTable, numeric_ok: bool) -> Option<Parsed> {
    let mut parsed = Parsed::default();
    let mut only_positional = false;
    let mut index = 0;

    while index < args.len() {
        let token = args[index].as_str();

        if only_positional || token == "-" || !token.starts_with('-') {
            parsed.positionals.push(token.to_owned());
            index += 1;
            continue;
        }
        if token == "--" {
            only_positional = true;
            index += 1;
            continue;
        }

        let base = token.split_once('=').map_or(token, |(head, _)| head);
        if let Some(takes_arg) = table.arity(base) {
            parsed.flags.push(base.to_owned());
            index += if takes_arg && !token.contains('=') {
                2
            } else {
                1
            };
            continue;
        }
        if numeric_ok && pattern::is_numeric_flag(token) {
            index += 1;
            continue;
        }
        if pattern::is_combined_short(token) && decompose(token, table, &mut parsed.flags) {
            // Only one token is consumed even when a decomposed letter takes an
            // argument, so `rg -nm 1` leaves `1` as a positional. Preserved
            // deliberately: the alternative reads better and classifies
            // differently.
            index += 1;
            continue;
        }
        return None;
    }
    Some(parsed)
}

/// Expand `-abc` letter by letter, stopping at the first that takes an argument.
fn decompose(token: &str, table: &FlagTable, flags: &mut Vec<String>) -> bool {
    for letter in token.chars().skip(1) {
        let flag = format!("-{letter}");
        match table.arity(&flag) {
            None => return false,
            Some(takes_arg) => {
                flags.push(flag);
                if takes_arg {
                    return true;
                }
            }
        }
    }
    true
}
