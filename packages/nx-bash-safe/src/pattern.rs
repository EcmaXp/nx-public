//! Hand-written matchers, where a regex crate cannot express the rule.
//!
//! The `sed` substitution check is the reason this file exists. Its pattern
//! uses a capture group as the delimiter and then refers back to it inside a
//! negative lookahead, and the `regex` crate has neither backreferences nor
//! lookaround. It is also the one place where a hand-port can silently *widen*
//! the allow surface, since `sed read-only` is high-traffic, so it is
//! implemented as an exact reachability scan rather than a plausible scanner.
//!
//! Everywhere else the rule is: when the ported matcher and the original could
//! disagree, disagree toward *prompt*. The originals were Unicode-aware
//! (python's `\d`, `\s`, `\b`); these are ASCII, which rejects some inputs the
//! original accepted and accepts none that it rejected.

/// `^[A-Za-z_][A-Za-z0-9_]*=`
pub fn is_assignment(token: &str) -> bool {
    let mut chars = token.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    for char in chars {
        if char == '=' {
            return true;
        }
        if !char.is_ascii_alphanumeric() && char != '_' {
            return false;
        }
    }
    false
}

/// `^-\d+$`, e.g. the `-5` in `head -5`.
pub fn is_numeric_flag(token: &str) -> bool {
    match token.strip_prefix('-') {
        Some(rest) => !rest.is_empty() && rest.chars().all(|char| char.is_ascii_digit()),
        None => false,
    }
}

/// `^-[A-Za-z0-9]{2,}$`, a short cluster like `-la`.
pub fn is_combined_short(token: &str) -> bool {
    match token.strip_prefix('-') {
        Some(rest) => rest.chars().count() >= 2 && rest.chars().all(|c| c.is_ascii_alphanumeric()),
        None => false,
    }
}

/// `^\d+(?:\.\d+)?[smhd]?$`, a `timeout` duration.
pub fn is_timeout_duration(token: &str) -> bool {
    let body = token.strip_suffix(['s', 'm', 'h', 'd']).unwrap_or(token);
    if body.is_empty() {
        return false;
    }
    match body.split_once('.') {
        Some((whole, fraction)) => {
            !whole.is_empty()
                && !fraction.is_empty()
                && whole.chars().all(|c| c.is_ascii_digit())
                && fraction.chars().all(|c| c.is_ascii_digit())
        }
        None => body.chars().all(|c| c.is_ascii_digit()),
    }
}

/// `^[A-Za-z0-9_][A-Za-z0-9_-]*$`, a plain remote or tag name.
pub fn is_simple_name(token: &str) -> bool {
    let mut chars = token.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphanumeric() || first == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// `^[+A-Za-z0-9_][A-Za-z0-9_./*:-]*$`, a fetch refspec with no URL in it.
pub fn is_fetch_ref(token: &str) -> bool {
    let mut chars = token.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphanumeric() || first == '_' || first == '+' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '*' | ':' | '-'))
}

/// A short cluster containing `f` or `L`, which load a jq program from disk.
///
/// The original anchored this at the start of the token, which `Regex::is_match`
/// would not: missing that would let `--from-file` style tokens slip past.
pub fn is_jq_unsafe_short(token: &str) -> bool {
    let Some(rest) = token.strip_prefix('-') else {
        return false;
    };
    rest.chars().all(|c| c.is_ascii_alphabetic()) && rest.contains(['f', 'L'])
}

/// `(get|search|list)_[a-z_]+`, matched against the *whole* name.
pub fn is_asana_read_tool(name: &str) -> bool {
    let Some(rest) = ["get_", "search_", "list_"]
        .iter()
        .find_map(|prefix| name.strip_prefix(prefix))
    else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|c| c.is_ascii_lowercase() || c == '_')
}

/// An `e` inside a short cluster, which is BSD ps for "show the environment of
/// other processes" and therefore leaks their secrets.
pub fn is_ps_env_cluster(token: &str) -> bool {
    let body = token.strip_prefix('-').unwrap_or(token);
    body.chars().all(|c| c.is_ascii_alphabetic()) && body.contains('e')
}

/// `\benv\b`, `$ENV\b`, `\binclude\b`, `\bimport\b` anywhere in the arguments.
pub fn has_jq_unsafe_word(joined: &str) -> bool {
    ["env", "include", "import"]
        .iter()
        .any(|word| contains_word(joined, word))
        || contains_dollar_env(joined)
}

fn is_word_char(char: char) -> bool {
    char.is_ascii_alphanumeric() || char == '_'
}

fn contains_word(haystack: &str, needle: &str) -> bool {
    let bytes = haystack.as_bytes();
    let mut from = 0;
    while let Some(offset) = haystack[from..].find(needle) {
        let start = from + offset;
        let end = start + needle.len();
        let before_ok = start == 0 || !is_word_char(bytes[start - 1] as char);
        let after_ok = end == bytes.len() || !is_word_char(bytes[end] as char);
        if before_ok && after_ok {
            return true;
        }
        from = start + 1;
    }
    false
}

