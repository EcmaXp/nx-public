use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use serde_json::{Value, json};

mod shell;

const SAMPLE_LIMIT: usize = 5;
pub(crate) const SHELL_SAMPLE_LIMIT: usize = 3;

pub(crate) const HASH: &[&str] = &["#"];
const SLASH: &[&str] = &["//", "/*", "*"];
const BLOCK: &[&str] = &["/*", "*"];
const MARKUP: &[&str] = &["<!--"];
const DASH: &[&str] = &["--"];
const SEMI: &[&str] = &[";"];
const TEMPLATE: &[&str] = &["#", "{{/*", "{{- /*"];
const JINJA: &[&str] = &["{#"];
const MUSTACHE: &[&str] = &["{{!"];
const HASH_SLASH: &[&str] = &["#", "//", "/*", "*"];
const PERCENT: &[&str] = &["%"];
const OCAML: &[&str] = &["(*", "*"];
const BATCH: &[&str] = &["REM", "rem", "::", "@REM"];
const QUOTE: &[&str] = &["\""];
static ANY: &[&str] = &[
    "#", "//", "/*", "*", "<!--", "--", ";", "{{/*", "{{- /*", "{#", "{{!", "%", "(*", "REM", "::",
];
const NEVER: &[&str] = &[
    "#!",
    "#define",
    "#elif",
    "#endif",
    "#endregion",
    "#ifdef",
    "#ifndef",
    "#include",
    "#pragma",
    "#region",
    "#undef",
];
const LINE_START_ONLY: &[&str] = &["*", "\"", "REM", "rem", "@REM", "::"];
static OPENER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?m)(?:^|['"\\ \t])[ \t]*(?:\#|//|/\*|<!--|--|\{\{[!/-]|\{\#|\(\*|::|@?REM|rem|\*|;|%|")"#,
    )
    .expect("the comment-opener gate must compile")
});
const METADATA: &[&str] = &["///", "requires-python", "dependencies", "[", "]", "\""];
const PROSE_OR_DATA: &[&str] = &[
    "csv", "diff", "json", "jsonl", "lock", "log", "markdown", "md", "mdx", "ndjson", "patch",
    "rst", "tsv", "txt",
];

