use std::io::Write;
use std::process::{Command, Output, Stdio};

use nx_comment_scan::{Kind, Report, scan};
use serde_json::{Value, json};

const BIN: &str = env!("CARGO_BIN_EXE_nx-comment-scan");

fn edit(path: &str, old: &str, new: &str) -> Value {
    json!({"tool_name": "Edit", "tool_input": {"file_path": path, "old_string": old, "new_string": new}})
}

fn write(path: &str, content: &str) -> Value {
    json!({"tool_name": "Write", "tool_input": {"file_path": path, "content": content}})
}

fn bash(command: &str) -> Value {
    json!({"tool_name": "Bash", "tool_input": {"command": command}})
}

fn added(payload: &Value) -> usize {
    scan(payload).iter().map(|report| report.added).sum()
}

fn only(payload: &Value) -> Report {
    let mut reports = scan(payload);
    assert_eq!(reports.len(), 1, "expected exactly one report");
    reports.pop().expect("report")
}

fn run(stdin: &str) -> Output {
    let mut child = Command::new(BIN)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(stdin.as_bytes())
        .expect("write");
    child.wait_with_output().expect("wait")
}

fn assert_silent(stdin: &str, what: &str) {
    let out = run(stdin);
    assert_eq!(out.status.code(), Some(0), "{what}: exit code");
    assert!(out.stdout.is_empty(), "{what}: stdout");
    assert!(out.stderr.is_empty(), "{what}: stderr");
}

#[test]
fn one_added_python_comment_counts() {
    assert_eq!(added(&edit("/tmp/x.py", "a = 1", "# set a\na = 1")), 1);
}

#[test]
fn removing_a_comment_counts_nothing() {
    assert_eq!(added(&edit("/tmp/x.py", "# set a\na = 1", "a = 1")), 0);
}

#[test]
fn reindenting_a_commented_block_counts_nothing() {
    let payload = edit(
        "/tmp/x.py",
        "# c\nif x:\n  y",
        "    # c\n    if x:\n        y",
    );
    assert_eq!(added(&payload), 0);
}

#[test]
fn every_line_of_a_block_comment_counts() {
    let payload = edit(
        "/tmp/l.rs",
        "let a = 1;",
        "/* why\n * because\n */\nlet a = 1;",
    );
    assert_eq!(added(&payload), 3);
}

#[test]
fn a_url_inside_a_string_is_not_a_comment() {
    let payload = edit("/tmp/a.ts", "x", "const u = \"https://x.dev\";");
    assert_eq!(added(&payload), 0);
}

#[test]
fn a_trailing_yaml_comment_counts() {
    let payload = write(
        "/tmp/a.yml",
        "paths-ignore:\n  - \".*\" # root dotfiles only\n",
    );
    assert_eq!(added(&payload), 1);
}

#[test]
fn a_yaml_block_scalar_of_javascript_counts_only_the_yaml_comments() {
    let content = concat!(
        "# A review that starts before the plan lands is a review of nothing.\n",
        "on:\n",
        "  pull_request:\n",
        "    # Everything else here is Terraform.\n",
        "    paths-ignore:\n",
        "      - \".*\" # root dotfiles only: `*` does not cross `/`\n",
        "      - \"intune/**\" # no Terraform root\n",
        "jobs:\n",
        "  gate:\n",
        "    steps:\n",
        "      - uses: actions/github-script@373c709 # v9.0.0\n",
        "        with:\n",
        "          script: |\n",
        "            // Author and position too: \"Quote reply\" copies raw markdown.\n",
        "            const body = [\n",
        "              '## Terraform plan을 확인한 뒤에 리뷰를 요청해 주세요',\n",
        "              '### What happened',\n",
        "            ].join('\\n');\n",
    );
    assert_eq!(added(&write("/tmp/gate.yml", content)), 5);
}

#[test]
fn a_quoted_markdown_heading_is_not_a_yaml_comment() {
    let payload = write("/tmp/a.yml", "body: |\n  const h = '## What happened';\n");
    assert_eq!(added(&payload), 0);
}

