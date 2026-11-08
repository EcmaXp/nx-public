use std::cell::RefCell;
use std::sync::LazyLock;

use regex::Regex;

use tree_sitter::{Node, Parser};

use crate::{
    HASH, Kind, Report, SHELL_SAMPLE_LIMIT, comment_lines, is_comment, is_markdown, marked,
    metadata_entry, named_family, sed,
};

const MAX_DEPTH: usize = 200;
const BOUNDARIES: &[char] = &['\'', '"', '\\'];
const DISCARDED: &[&str] = &["/dev/null", "/dev/stdout", "/dev/stderr", "/dev/tty"];
const WRITE_OPERATORS: &[&str] = &[">", ">>", "&>", "&>>", ">|"];
const INTERPRETERS: &[&str] = &[
    "python", "python3", "uv", "ruby", "perl", "node", "bash", "sh", "zsh", "fish",
];
const SITE_NODES: &[&str] = &["redirected_statement", "pipeline", "command"];
const CHAINS: &[&str] = &["list", "pipeline"];

static QUOTED_PATH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"["']([^"'\s]+\.[A-Za-z0-9]+)["']"#).expect("the quoted-path pattern must compile")
});

thread_local! {
    static PARSER: RefCell<Parser> = RefCell::new({
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter::Language::new(tree_sitter_bash::LANGUAGE))
            .expect("the pinned bash grammar must load");
        parser
    });
}

enum Inserted {
    Lines(Vec<String>),
    Scripts(Vec<String>),
}

#[derive(Default)]
struct Site {
    targets: Vec<String>,
    bodies: Vec<String>,
    names: Vec<String>,
    args: Vec<String>,
    inplace: Vec<(String, Inserted)>,
}

pub(crate) fn scan(command: &str) -> Vec<Report> {
    if !marked(command) && !command.split_whitespace().any(names_sed) {
        return Vec::new();
    }
    let src = command.as_bytes();
    let Some(tree) = PARSER.with(|parser| parser.borrow_mut().parse(src, None)) else {
        return Vec::new();
    };
    let root = tree.root_node();
    if root.has_error() {
        return Vec::new();
    }
    let mut out = Vec::new();
    walk(root, src, 0, &mut out);
    out
}

fn walk(node: Node, src: &[u8], depth: usize, out: &mut Vec<Report>) {
    if depth > MAX_DEPTH {
        return;
    }
    if node.kind() == "redirected_statement"
        && let Some(body) = node.child_by_field_name("body")
        && CHAINS.contains(&body.kind())
    {
        let mut site = Site::default();
        split(body, src, depth + 1, out, &mut site);
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.id() != body.id() {
                collect(child, src, depth + 1, &mut site);
            }
        }
        report(site, out);
        return;
    }
    if SITE_NODES.contains(&node.kind()) {
        let mut site = Site::default();
        collect(node, src, depth, &mut site);
        report(site, out);
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(child, src, depth + 1, out);
    }
}

fn split(node: Node, src: &[u8], depth: usize, out: &mut Vec<Report>, site: &mut Site) {
    if depth > MAX_DEPTH {
        return;
    }
    if !CHAINS.contains(&node.kind()) {
        collect(node, src, depth, site);
        return;
    }
    let mut cursor = node.walk();
    let parts: Vec<Node> = node.named_children(&mut cursor).collect();
    if let Some((last, rest)) = parts.split_last() {
        for part in rest {
            walk(*part, src, depth + 1, out);
        }
        split(*last, src, depth + 1, out, site);
    }
}

fn collect(node: Node, src: &[u8], depth: usize, site: &mut Site) {
    if depth > MAX_DEPTH {
        return;
    }
    match node.kind() {
        "heredoc_body" => site.bodies.push(text(node, src)),
        "file_redirect" => {
            if let Some(target) = redirect_target(node, src) {
                site.targets.push(target);
            }
        }
        "command" => command(node, src, site),
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect(child, src, depth + 1, site);
    }
}

fn command(node: Node, src: &[u8], site: &mut Site) {
    let mut name = String::new();
    let mut args = Vec::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "command_name" => name = text(child, src),
            "file_redirect" | "heredoc_redirect" | "herestring_redirect" => {}
            _ => args.push(unquote(child, src)),
        }
    }
    if name.is_empty() {
        return;
    }
    site.names.push(name.clone());
    let base = name.rsplit('/').next().unwrap_or(&name).to_string();

    if base == "tee" {
        site.targets
            .extend(args.iter().filter(|arg| resolves(arg)).cloned());
    } else if base == "sed" {
        let invocation = sed::invocation(&args);
        if invocation.inplace
            && let Some(target) = invocation.files.iter().rev().find(|file| resolves(file))
        {
            let inserted = match sed::inserted_lines(&invocation.scripts) {
                Some(lines) => Inserted::Lines(
                    lines
                        .into_iter()
                        .filter(|line| !metadata_entry(line))
                        .collect(),
                ),
                None => Inserted::Scripts(invocation.scripts.clone()),
            };
            site.inplace.push((target.clone(), inserted));
            return;
        }
    } else if base == "perl"
        && args.iter().any(|arg| arg.starts_with("-i"))
        && let Some(position) = args.iter().rposition(|arg| resolves(arg))
    {
        let target = args[position].clone();
        let scripts = args
            .iter()
            .enumerate()
            .filter(|(index, arg)| *index != position && !arg.starts_with('-'))
            .map(|(_, arg)| arg.clone())
            .collect();
        site.inplace.push((target, Inserted::Scripts(scripts)));
        return;
    }
    site.args.extend(args);
}