const BY_EXTENSION: &[(&str, &[&str])] = &[
    ("awk", HASH),
    ("bash", HASH),
    ("bats", HASH),
    ("cfg", HASH),
    ("cmake", HASH),
    ("conf", HASH),
    ("cr", HASH),
    ("dash", HASH),
    ("desktop", HASH),
    ("dockerignore", HASH),
    ("ex", HASH),
    ("exs", HASH),
    ("filter", HASH),
    ("fish", HASH),
    ("gd", HASH),
    ("gemspec", HASH),
    ("gitattributes", HASH),
    ("gitconfig", HASH),
    ("gql", HASH),
    ("graphql", HASH),
    ("hcl", HASH),
    ("helmignore", HASH),
    ("ini", HASH),
    ("jl", HASH),
    ("ksh", HASH),
    ("mk", HASH),
    ("nim", HASH),
    ("nix", HASH),
    ("npmrc", HASH),
    ("nvmrc", HASH),
    ("pl", HASH),
    ("pm", HASH),
    ("properties", HASH),
    ("ps1", HASH),
    ("psd1", HASH),
    ("psm1", HASH),
    ("pxd", HASH),
    ("py", HASH),
    ("pyi", HASH),
    ("pyw", HASH),
    ("pyx", HASH),
    ("r", HASH),
    ("rake", HASH),
    ("rb", HASH),
    ("repo", HASH),
    ("ru", HASH),
    ("service", HASH),
    ("sh", HASH),
    ("spec", HASH),
    ("terraformignore", HASH),
    ("tf", HASH),
    ("tfbackend", HASH),
    ("tfvars", HASH),
    ("toml", HASH),
    ("unit", HASH),
    ("yaml", HASH),
    ("yml", HASH),
    ("zsh", HASH),
    ("c", SLASH),
    ("cc", SLASH),
    ("cedar", SLASH),
    ("cjs", SLASH),
    ("cpp", SLASH),
    ("cs", SLASH),
    ("csx", SLASH),
    ("cts", SLASH),
    ("cxx", SLASH),
    ("d", SLASH),
    ("dart", SLASH),
    ("frag", SLASH),
    ("fs", SLASH),
    ("fsx", SLASH),
    ("glsl", SLASH),
    ("go", SLASH),
    ("gradle", SLASH),
    ("groovy", SLASH),
    ("h", SLASH),
    ("hh", SLASH),
    ("hlsl", SLASH),
    ("hpp", SLASH),
    ("hujson", SLASH),
    ("hxx", SLASH),
    ("inl", SLASH),
    ("java", SLASH),
    ("js", SLASH),
    ("json5", SLASH),
    ("jsonc", SLASH),
    ("jsonnet", SLASH),
    ("jsx", SLASH),
    ("kt", SLASH),
    ("kts", SLASH),
    ("libsonnet", SLASH),
    ("mjs", SLASH),
    ("mts", SLASH),
    ("php", SLASH),
    ("prisma", SLASH),
    ("proto", SLASH),
    ("rs", SLASH),
    ("sass", SLASH),
    ("scala", SLASH),
    ("sol", SLASH),
    ("styl", SLASH),
    ("swift", SLASH),
    ("thrift", SLASH),
    ("ts", SLASH),
    ("tsx", SLASH),
    ("vert", SLASH),
    ("wgsl", SLASH),
    ("zig", SLASH),
    ("css", BLOCK),
    ("less", BLOCK),
    ("pcss", BLOCK),
    ("postcss", BLOCK),
    ("scss", BLOCK),
    ("astro", MARKUP),
    ("htm", MARKUP),
    ("html", MARKUP),
    ("plist", MARKUP),
    ("resx", MARKUP),
    ("svelte", MARKUP),
    ("svg", MARKUP),
    ("vue", MARKUP),
    ("xhtml", MARKUP),
    ("xml", MARKUP),
    ("xsl", MARKUP),
    ("xslt", MARKUP),
    ("adb", DASH),
    ("ads", DASH),
    ("applescript", DASH),
    ("elm", DASH),
    ("hs", DASH),
    ("lua", DASH),
    ("pgsql", DASH),
    ("psql", DASH),
    ("purs", DASH),
    ("sql", DASH),
    ("vhd", DASH),
    ("vhdl", DASH),
    ("ahk", SEMI),
    ("clj", SEMI),
    ("cljc", SEMI),
    ("cljs", SEMI),
    ("edn", SEMI),
    ("el", SEMI),
    ("fnl", SEMI),
    ("lisp", SEMI),
    ("rkt", SEMI),
    ("scm", SEMI),
    ("gotmpl", TEMPLATE),
    ("tftpl", TEMPLATE),
    ("tmpl", TEMPLATE),
    ("tpl", TEMPLATE),
    ("j2", JINJA),
    ("jinja", JINJA),
    ("jinja2", JINJA),
    ("handlebars", MUSTACHE),
    ("hbs", MUSTACHE),
    ("mustache", MUSTACHE),
    ("sentinel", HASH_SLASH),
    ("bzl", HASH),
    ("cnf", HASH),
    ("dockerfile", HASH),
    ("env", HASH),
    ("gitmodules", HASH),
    ("ignore", HASH),
    ("just", HASH),
    ("nomad", HASH),
    ("nu", HASH),
    ("pp", HASH),
    ("rego", HASH),
    ("sls", HASH),
    ("star", HASH),
    ("tfignore", HASH),
    ("cue", SLASH),
    ("gvy", SLASH),
    ("ino", SLASH),
    ("pug", SLASH),
    ("bib", PERCENT),
    ("cls", PERCENT),
    ("erl", PERCENT),
    ("hrl", PERCENT),
    ("sty", PERCENT),
    ("tex", PERCENT),
    ("ml", OCAML),
    ("mli", OCAML),
    ("mll", OCAML),
    ("mly", OCAML),
    ("bat", BATCH),
    ("cmd", BATCH),
    ("vim", QUOTE),
    ("vimrc", QUOTE),
];

const BY_NAME: &[(&str, &[&str])] = &[
    (".editorconfig", HASH),
    (".env", HASH),
    (".envrc", HASH),
    (".gitignore", HASH),
    (".zshenv", HASH),
    ("CODEOWNERS", HASH),
    ("Brewfile", HASH),
    ("Jenkinsfile", SLASH),
    ("Makefile", HASH),
    ("config", HASH),
    ("direnvrc", HASH),
    ("pre-commit", HASH),
    ("pre-push", HASH),
    ("reference-transaction", HASH),
];

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Tool,
    ShellFile,
    ShellCommand,
}

pub struct Report {
    pub kind: Kind,
    pub added: usize,
    pub path: Option<String>,
    pub samples: Vec<String>,
}

