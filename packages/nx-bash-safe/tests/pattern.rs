//! The hand-written matchers, against the patterns they replace.
//!
//! Expected values were produced by running the original python patterns, not
//! by reasoning about what they ought to accept. That distinction matters most
//! for the sed substitution check, where the intuitive reading is wrong: a
//! greedy scanner rejects `s/a\/b/` and `s\a\b\`, which the original accepts,
//! because its engine backtracks and may read a backslash as an ordinary
//! character instead of an escape lead.

use nx_bash_safe::pattern;

/// `(script, accepted by the original)`.
const SUBSTITUTIONS: &[(&str, bool)] = &[
    // ordinary forms, one per delimiter
    ("s/a/b/", true),
    ("s|a|b|", true),
    ("s,a,b,", true),
    ("sXaXbX", true),
    ("s#a#b#", true),
    ("s@a@b@", true),
    ("s%a%b%", true),
    ("s^a^b^", true),
    ("s:a:b:", true),
    // flags
    ("s/a/b/g", true),
    ("s/a/b/i", true),
    ("s/a/b/p", true),
    ("s/a/b/m", true),
    ("s/a/b/2g", true),
    ("s/a/b/gI3", true),
    ("s/a/b/gg", true),
    ("s/a/b/0", true),
    // empty sections
    ("s//x/", true),
    ("s///", true),
    // escapes, and the cases a greedy scanner gets wrong
    (r"s/(.)/\1/", true),
    (r"s/a\/b/", true),
    (r"s\a\b\", true),
    (r"s\\a\\b\\", true),
    (r"s/\//x/", true),
    (r"s/a\\/b/", true),
    // a delimiter that is also an ordinary letter
    ("ss s s", true),
    // a `;` inside a bracket class, which is why the whole script is tried
    // before splitting it
    ("s/[;]/x/", true),
    // an expansion is data here: the walker decides whether it may appear
    ("s/a/$(evil)/", true),
    // rejected: w writes a file, e executes, x is not a flag
    ("s/a/b/w /tmp/x", false),
    ("s/a/b/e", false),
    ("s/a/b/x", false),
    // rejected: not enough delimiters
    ("s/a/b", false),
    ("s/a", false),
    ("sabc", false),
    ("s", false),
    // rejected: a newline cannot appear inside a section
    ("s/a\nb/c/", false),
    // rejected: two scripts joined, which the caller splits before retrying
    ("s/x/y/;s/z/w/", false),
];

#[test]
fn the_substitution_scanner_matches_the_pattern_it_replaces() {
    let mut failures = Vec::new();
    for (script, want) in SUBSTITUTIONS {
        let got = pattern::is_sed_substitution(script);
        if got != *want {
            failures.push(format!("{script:?}: want {want}, got {got}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The original's `$` also matched before one trailing newline, so it accepted
/// `"s/a/b/\n"`. This is stricter, and the difference is unreachable: every
/// caller strips the script before matching, and the fallback path splits it on
/// newlines first. Stricter means "prompt", which is the safe direction.
#[test]
fn a_trailing_newline_is_refused_where_the_original_allowed_it() {
    assert!(!pattern::is_sed_substitution("s/a/b/\n"));
    assert!(pattern::is_sed_substitution("s/a/b/\n".trim()));
}

#[test]
fn sed_addresses_are_stripped_before_the_body_is_judged() {
    for (script, want) in [
        ("p", 0),
        ("1p", 1),
        ("1,5p", 3),
        ("$d", 1),
        ("1~3p", 3),
        ("/start/,/end/p", 13),
        ("/a\\/b/d", 6),
        ("1!p", 2),
        ("1, 5 ! p", 7),
        ("s/a/b/", 0),
    ] {
        assert_eq!(
            pattern::sed_address_prefix_len(script),
            want,
            "address prefix of {script:?}"
        );
    }
}

#[test]
fn jq_program_loading_flags_are_caught_anchored() {
    // Anchoring matters: the original matched from the start of the token, and
    // an unanchored port would miss these.
    assert!(pattern::is_jq_unsafe_short("-f"));
    assert!(pattern::is_jq_unsafe_short("-rf"));
    assert!(pattern::is_jq_unsafe_short("-L"));
    assert!(!pattern::is_jq_unsafe_short("-r"));
    assert!(!pattern::is_jq_unsafe_short("-n"));

    assert!(pattern::has_jq_unsafe_word("jq env.PATH"));
    assert!(pattern::has_jq_unsafe_word("jq $ENV.PATH"));
    assert!(pattern::has_jq_unsafe_word("jq include \"x\""));
    assert!(!pattern::has_jq_unsafe_word("jq .environment"));
    assert!(!pattern::has_jq_unsafe_word("jq .envoy"));
}

#[test]
fn asana_write_tools_are_refused_by_whole_name() {
    // fullmatch, not prefix: `list_x` is a read, `update_task` is not, and a
    // port that anchors only the start would allow `get_x; rm -rf`.
    assert!(pattern::is_asana_read_tool("get_task"));
    assert!(pattern::is_asana_read_tool("search_tasks"));
    assert!(pattern::is_asana_read_tool("list_workspaces"));
    assert!(!pattern::is_asana_read_tool("update_task"));
    assert!(!pattern::is_asana_read_tool("create_task"));
    assert!(!pattern::is_asana_read_tool("get_Task"));
    assert!(!pattern::is_asana_read_tool("get_"));
}

#[test]
fn token_shapes() {
    assert!(pattern::is_assignment("AWS_PROFILE=prod"));
    assert!(pattern::is_assignment("X="));
    assert!(!pattern::is_assignment("1BAD=x"));
    assert!(!pattern::is_assignment("--flag"));

    assert!(pattern::is_numeric_flag("-5"));
    assert!(!pattern::is_numeric_flag("-a"));
    assert!(!pattern::is_numeric_flag("-"));

    assert!(pattern::is_combined_short("-la"));
    assert!(!pattern::is_combined_short("-l"));
    assert!(!pattern::is_combined_short("--all"));

    assert!(pattern::is_timeout_duration("30"));
    assert!(pattern::is_timeout_duration("1.5s"));
    assert!(pattern::is_timeout_duration("5m"));
    assert!(!pattern::is_timeout_duration("5x"));
    assert!(!pattern::is_timeout_duration("s"));

    // The refspec shape alone does not exclude a URL, because `:` and `/` are
    // both legal in a refspec. Rejecting `git fetch https://...` is a separate
    // check in the git matcher, and this asserts the division of labour rather
    // than a property this pattern has.
    assert!(pattern::is_fetch_ref("+refs/heads/*:refs/remotes/origin/*"));
    assert!(pattern::is_fetch_ref("https://example.com/repo"));
    assert!(!pattern::is_fetch_ref("-not-a-ref"));

    assert!(pattern::is_ps_env_cluster("-ef"));
    assert!(!pattern::is_ps_env_cluster("-af"));
}
