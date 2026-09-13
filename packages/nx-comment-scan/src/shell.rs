use std::cell::RefCell;

use tree_sitter::{Node, Parser};

use crate::{HASH, Kind, Report, SHELL_SAMPLE_LIMIT, is_comment, marked, named_family};

const MAX_DEPTH: usize = 200;
const BOUNDARIES: &[char] = &['\'', '"', '\\'];
const DISCARDED: &[&str] = &["/dev/null", "/dev/stdout", "/dev/stderr", "/dev/tty"];
const WRITE_OPERATORS: &[&str] = &[">", ">>", "&>", "&>>", ">|"];
const INPLACE_TOOLS: &[&str] = &["sed", "perl"];
const INTERPRETERS: &[&str] = &[
    "python", "python3", "uv", "ruby", "perl", "node", "bash", "sh", "zsh", "fish",
];
const SITE_NODES: &[&str] = &["redirected_statement", "pipeline", "command"];

thread_local! {
    static PARSER: RefCell<Parser> = RefCell::new({
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter::Language::new(tree_sitter_bash::LANGUAGE))
            .expect("the pinned bash grammar must load");
        parser
    });
}

#[derive(Default)]
struct Site {
    targets: Vec<String>,
    bodies: Vec<String>,
    names: Vec<String>,
    args: Vec<String>,
    inplace: Vec<(String, Vec<String>)>,
}

pub(crate) fn scan(command: &str) -> Vec<Report> {
    if !marked(command) {
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
    } else if INPLACE_TOOLS.contains(&base.as_str()) && args.iter().any(|arg| arg.starts_with("-i"))
    {
        if let Some(position) = args.iter().rposition(|arg| resolves(arg)) {
            let target = args[position].clone();
            let scripts = args
                .iter()
                .enumerate()
                .filter(|(index, arg)| *index != position && !arg.starts_with('-'))
                .map(|(_, arg)| arg.clone())
                .collect();
            site.inplace.push((target, scripts));
            return;
        }
    }
    site.args.extend(args);
}

fn report(site: Site, out: &mut Vec<Report>) {
    for (target, scripts) in &site.inplace {
        let Some(prefixes) = named_family(target) else {
            continue;
        };
        push(out, Kind::ShellFile, Some(target), fuzzy(scripts, prefixes));
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

fn exact(lines: &[String], prefixes: &[&str]) -> (usize, Vec<String>) {
    let mut added = 0;
    let mut samples = Vec::new();
    for line in lines.iter().flat_map(|arg| arg.lines()) {
        if is_comment(line, prefixes) {
            added += 1;
            samples.push(line.trim().to_string());
        }
    }
    (added, samples)
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