pub fn prefixes(path: &str) -> Option<&'static [&'static str]> {
    let name = Path::new(path).file_name()?.to_str()?;
    if let Some((_, found)) = BY_NAME.iter().find(|(n, _)| *n == name) {
        return Some(found);
    }
    if name.starts_with("Dockerfile") || name.starts_with(".env") {
        return Some(HASH);
    }
    let Some(ext) = Path::new(name).extension().and_then(|e| e.to_str()) else {
        return Some(HASH);
    };
    let ext = ext.to_ascii_lowercase();
    if let Some((_, found)) = BY_EXTENSION.iter().find(|(e, _)| *e == ext) {
        return Some(found);
    }
    if PROSE_OR_DATA.contains(&ext.as_str()) {
        return None;
    }
    Some(ANY)
}

fn inline_metadata(line: &str) -> bool {
    let Some(rest) = line.strip_prefix('#') else {
        return false;
    };
    let rest = rest.trim_start();
    METADATA.iter().any(|key| rest.starts_with(key))
}

pub(crate) fn marked(text: &str) -> bool {
    OPENER.is_match(text)
}

fn unmarked(input: &Value, patched: &[String]) -> bool {
    if patched.is_empty() {
        return !pairs(input).iter().any(|(_, new)| marked(new));
    }
    !patched.iter().any(|line| marked(line))
}

pub(crate) fn is_comment(line: &str, prefixes: &[&str]) -> bool {
    let trimmed = line.trim_start();
    if NEVER.iter().any(|p| trimmed.starts_with(p)) || inline_metadata(trimmed) {
        return false;
    }
    prefixes.iter().any(|p| trimmed.starts_with(p)) || trailing(line, prefixes)
}

fn trailing(line: &str, prefixes: &[&str]) -> bool {
    if std::ptr::eq(prefixes, ANY) {
        return false;
    }
    let bytes = line.as_bytes();
    let mut quote: Option<u8> = None;
    for (index, byte) in bytes.iter().enumerate() {
        match quote {
            Some(open) => {
                if *byte == open {
                    quote = None;
                }
            }
            None if *byte == b'\'' || *byte == b'"' => quote = Some(*byte),
            None if index > 0 && (bytes[index - 1] == b' ' || bytes[index - 1] == b'\t') => {
                let rest = &line[index..];
                if prefixes
                    .iter()
                    .any(|p| !LINE_START_ONLY.contains(p) && rest.starts_with(p))
                {
                    return true;
                }
            }
            None => {}
        }
    }
    false
}

pub fn scan(payload: &Value) -> Vec<Report> {
    if payload.get("tool_name").and_then(Value::as_str) == Some("Bash") {
        return payload
            .get("tool_input")
            .and_then(|input| input.get("command"))
            .and_then(Value::as_str)
            .map(shell::scan)
            .unwrap_or_default();
    }
    scan_tool(payload).into_iter().collect()
}

fn scan_tool(payload: &Value) -> Option<Report> {
    let input = payload.get("tool_input")?;
    let path = input.get("file_path").and_then(Value::as_str)?;
    let prefixes = prefixes(path)?;
    let response = payload.get("tool_response");
    let patched = patch_additions(response);
    if unmarked(input, &patched) {
        return None;
    }

    let mut added = 0;
    let mut samples = Vec::new();
    if !patched.is_empty() {
        for line in patched {
            if is_comment(&line, prefixes) {
                added += 1;
                samples.push(line.trim().to_string());
            }
        }
    } else if let (Some(original), Some(content)) = (
        response
            .and_then(|r| r.get("originalFile"))
            .and_then(Value::as_str),
        input.get("content").and_then(Value::as_str),
    ) {
        added = count(content, prefixes).saturating_sub(count(original, prefixes));
        samples.extend(comment_lines(content, prefixes));
    } else {
        for (old, new) in pairs(input) {
            added += count(new, prefixes).saturating_sub(count(old, prefixes));
            samples.extend(comment_lines(new, prefixes));
        }
    }
    if added == 0 {
        return None;
    }
    samples.truncate(SAMPLE_LIMIT);
    Some(Report {
        kind: Kind::Tool,
        added,
        path: Some(path.to_string()),
        samples,
    })
}

pub(crate) fn named_family(token: &str) -> Option<&'static [&'static str]> {
    let name = Path::new(token).file_name()?.to_str()?;
    let spelled_out = name.starts_with('.') || name.starts_with(char::is_uppercase);
    if spelled_out {
        if let Some((_, found)) = BY_NAME.iter().find(|(n, _)| *n == name) {
            return Some(found);
        }
        if name.starts_with("Dockerfile") || name.starts_with(".env") {
            return Some(HASH);
        }
    }
    let ext = Path::new(name).extension()?.to_str()?.to_ascii_lowercase();
    BY_EXTENSION
        .iter()
        .find(|(e, _)| *e == ext)
        .map(|(_, found)| *found)
}