#[test]
fn a_url_fragment_is_not_a_trailing_comment() {
    let payload = write("/tmp/a.yml", "url: \"https://x.dev/a#frag\"\n");
    assert_eq!(added(&payload), 0);
}

#[test]
fn a_multiplication_is_not_a_block_comment_continuation() {
    assert_eq!(added(&write("/tmp/a.ts", "const area = w * h;\n")), 0);
}

#[test]
fn a_vim_string_is_not_a_trailing_comment() {
    assert_eq!(added(&write("/tmp/.vimrc", "let g:x = \"abc\"\n")), 0);
}

#[test]
fn an_unknown_extension_never_counts_a_trailing_comment() {
    assert_eq!(added(&write("/tmp/a.zzz", "attrs // { x = 1; }\n")), 0);
    assert_eq!(added(&write("/tmp/a.zzz", "// why\nattrs\n")), 1);
}

#[test]
fn a_nix_update_operator_is_not_a_comment() {
    assert_eq!(added(&write("/tmp/a.nix", "old // { x = 1; }\n")), 0);
    assert_eq!(added(&write("/tmp/a.nix", "x = 1; # why\n")), 1);
}

#[test]
fn a_trailing_comment_counts_in_every_family() {
    assert_eq!(
        added(&write("/tmp/a.py", "a = 1  # why this constant\n")),
        1
    );
    assert_eq!(added(&write("/tmp/a.ts", "let a = 1; // why\n")), 1);
    assert_eq!(added(&write("/tmp/a.sql", "SELECT 1 -- why\n")), 1);
    assert_eq!(added(&write("/tmp/a.tf", "count = 1 # why\n")), 1);
}

#[test]
fn css_counts_blocks_but_not_custom_properties() {
    assert_eq!(
        added(&write("/tmp/a.css", "/* palette */\n:root{--gap:1}")),
        1
    );
}

#[test]
fn markup_and_sql_and_makefile_are_covered() {
    assert_eq!(added(&write("/tmp/a.html", "<!-- nav -->\n<div/>")), 1);
    assert_eq!(
        added(&edit("/tmp/q.sql", "select 1", "-- rows\nselect 1")),
        1
    );
    assert_eq!(added(&write("/tmp/Makefile", "# build\nall:")), 1);
}

#[test]
fn prose_files_are_out_of_scope() {
    assert_eq!(added(&write("/tmp/a.md", "# heading\ntext")), 0);
    assert_eq!(added(&write("/tmp/a.json", "{\"a\": 1}")), 0);
}

#[test]
fn multiedit_sums_its_edits() {
    let payload = json!({"tool_name": "MultiEdit", "tool_input": {"file_path": "/tmp/x.py", "edits": [
        {"old_string": "a", "new_string": "# one\na"},
        {"old_string": "b", "new_string": "# two\nb"},
    ]}});
    assert_eq!(added(&payload), 2);
}

#[test]
fn a_reported_edit_prints_both_channels() {
    let out = run(&edit("/tmp/x.py", "a = 1", "# set a\na = 1").to_string());
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty());
    let printed: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(
        printed["systemMessage"],
        json!("comment-scan: +1 comment line(s) in x.py")
    );
    let hook = &printed["hookSpecificOutput"];
    assert_eq!(hook["hookEventName"], json!("PostToolUse"));
    let context = hook["additionalContext"].as_str().expect("context");
    assert!(context.contains("# set a"), "{context}");
}

#[test]
fn an_overwrite_that_adds_nothing_counts_nothing() {
    let body = "# first comment\n# second comment\na = 1\n";
    let payload = json!({"tool_name": "Write",
        "tool_input": {"file_path": "/tmp/demo.py", "content": body},
        "tool_response": {"type": "update", "originalFile": body, "structuredPatch": []}});
    assert_eq!(added(&payload), 0);
}

#[test]
fn an_overwrite_counts_only_the_surplus() {
    let payload = json!({"tool_name": "Write",
        "tool_input": {"file_path": "/tmp/demo.py", "content": "# one\n# two\na = 1\n"},
        "tool_response": {"type": "update", "originalFile": "# one\na = 1\n", "structuredPatch": []}});
    assert_eq!(added(&payload), 1);
}