fn contains_dollar_env(haystack: &str) -> bool {
    let bytes = haystack.as_bytes();
    let mut from = 0;
    while let Some(offset) = haystack[from..].find("$ENV") {
        let end = from + offset + 4;
        if end == bytes.len() || !is_word_char(bytes[end] as char) {
            return true;
        }
        from = from + offset + 1;
    }
    false
}

/// Length of the leading sed address, or 0 when the script has none.
///
/// `\d+(~\d+)?` | `$` | `/regex/`, optionally a second after a comma, then an
/// optional `!`. Mirrors a single substitution of the original prefix pattern.
pub fn sed_address_prefix_len(script: &str) -> usize {
    let chars: Vec<char> = script.chars().collect();
    let Some(mut cursor) = one_address(&chars, 0) else {
        return 0;
    };
    let after_first = cursor;
    let mut probe = skip_space(&chars, cursor);
    if chars.get(probe) == Some(&',') {
        probe = skip_space(&chars, probe + 1);
        match one_address(&chars, probe) {
            Some(end) => cursor = end,
            None => cursor = after_first,
        }
    }
    cursor = skip_space(&chars, cursor);
    if chars.get(cursor) == Some(&'!') {
        cursor += 1;
    }
    cursor = skip_space(&chars, cursor);
    chars[..cursor].iter().map(|c| c.len_utf8()).sum()
}

fn skip_space(chars: &[char], mut index: usize) -> usize {
    while index < chars.len() && chars[index].is_whitespace() {
        index += 1;
    }
    index
}

fn one_address(chars: &[char], start: usize) -> Option<usize> {
    let mut index = start;
    match chars.get(index)? {
        '$' => Some(index + 1),
        '/' => {
            index += 1;
            while index < chars.len() {
                match chars[index] {
                    '\\' if index + 1 < chars.len() => index += 2,
                    '/' => return Some(index + 1),
                    _ => index += 1,
                }
            }
            None
        }
        char if char.is_ascii_digit() => {
            while index < chars.len() && chars[index].is_ascii_digit() {
                index += 1;
            }
            if chars.get(index) == Some(&'~') {
                let mut step = index + 1;
                while step < chars.len() && chars[step].is_ascii_digit() {
                    step += 1;
                }
                if step > index + 1 {
                    index = step;
                }
            }
            Some(index)
        }
        _ => None,
    }
}

const SUBST_FLAGS: &[char] = &['g', 'i', 'p', 'I', 'm'];

/// True exactly when the original substitution pattern matched.
///
/// A greedy scanner is wrong here, and provably so: `s/a\/b/` and `s\a\b\` both
/// match, because the engine backtracks and may read a backslash as an ordinary
/// character rather than as an escape lead. So this walks the two sections as a
/// reachability problem instead, taking both steps wherever both apply. Linear
/// time, no backtracking, and exact.
pub fn is_sed_substitution(script: &str) -> bool {
    let chars: Vec<char> = script.chars().collect();
    let count = chars.len();
    if count < 2 || chars[0] != 's' || chars[1] == '\n' {
        return false;
    }
    let delimiter = chars[1];

    // A section starts after the opening delimiter and ends at the next one.
    let mut reachable = vec![false; count + 1];
    reachable[2] = true;
    let ends_of_pattern = advance(&chars, &mut reachable, delimiter, 2);

    let mut reachable = vec![false; count + 1];
    for end in &ends_of_pattern {
        reachable[end + 1] = true;
    }
    let first_seed = ends_of_pattern.iter().map(|end| end + 1).min();
    let Some(seed) = first_seed else {
        return false;
    };
    let ends_of_replacement = advance(&chars, &mut reachable, delimiter, seed);

    // Everything after the closing delimiter must be a substitution flag.
    let mut flags_start = count;
    while flags_start > 0 {
        let candidate = chars[flags_start - 1];
        if SUBST_FLAGS.contains(&candidate) || candidate.is_ascii_digit() {
            flags_start -= 1;
        } else {
            break;
        }
    }
    ends_of_replacement.iter().any(|end| end + 1 >= flags_start)
}

/// One forward pass, marking every position the section can reach, and
/// returning the positions where it can sit on the delimiter.
fn advance(chars: &[char], reachable: &mut [bool], delimiter: char, from: usize) -> Vec<usize> {
    let count = chars.len();
    let mut ends = Vec::new();
    for index in from..count {
        if !reachable[index] {
            continue;
        }
        if chars[index] == delimiter {
            ends.push(index);
        }
        // Both steps may apply at the same position: that is the backtracking
        // the original pattern relied on.
        if chars[index] == '\\' && index + 1 < count && chars[index + 1] != '\n' {
            reachable[index + 2] = true;
        }
        if chars[index] != delimiter && chars[index] != '\n' {
            reachable[index + 1] = true;
        }
    }
    ends
}