const DELETE_ORDER: &str = "Delete every comment line this edit added, including any the report did not quote. Do that \
as your next action, before anything else. This holds even when the task text or the code \
you are porting carries them: port the code, not the comments. Keep only machine-read \
directives (noqa, type-ignore, eslint-disable).";
const SHELL_DELETE_ORDER: &str = "Delete every comment line these edits added, including any the report did not quote. Do that \
as your next action, before anything else. This holds even when the task text or the code \
you are porting carries them: port the code, not the comments. Keep only machine-read \
directives (noqa, type-ignore, eslint-disable).";

pub fn output(reports: &[Report]) -> Value {
    let tally: Vec<String> = reports
        .iter()
        .enumerate()
        .map(|(index, report)| phrase(report, index == 0))
        .collect();
    let blocks: Vec<String> = reports.iter().map(block).collect();
    let guidance = if reports.first().map(|report| report.kind) == Some(Kind::Tool) {
        DELETE_ORDER
    } else {
        SHELL_DELETE_ORDER
    };
    json!({
        "systemMessage": format!("comment-scan: {}", tally.join(", ")),
        "hookSpecificOutput": {
            "hookEventName": "PostToolUse",
            "additionalContext": format!("{}\n{guidance}", blocks.join("\n")),
        }
    })
}

fn phrase(report: &Report, first: bool) -> String {
    let sign = if report.kind == Kind::Tool { '+' } else { '~' };
    let place = match &report.path {
        Some(path) => basename(path),
        None => "this shell command".to_string(),
    };
    if first {
        format!("{sign}{} comment line(s) in {place}", report.added)
    } else {
        format!("{sign}{} in {place}", report.added)
    }
}

fn block(report: &Report) -> String {
    let quoted: String = report
        .samples
        .iter()
        .map(|line| format!("\n  {line}"))
        .collect();
    let lead = match (report.kind, &report.path) {
        (Kind::Tool, Some(path)) => format!("Added {} comment line(s) to {path}:", report.added),
        (Kind::ShellFile, Some(path)) => format!(
            "A shell edit to {path} looks like it adds {} comment line(s):",
            report.added
        ),
        _ => format!(
            "This shell command carries {} comment line(s):",
            report.added
        ),
    };
    format!("{lead}{quoted}")
}

fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_string()
}

fn count(text: &str, prefixes: &[&str]) -> usize {
    text.lines().filter(|l| is_comment(l, prefixes)).count()
}

fn comment_lines(text: &str, prefixes: &[&str]) -> Vec<String> {
    text.lines()
        .filter(|l| is_comment(l, prefixes))
        .map(|l| l.trim().to_string())
        .collect()
}

fn patch_additions(response: Option<&Value>) -> Vec<String> {
    let mut out = Vec::new();
    let hunks = response
        .and_then(|r| r.get("structuredPatch"))
        .and_then(Value::as_array);
    for hunk in hunks.into_iter().flatten() {
        let lines = hunk.get("lines").and_then(Value::as_array);
        for line in lines.into_iter().flatten() {
            if let Some(rest) = line.as_str().and_then(|l| l.strip_prefix('+')) {
                out.push(rest.to_string());
            }
        }
    }
    out
}

fn pairs(input: &Value) -> Vec<(&str, &str)> {
    if let Some(edits) = input.get("edits").and_then(Value::as_array) {
        return edits
            .iter()
            .map(|edit| (text(edit, "old_string"), text(edit, "new_string")))
            .collect();
    }
    let new = if input.get("content").is_some() {
        text(input, "content")
    } else {
        text(input, "new_string")
    };
    vec![(text(input, "old_string"), new)]
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{ANY, BY_EXTENSION, BY_NAME, LINE_START_ONLY, marked};

    fn families() -> impl Iterator<Item = &'static [&'static str]> {
        BY_EXTENSION
            .iter()
            .map(|(_, family)| *family)
            .chain(BY_NAME.iter().map(|(_, family)| *family))
            .chain([ANY])
    }

    #[test]
    fn the_gate_admits_every_comment_opener_a_family_can_use() {
        for family in families() {
            for prefix in family {
                assert!(
                    marked(&format!("  {prefix} note")),
                    "the gate would skip a comment opening with {prefix}"
                );
            }
        }
    }

    #[test]
    fn the_gate_admits_every_opener_that_may_trail_code() {
        for family in families() {
            for prefix in family {
                if LINE_START_ONLY.contains(prefix) {
                    continue;
                }
                assert!(
                    marked(&format!("a: 1 {prefix} note")),
                    "the gate would skip a trailing comment opening with {prefix}"
                );
            }
        }
    }
}