fn report(site: Site, out: &mut Vec<Report>) {
    for (target, inserted) in &site.inplace {
        let Some(prefixes) = named_family(target) else {
            continue;
        };
        let counted = match inserted {
            Inserted::Lines(lines) => exact(lines, prefixes),
            Inserted::Scripts(scripts) => fuzzy(scripts, prefixes),
        };
        push(out, Kind::ShellFile, Some(target), counted);
    }

    if let Some((target, prefixes)) = site
        .targets
        .iter()
        .find_map(|target| named_family(target).map(|prefixes| (target, prefixes)))
    {
        if site.bodies.is_empty() {
            push(
                out,
                Kind::ShellFile,
                Some(target),
                exact(&site.args, prefixes),
            );
        }
        for body in &site.bodies {
            push(
                out,
                Kind::ShellFile,
                Some(target),
                exact_text(body, prefixes),
            );
        }
        return;
    }

    if site.bodies.is_empty() || !site.names.iter().any(interpreter) {
        return;
    }
    let counted: Vec<(usize, Vec<String>)> = site
        .bodies
        .iter()
        .filter(|body| !writes_only_markdown(body, &site.args))
        .map(|body| exact_text(body, HASH))
        .collect();
    let added = counted.iter().map(|(count, _)| count).sum();
    let samples = counted.into_iter().flat_map(|(_, lines)| lines).collect();
    push(out, Kind::ShellCommand, None, (added, samples));
}

fn push(out: &mut Vec<Report>, kind: Kind, path: Option<&String>, counted: (usize, Vec<String>)) {
    let (added, mut samples) = counted;
    if added == 0 {
        return;
    }
    samples.truncate(SHELL_SAMPLE_LIMIT);
    out.push(Report {
        kind,
        added,
        path: path.cloned(),
        samples,
    });
}

fn exact(texts: &[String], prefixes: &[&str]) -> (usize, Vec<String>) {
    let samples: Vec<String> = texts
        .iter()
        .flat_map(|text| comment_lines(text, prefixes))
        .collect();
    (samples.len(), samples)
}

fn exact_text(text: &str, prefixes: &[&str]) -> (usize, Vec<String>) {
    exact(&[text.to_string()], prefixes)
}

fn fuzzy(scripts: &[String], prefixes: &[&str]) -> (usize, Vec<String>) {
    let mut added = 0;
    let mut samples = Vec::new();
    for line in scripts.iter().flat_map(|script| script.lines()) {
        if let Some(found) = fuzzy_line(line, prefixes) {
            added += 1;
            samples.push(found);
        }
    }
    (added, samples)
}

fn fuzzy_line(line: &str, prefixes: &[&str]) -> Option<String> {
    line.char_indices()
        .filter(|(_, character)| BOUNDARIES.contains(character))
        .map(|(index, character)| index + character.len_utf8())
        .find_map(|start| {
            let rest = line[start..].trim_start();
            is_comment(rest, prefixes).then(|| rest.trim().to_string())
        })
        .or_else(|| is_comment(line, prefixes).then(|| line.trim().to_string()))
}

fn redirect_target(node: Node, src: &[u8]) -> Option<String> {
    let mut operator = String::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if !child.is_named() {
            operator = text(child, src);
        }
    }
    if !WRITE_OPERATORS.contains(&operator.as_str()) {
        return None;
    }
    let destination = unquote(node.child_by_field_name("destination")?, src);
    (!DISCARDED.contains(&destination.as_str())).then_some(destination)
}

fn resolves(token: &str) -> bool {
    named_family(token).is_some()
}

fn writes_only_markdown(body: &str, args: &[String]) -> bool {
    let paths: Vec<&str> = QUOTED_PATH
        .captures_iter(body)
        .filter_map(|found| found.get(1))
        .map(|found| found.as_str())
        .chain(args.iter().map(String::as_str))
        .collect();
    paths.iter().any(|path| is_markdown(path)) && !paths.iter().any(|path| resolves(path))
}

fn names_sed(word: &str) -> bool {
    word.rsplit('/').next() == Some("sed")
}

fn interpreter(name: &String) -> bool {
    let base = name.rsplit('/').next().unwrap_or(name);
    INTERPRETERS.contains(&base)
}

fn text(node: Node, src: &[u8]) -> String {
    String::from_utf8_lossy(&src[node.start_byte()..node.end_byte()]).into_owned()
}

fn unquote(node: Node, src: &[u8]) -> String {
    let raw = text(node, src);
    match node.kind() {
        "raw_string" => raw.trim_matches('\'').to_string(),
        "string" => raw.trim_matches('"').to_string(),
        _ => raw,
    }
}