#[test]
fn a_new_file_counts_all_of_its_comments() {
    let payload = json!({"tool_name": "Write",
        "tool_input": {"file_path": "/tmp/demo.py", "content": "# one\n# two\na = 1\n"},
        "tool_response": {"type": "create", "originalFile": null, "structuredPatch": []}});
    assert_eq!(added(&payload), 2);
}

#[test]
fn a_patch_counts_only_its_added_lines() {
    let payload = json!({"tool_name": "Edit",
    "tool_input": {"file_path": "/tmp/demo.py", "old_string": "a = 1", "new_string": "# second\na = 1"},
    "tool_response": {"originalFile": "# first\na = 1\n", "structuredPatch": [
        {"oldStart": 1, "oldLines": 2, "newStart": 1, "newLines": 3,
         "lines": [" # first", "+# second", " a = 1"]}
    ]}});
    let report = only(&payload);
    assert_eq!(report.added, 1);
    assert_eq!(report.samples, vec!["# second".to_string()]);
}

#[test]
fn dockerfile_variants_are_covered() {
    assert_eq!(added(&write("/tmp/Dockerfile", "# base\nFROM x")), 1);
    assert_eq!(added(&write("/tmp/Dockerfile.proxy", "# base\nFROM x")), 1);
}

#[test]
fn extensionless_files_default_to_hash() {
    assert_eq!(added(&write("/tmp/nx-lock", "#!/bin/sh\n# why\ntrue")), 1);
    assert_eq!(added(&write("/tmp/CODEOWNERS", "# owners\n* @me")), 1);
}

#[test]
fn a_shebang_is_never_a_comment() {
    assert_eq!(added(&write("/tmp/run.sh", "#!/usr/bin/env bash\ntrue")), 0);
    assert_eq!(added(&write("/tmp/run", "#!/bin/sh\ntrue")), 0);
}

#[test]
fn inline_script_metadata_is_read_by_uv_not_by_people() {
    let body = "# /// script\n# requires-python = \">=3.14\"\n# dependencies = [\n#     \"duckdb\",\n# ]\n# ///\nx = 1\n";
    assert!(scan(&bash(&format!("cat > q.py <<'PY'\n{body}PY"))).is_empty());
}

#[test]
fn preprocessor_directives_are_never_comments() {
    let body = "#include <stdio.h>\n#define X 1\n#pragma once\nint x;";
    assert_eq!(added(&write("/tmp/a.c", body)), 0);
    assert_eq!(added(&write("/tmp/a.unknownext", body)), 0);
}

#[test]
fn an_unknown_extension_matches_any_comment_syntax() {
    assert_eq!(added(&write("/tmp/a.weird", "# hash\nx")), 1);
    assert_eq!(added(&write("/tmp/a.weird", "// slash\nx")), 1);
    assert_eq!(added(&write("/tmp/a.weird", "-- dash\nx")), 1);
    assert_eq!(added(&write("/tmp/a.weird", "<!-- markup -->\nx")), 1);
    assert_eq!(added(&write("/tmp/a.weird", "% percent\nx")), 1);
}

#[test]
fn prose_and_data_stay_out_even_when_unknown_matching_is_on() {
    for name in [
        "a.md",
        "a.markdown",
        "a.mdx",
        "a.json",
        "a.txt",
        "a.csv",
        "a.lock",
        "a.diff",
    ] {
        assert_eq!(
            added(&write(&format!("/tmp/{name}"), "# h\n-- x\n// y")),
            0,
            "{name}"
        );
    }
}

#[test]
fn env_variants_are_covered() {
    assert_eq!(added(&write("/tmp/.env.local", "# key\nA=1")), 1);
    assert_eq!(added(&write("/tmp/app.env", "# key\nA=1")), 1);
}

#[test]
fn sibling_families_are_covered() {
    assert_eq!(added(&write("/tmp/a.pyx", "# cython\nx = 1")), 1);
    assert_eq!(added(&write("/tmp/a.mts", "// esm\nexport {}")), 1);
    assert_eq!(added(&write("/tmp/a.hujson", "// acl\n{}")), 1);
    assert_eq!(added(&write("/tmp/a.tex", "% latex\n\\x")), 1);
    assert_eq!(added(&write("/tmp/a.ml", "(* ocaml *)\nlet x = 1")), 1);
    assert_eq!(added(&write("/tmp/a.bat", "REM batch\necho x")), 1);
    assert_eq!(added(&write("/tmp/a.j2", "{# jinja #}\nx")), 1);
    assert_eq!(added(&write("/tmp/a.hbs", "{{! mustache }}\nx")), 1);
    assert_eq!(added(&write("/tmp/a.rkt", "; racket\n(x)")), 1);
    assert_eq!(
        added(&write("/tmp/p.sentinel", "# one\n// two\nmain = rule {}")),
        2
    );
}

#[test]
fn a_named_file_picks_its_own_family() {
    assert_eq!(added(&write("/tmp/Jenkinsfile", "// stage\nnode {}")), 1);
    assert_eq!(
        added(&write("/tmp/Jenkinsfile", "# not groovy\nnode {}")),
        0
    );
}

#[test]
fn templates_and_policies_are_covered() {
    assert_eq!(added(&write("/tmp/a.tpl", "{{/* helm */}}\nkey: 1")), 1);
    assert_eq!(added(&write("/tmp/a.tftpl", "# rendered\nkey = 1")), 1);
    assert_eq!(added(&write("/tmp/a.cedar", "// policy\npermit();")), 1);
    assert_eq!(
        added(&write("/tmp/tinyproxy.filter", "# hosts\n^x\\.dev$")),
        1
    );
}

#[test]
fn every_broken_payload_is_silent() {
    assert_silent("", "empty stdin");
    assert_silent("not json", "malformed json");
    assert_silent("{}", "empty object");
    assert_silent(&json!({"tool_name": "Edit"}).to_string(), "no tool_input");
    assert_silent(
        &json!({"tool_input": {"new_string": "# c"}}).to_string(),
        "no file_path",
    );
    assert_silent(&write("/tmp/a.md", "# heading").to_string(), "prose file");
    assert_silent(&edit("/tmp/x.py", "# c\na", "a").to_string(), "removal");
}

#[test]
fn a_heredoc_body_is_counted_against_its_redirect_target() {
    let report = only(&bash("cat > src/a.py <<'PY'\n# one\n# two\na = 1\nPY"));
    assert_eq!(report.added, 2);
    assert_eq!(report.path.as_deref(), Some("src/a.py"));
    assert!(report.kind == Kind::ShellFile);
}

#[test]
fn a_sed_inplace_edit_names_the_file_it_rewrites() {
    let report = only(&bash(
        "sed -i '2i\\ # Rectangle area: width times height.' shapes.py",
    ));
    assert_eq!(report.added, 1);
    assert_eq!(report.path.as_deref(), Some("shapes.py"));
    assert_eq!(
        report.samples,
        vec!["# Rectangle area: width times height.".to_string()]
    );
}

#[test]
fn an_inline_script_never_claims_a_file() {
    let report = only(&bash(
        "uv run python - <<'PY'\n# scratch\nprint(Path('a.py'))\nPY",
    ));
    assert_eq!(report.added, 1);
    assert_eq!(report.path, None);
    assert!(report.kind == Kind::ShellCommand);
}

#[test]
fn a_pipeline_hanging_off_a_heredoc_is_still_walked() {
    let report = only(&bash("cat <<'PY' | uv run python -\n# piped\nprint(1)\nPY"));
    assert_eq!(report.added, 1);
    assert_eq!(report.path, None);
}

#[test]
fn two_targets_in_one_command_are_reported_apart() {
    let reports = scan(&bash(
        "cat > a.py <<'A'\n# in a\nA\ncat > b.py <<'B'\n# in b\n# more b\nB",
    ));
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0].path.as_deref(), Some("a.py"));
    assert_eq!(reports[0].added, 1);
    assert_eq!(reports[1].path.as_deref(), Some("b.py"));
    assert_eq!(reports[1].added, 2);
}

#[test]
fn a_quoted_command_string_writes_nothing() {
    let payload = bash("for c in \"cat > x.py <<'PY'\n# one\nx = 1\nPY\"; do echo \"$c\"; done");
    assert_eq!(added(&payload), 0);
}

#[test]
fn a_tee_target_is_a_write() {
    let report = only(&bash("cat <<'EOF' | tee conf.py\n# teed\nEOF"));
    assert_eq!(report.path.as_deref(), Some("conf.py"));
    assert_eq!(report.added, 1);
}

#[test]
fn a_dev_null_redirect_is_not_a_write() {
    assert_eq!(added(&bash("echo '# x' > /dev/null")), 0);
    assert_eq!(added(&bash("grep '# x' a.py 2>&1")), 0);
}

#[test]
fn a_shell_edit_without_a_comment_is_silent() {
    assert_eq!(added(&bash("sed -i 's/a/b/' shapes.py")), 0);
    assert_eq!(added(&bash("echo 'x = 1' >> shapes.py")), 0);
}

#[test]
fn a_shell_command_that_writes_nothing_is_silent() {
    assert_eq!(added(&bash("rg '# TODO' src/")), 0);
    assert_eq!(added(&bash("cat shapes.py")), 0);
    assert_eq!(added(&bash("git commit -m '#31442 fix'")), 0);
}

#[test]
fn a_prose_target_stays_out_of_scope() {
    assert_eq!(added(&bash("cat > notes.md <<'EOF'\n# heading\nEOF")), 0);
    assert_eq!(added(&bash("cat > out.txt <<'EOF'\n# heading\nEOF")), 0);
}

#[test]
fn a_lowercase_bare_word_is_not_a_shell_target() {
    assert_eq!(added(&bash("git config --get x > '# not a comment'")), 0);
    assert_eq!(added(&bash("sed -i '1i\\ # owners' CODEOWNERS")), 1);
}

#[test]
fn a_targeted_node_heredoc_takes_its_family_from_the_file() {
    assert_eq!(
        added(&bash("cat > a.js <<'JS'\n// note\nconst x = 1;\nJS")),
        1
    );
}

#[test]
fn an_inline_node_script_is_missed_because_tier_two_assumes_hash() {
    assert_eq!(
        added(&bash("node - <<'JS'\n// note\nconsole.log(1)\nJS")),
        0
    );
}

#[test]
fn a_command_with_no_comment_marker_skips_the_parse() {
    assert!(scan(&bash("git status && cargo build")).is_empty());
}

#[test]
fn an_unparsable_command_is_silent() {
    assert_eq!(added(&bash("cat > x.py <<'PY'\n# one")), 0);
    assert_eq!(added(&bash("if then fi else")), 0);
}

#[test]
fn a_shell_report_names_the_file_it_scanned() {
    let out = run(&bash("sed -i '2i\\ # why' shapes.py").to_string());
    let printed: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(
        printed["systemMessage"],
        json!("comment-scan: ~1 comment line(s) in shapes.py")
    );
    let context = printed["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .expect("context");
    assert!(
        context.starts_with("A shell edit to shapes.py looks like it adds"),
        "{context}"
    );
    assert!(
        context.contains("Delete every comment line these edits added"),
        "{context}"
    );
}

#[test]
fn every_report_orders_the_deletion_and_spares_tool_directives() {
    let tool = json!({"tool_name": "Write",
        "tool_input": {"file_path": "/tmp/x.py", "content": "# set a\na = 1\n"},
        "tool_response": {"type": "create"}});
    let shell = bash("sed -i '2i\\ # why' shapes.py");
    for payload in [tool, shell] {
        let out = run(&payload.to_string());
        let printed: Value = serde_json::from_slice(&out.stdout).expect("json");
        let context = printed["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .expect("context");
        assert!(context.contains("Delete every comment line"), "{context}");
        assert!(
            context.contains("including any the report did not quote"),
            "{context}"
        );
        assert!(
            context.contains("Do that as your next action, before anything else."),
            "{context}"
        );
        assert!(
            context.contains("port the code, not the comments"),
            "{context}"
        );
        assert!(
            context.contains("Keep only machine-read directives"),
            "{context}"
        );
    }
}
