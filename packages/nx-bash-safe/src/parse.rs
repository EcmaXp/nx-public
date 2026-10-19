//! The tree walk: a bash parse tree projected down to flat token segments.
//!
//! One rule governs the whole file: **every node is consumed by an explicit
//! branch, and an unrecognized one aborts.** Skipping a node is not a harmless
//! omission here. In `cat <<'PY' | uv run python -` the entire trailing
//! pipeline hangs off the heredoc node, so a walker that reads only the
//! children it expects sees a bare `cat` and allows a command that runs python.
//!
//! Two more grammar facts the walk defends against:
//!
//! - A `command` node whose arguments span a bare newline is a mis-parse, and
//!   it is reported with `has_error == false`. It makes the next line's tokens
//!   look like arguments of the previous command, so the source is cut at that
//!   newline and each piece is classified on its own.
//! - `$VAR`, `$(...)`, backticks and `<(...)` yield a value the source does not
//!   reveal. Such a value may not reach a command whose safety rests on
//!   per-flag validation, because it can smuggle a flag past classification
//!   (`rg $F` with `F='--pre evil'`).

use std::cell::RefCell;
use std::collections::HashMap;

use tree_sitter::{Node, Parser};

use crate::env;
use crate::policy::Policy;
use crate::prompt::{MAX_NODE_DEPTH, MAX_SPLIT_DEPTH, Prompt, PromptKind, Safe};

/// Stands in for a value the source does not reveal.
pub const OPAQUE: &str = "\u{0}expansion\u{0}";

/// How the sentinel is rendered in a segment dump: its NUL bytes survive JSON
/// but make the artifact painful to read, and only the label has to agree.
pub const OPAQUE_LABEL: &str = "<expansion>";

/// Statement nodes the walker knows how to take apart. Everything absent from
/// this list prompts, `function_definition` and `case_statement` on purpose.
const STATEMENT_NODES: &[&str] = &[
    "program",
    "list",
    "pipeline",
    "subshell",
    "compound_statement",
    "redirected_statement",
    "negated_command",
    "while_statement",
    "if_statement",
    "elif_clause",
    "else_clause",
    "do_group",
];

const LITERAL_NODES: &[&str] = &[
    "word",
    "number",
    "raw_string",
    "ansi_c_string",
    "regex",
    "extglob_pattern",
    "variable_name",
];

const EXPANSION_NODES: &[&str] = &["simple_expansion", "expansion", "arithmetic_expansion"];

const REDIRECT_NODES: &[&str] = &["file_redirect", "heredoc_redirect", "herestring_redirect"];

/// Write targets that discard, so writing to them is not a write.
const DUP_TARGETS: &[&str] = &["/dev/null", "/dev/stdout", "/dev/stderr", "/dev/tty"];

const TRUSTED_TMP_VARS: &[&str] = &["TMPDIR", "SCRATCHPAD"];

thread_local! {
    static PARSER: RefCell<Parser> = RefCell::new({
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter::Language::new(tree_sitter_bash::LANGUAGE))
            .expect("the pinned bash grammar must load");
        parser
    });
}

/// What a parse looks like to the classifier, and nothing more.
#[derive(Debug, Default)]
pub struct Dump {
    pub errored: bool,
    pub cuts: Vec<usize>,
    pub segments: Vec<Vec<String>>,
}

struct Walker<'a> {
    src: &'a [u8],
    policy: &'a Policy,
    explain: bool,
    reasons: Vec<String>,
    /// Same-line literal assignments, folded into later `$NAME` uses.
    bindings: HashMap<String, String>,
    /// When present, segments are recorded and classification is skipped, so a
    /// dump depends on the grammar alone and can be compared before any policy
    /// exists to classify with.
    sink: Option<&'a mut Vec<Vec<String>>>,
}

/// One word the shell passes through verbatim: nothing re-splits or globs.
fn foldable_value(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || "_@%+=:,./~-".contains(ch))
}

/// `$NAME` or `${NAME}` with no operator, else `None`.
fn plain_var_name(kind: &str, text: &str) -> Option<String> {
    let name = match kind {
        "simple_expansion" => text.strip_prefix('$')?,
        "expansion" => text.strip_prefix("${")?.strip_suffix('}')?,
        _ => return None,
    };
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return None,
    }
    chars
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        .then(|| name.to_owned())
}

impl<'a> Walker<'a> {
    fn text(&self, node: Node) -> String {
        String::from_utf8_lossy(&self.src[node.start_byte()..node.end_byte()]).into_owned()
    }

    fn deeper(&self, depth: u32) -> Safe<u32> {
        // Python unwound a RecursionError into a prompt. Rust would take a
        // SIGSEGV that catch_unwind cannot catch, killing the hook on a signal
        // rather than exiting 0, so the bound has to be explicit.
        if depth >= MAX_NODE_DEPTH {
            return Err(Prompt::new(PromptKind::TooDeep));
        }
        Ok(depth + 1)
    }

    /// One argument, as (text, opaque).
    fn token(&mut self, node: Node, depth: u32) -> Safe<(String, bool)> {
        let depth = self.deeper(depth)?;
        let kind = node.kind();

        if LITERAL_NODES.contains(&kind) {
            let text = self.text(node);
            if kind == "raw_string" {
                let mut chars = text.chars();
                chars.next();
                chars.next_back();
                return Ok((chars.as_str().to_owned(), false));
            }
            return Ok((text, false));
        }

        if EXPANSION_NODES.contains(&kind) {
            let text = self.text(node);
            if let Some(name) = plain_var_name(kind, &text) {
                if TRUSTED_TMP_VARS.contains(&name.as_str()) && env::env_tmp_ok(&name) {
                    // Keeping the literal `$TMPDIR/...` lets the redirect check
                    // reason about the path instead of giving up on it.
                    return Ok((text, false));
                }
                if let Some(value) = self.bindings.get(&name) {
                    return Ok((value.clone(), false));
                }
                if name == "HOME"
                    && let Ok(home) = std::env::var("HOME")
                    && home.starts_with('/')
                    && foldable_value(&home)
                {
                    return Ok((home, false));
                }
            }
            return Ok((OPAQUE.to_owned(), true));
        }

        if matches!(kind, "string" | "concatenation" | "array") {
            let mut parts = String::new();
            let mut opaque = false;
            for index in 0..node.child_count() as u32 {
                let child = node.child(index).expect("child_count is authoritative");
                if !child.is_named() {
                    let punctuation = self.text(child);
                    if !matches!(punctuation.as_str(), "\"" | "'" | "$'" | "(" | ")") {
                        parts.push_str(&punctuation);
                    }
                    continue;
                }
                if child.kind() == "string_content" {
                    parts.push_str(&self.text(child));
                    continue;
                }
                let (text, child_opaque) = self.token(child, depth)?;
                parts.push_str(&text);
                opaque |= child_opaque;
            }
            return Ok((if opaque { OPAQUE.to_owned() } else { parts }, opaque));
        }

        if kind == "command_substitution" {
            // The inner command must classify as a pure read on its own before
            // its result counts as data.
            for index in 0..node.child_count() as u32 {
                let child = node.child(index).expect("child_count is authoritative");
                if child.is_named() {
                    self.walk(child, depth)?;
                }
            }
            return Ok((OPAQUE.to_owned(), true));
        }

        if kind == "process_substitution" {
            if self.text(node).starts_with(">(") {
                return Err(Prompt::new(PromptKind::ProcessSubstOutput));
            }
            for index in 0..node.child_count() as u32 {
                let child = node.child(index).expect("child_count is authoritative");
                if child.is_named() {
                    self.walk(child, depth)?;
                }
            }
            return Ok((OPAQUE.to_owned(), true));
        }

        Err(Prompt::detailed(
            PromptKind::ArgumentNode,
            self.explain,
            || kind.to_owned(),
        ))
    }

    /// `NAME=value` as a token; with `bind`, a foldable literal is remembered.
    fn assignment(&mut self, node: Node, depth: u32, bind: bool) -> Safe<(String, bool)> {
        let depth = self.deeper(depth)?;
        let mut opaque = false;
        let mut parts = String::new();
        for index in 1..node.child_count() as u32 {
            let child = node.child(index).expect("child_count is authoritative");
            if child.is_named() {
                let (text, child_opaque) = self.token(child, depth)?;
                parts.push_str(&text);
                opaque |= child_opaque;
            }
        }
        let name = node
            .child(0)
            .map(|child| self.text(child))
            .unwrap_or_default();
        self.bindings.remove(&name);
        if bind && !opaque && foldable_value(&parts) {
            self.bindings.insert(name.clone(), parts);
        }
        let value = if opaque { OPAQUE } else { "value" };
        Ok((format!("{name}={value}"), opaque))
    }

    /// Heredoc and herestring content is stdin data; only substitutions run.
    fn walk_data(&mut self, node: Node, depth: u32) -> Safe<()> {
        let depth = self.deeper(depth)?;
        for index in 0..node.child_count() as u32 {
            let child = node.child(index).expect("child_count is authoritative");
            if matches!(
                child.kind(),
                "command_substitution" | "process_substitution"
            ) {
                self.token(child, depth)?;
            } else {
                self.walk_data(child, depth)?;
            }
        }
        Ok(())
    }

    fn walk_redirect(&mut self, node: Node, depth: u32) -> Safe<()> {
        let depth = self.deeper(depth)?;
        let kind = node.kind();

        if kind == "herestring_redirect" {
            return self.walk_data(node, depth);
        }

        if kind == "heredoc_redirect" {
            for index in 0..node.child_count() as u32 {
                let child = node.child(index).expect("child_count is authoritative");
                if !child.is_named() || matches!(child.kind(), "heredoc_start" | "heredoc_end") {
                    continue;
                }
                match child.kind() {
                    "heredoc_body" => self.walk_data(child, depth)?,
                    other if REDIRECT_NODES.contains(&other) => self.walk_redirect(child, depth)?,
                    // A trailing pipeline nests in here. It is not decoration.
                    _ => self.walk(child, depth)?,
                }
            }
            return Ok(());
        }

        if kind != "file_redirect" {
            return Err(Prompt::detailed(
                PromptKind::RedirectNode,
                self.explain,
                || kind.to_owned(),
            ));
        }

        let mut operator = String::new();
        let mut targets = Vec::new();
        for index in 0..node.child_count() as u32 {
            let child = node.child(index).expect("child_count is authoritative");
            if !child.is_named() {
                operator = self.text(child);
            } else if child.kind() != "file_descriptor" {
                targets.push(child);
            }
        }
        if matches!(operator.as_str(), "<" | "<&" | "<>") {
            return Ok(());
        }
        for target in targets {
            if target.kind() == "number" && operator.ends_with('&') {
                continue; // 2>&1 and friends duplicate an fd, they open nothing
            }
            let (text, opaque) = self.token(target, depth)?;
            if opaque {
                return Err(Prompt::new(PromptKind::RedirectOpaque));
            }
            // A verified tmp target is the same contained-write class as
            // compiling a python file into tmp.
            if !DUP_TARGETS.contains(&text.as_str()) && !env::scratch_path(&text) {
                return Err(Prompt::detailed(
                    PromptKind::RedirectTarget,
                    self.explain,
                    || text.clone(),
                ));
            }
        }
        Ok(())
    }

    fn emit(&mut self, tokens: Vec<String>, opaque: bool) -> Safe<()> {
        if tokens.is_empty() {
            return Ok(());
        }
        if let Some(sink) = self.sink.as_deref_mut() {
            sink.push(tokens);
            return Ok(());
        }
        if opaque {
            let stripped = crate::policy::strip_wrappers(&tokens, self.policy);
            let name = stripped
                .first()
                .map(|token| basename(token))
                .unwrap_or_default();
            if !name.is_empty() && !self.policy.is_simple_reader(name) {
                return Err(Prompt::detailed(
                    PromptKind::ExpansionInFlagValidatedCommand,
                    self.explain,
                    || name.to_owned(),
                ));
            }
        }
        match crate::policy::classify_segment(&tokens, self.policy) {
            Some(reason) => {
                self.reasons.push(reason);
                Ok(())
            }
            None => Err(Prompt::detailed(
                PromptKind::Unclassified,
                self.explain,
                || tokens.iter().take(3).cloned().collect::<Vec<_>>().join(" "),
            )),
        }
    }

    fn walk_command(&mut self, node: Node, depth: u32) -> Safe<()> {
        let depth = self.deeper(depth)?;
        let mut tokens: Vec<String> = Vec::new();
        let mut opaque = false;
        for index in 0..node.child_count() as u32 {
            let child = node.child(index).expect("child_count is authoritative");
            if !child.is_named() {
                continue;
            }
            match child.kind() {
                "command_name" => {
                    let named = (0..child.child_count() as u32)
                        .filter_map(|i| child.child(i))
                        .find(|grandchild| grandchild.is_named());
                    let (text, name_opaque) = match named {
                        Some(grandchild) => self.token(grandchild, depth)?,
                        None => (self.text(child), false),
                    };
                    if name_opaque {
                        return Err(Prompt::new(PromptKind::CommandNameExpansion));
                    }
                    tokens.push(text);
                }
                "variable_assignment" => {
                    // Prefix assignments scope to this command alone.
                    let (text, child_opaque) = self.assignment(child, depth, false)?;
                    tokens.push(text);
                    opaque |= child_opaque;
                }
                other if REDIRECT_NODES.contains(&other) => self.walk_redirect(child, depth)?,
                _ => {
                    let (text, child_opaque) = self.token(child, depth)?;
                    tokens.push(text);
                    opaque |= child_opaque;
                }
            }
        }
        if tokens.first().is_some_and(|first| first == "read") {
            // `read` refills its variables from stdin at runtime.
            for token in tokens.iter().skip(1) {
                if !token.starts_with('-') {
                    self.bindings.remove(token);
                }
            }
        }
        self.emit(tokens, opaque)
    }

    fn walk(&mut self, node: Node, depth: u32) -> Safe<()> {
        let depth = self.deeper(depth)?;
        let kind = node.kind();

        match kind {
            "comment" => return Ok(()),
            "command" => return self.walk_command(node, depth),
            "variable_assignment" => {
                // Bind only in sequential top-level flow, never inside a branch.
                let bind = node
                    .parent()
                    .is_some_and(|parent| matches!(parent.kind(), "program" | "list"));
                let (text, _) = self.assignment(node, depth, bind)?;
                return self.emit(vec![text], false);
            }
            "variable_assignments" => {
                for index in 0..node.child_count() as u32 {
                    let child = node.child(index).expect("child_count is authoritative");
                    if child.is_named() {
                        self.walk(child, depth)?;
                    }
                }
                return Ok(());
            }
            "declaration_command" | "unset_command" => {
                let mut tokens = Vec::new();
                let mut opaque = false;
                for index in 0..node.child_count() as u32 {
                    let child = node.child(index).expect("child_count is authoritative");
                    if !child.is_named() {
                        tokens.push(self.text(child)); // export / unset / declare
                        continue;
                    }
                    let (text, child_opaque) = if child.kind() == "variable_assignment" {
                        self.assignment(child, depth, false)?
                    } else {
                        self.token(child, depth)?
                    };
                    tokens.push(text);
                    opaque |= child_opaque;
                }
                if tokens.first().is_some_and(|first| first == "unset") {
                    for token in tokens.iter().skip(1) {
                        self.bindings.remove(token);
                    }
                }
                return self.emit(tokens, opaque);
            }
            "test_command" => {
                for index in 0..node.child_count() as u32 {
                    let child = node.child(index).expect("child_count is authoritative");
                    if child.is_named() {
                        self.walk_data(child, depth)?; // a test evaluates, it does not exec
                    }
                }
                self.reasons.push("test".to_owned());
                return Ok(());
            }
            "for_statement" => {
                for index in 0..node.child_count() as u32 {
                    let child = node.child(index).expect("child_count is authoritative");
                    if !child.is_named() || child.kind() == "variable_name" {
                        if child.kind() == "variable_name" {
                            self.bindings.remove(&self.text(child));
                        }
                        continue; // the loop variable is bound, not executed
                    }
                    if child.kind() == "do_group" {
                        self.walk(child, depth)?;
                    } else {
                        self.token(child, depth)?; // iterated values are data
                    }
                }
                return Ok(());
            }
            _ => {}
        }

        if !STATEMENT_NODES.contains(&kind) {
            return Err(Prompt::detailed(
                PromptKind::StatementNode,
                self.explain,
                || kind.to_owned(),
            ));
        }
        for index in 0..node.child_count() as u32 {
            let child = node.child(index).expect("child_count is authoritative");
            if !child.is_named() {
                if self.text(child) == "&" {
                    return Err(Prompt::new(PromptKind::Backgrounded));
                }
                continue;
            }
            if REDIRECT_NODES.contains(&child.kind()) {
                self.walk_redirect(child, depth)?;
            } else {
                self.walk(child, depth)?;
            }
        }
        Ok(())
    }
}

fn basename(token: &str) -> &str {
    token.rsplit('/').next().unwrap_or(token)
}

/// Offsets of bare newlines sitting inside a command's argument list.
///
/// The gap between two sibling arguments cannot be inside a quoted string,
/// because quotes live inside the nodes. A newline there is therefore unquoted,
/// which in bash ends the command: its presence proves the parse is wrong and
/// marks where bash would have split.
fn misparse_cuts(node: Node, src: &[u8], cuts: &mut Vec<usize>, depth: u32) -> Safe<()> {
    if depth >= MAX_NODE_DEPTH {
        return Err(Prompt::new(PromptKind::TooDeep));
    }
    if node.kind() == "command" {
        for index in 1..node.child_count() as u32 {
            let left = node.child(index - 1).expect("child_count is authoritative");
            let right = node.child(index).expect("child_count is authoritative");
            let gap = &src[left.end_byte()..right.start_byte()];
            let mut escaped = false;
            for (offset, byte) in gap.iter().enumerate() {
                if escaped {
                    escaped = false;
                } else if *byte == b'\\' {
                    escaped = true; // a backslash-newline is a continuation
                } else if *byte == b'\n' {
                    cuts.push(left.end_byte() + offset);
                }
            }
        }
    }
    for index in 0..node.child_count() as u32 {
        let child = node.child(index).expect("child_count is authoritative");
        misparse_cuts(child, src, cuts, depth + 1)?;
    }
    Ok(())
}

fn sorted_unique(mut cuts: Vec<usize>) -> Vec<usize> {
    cuts.sort_unstable();
    cuts.dedup();
    cuts
}

/// Dedupe preserving first appearance, then join. The reason string is an
/// interface (plan mode splits it back apart), so both halves are load-bearing.
fn join_reasons(reasons: Vec<String>) -> Option<String> {
    let mut seen = Vec::with_capacity(reasons.len());
    for reason in reasons {
        if !seen.contains(&reason) {
            seen.push(reason);
        }
    }
    if seen.is_empty() {
        return None;
    }
    Some(seen.join(crate::REASON_JOIN))
}

fn parse_tree(src: &[u8]) -> Option<tree_sitter::Tree> {
    PARSER.with(|parser| parser.borrow_mut().parse(src, None))
}

fn run(
    src: &[u8],
    policy: &Policy,
    explain: bool,
    depth: u32,
    sink: &mut Option<&mut Vec<Vec<String>>>,
    why: &mut Option<Prompt>,
) -> Option<String> {
    let tree = parse_tree(src)?;
    let root = tree.root_node();
    // A parse error prompts. That is where fish syntax pasted into the Bash
    // tool lands, and it is also the fail-safe gate: a tree we could not build
    // is a tree we cannot make claims about.
    if root.has_error() {
        why.get_or_insert_with(|| Prompt::new(PromptKind::ParseError));
        return None;
    }

    let mut cuts = Vec::new();
    if let Err(prompt) = misparse_cuts(root, src, &mut cuts, 0) {
        why.get_or_insert(prompt);
        return None;
    }
    let cuts = sorted_unique(cuts);

    if !cuts.is_empty() {
        if depth >= MAX_SPLIT_DEPTH {
            why.get_or_insert_with(|| Prompt::new(PromptKind::TooDeep));
            return None;
        }
        let mut reasons = Vec::new();
        let mut start = 0usize;
        for cut in cuts.into_iter().chain(std::iter::once(src.len())) {
            let piece = &src[start.min(src.len())..cut.min(src.len())];
            start = cut + 1;
            if piece.iter().all(|byte| byte.is_ascii_whitespace()) {
                continue;
            }
            let reason = run(piece, policy, explain, depth + 1, sink, why)?;
            reasons.push(reason);
        }
        return join_reasons(reasons);
    }

    let mut walker = Walker {
        src,
        policy,
        explain,
        reasons: Vec::new(),
        bindings: HashMap::new(),
        sink: sink.as_deref_mut(),
    };
    if let Err(prompt) = walker.walk(root, 0) {
        why.get_or_insert(prompt);
        return None;
    }
    let reasons = walker.reasons;
    join_reasons(reasons)
}

/// Classify a command line: `Some(reason)` to allow, `None` to prompt.
pub fn classify_source(src: &[u8], policy: &Policy, explain: bool) -> Option<String> {
    run(src, policy, explain, 0, &mut None, &mut None)
}

/// Classify, keeping the reason it could not be proven safe.
///
/// The first abort wins, which is also the one the walk actually stopped on.
pub fn explain_source(src: &[u8], policy: &Policy) -> Result<String, Prompt> {
    let mut why = None;
    match run(src, policy, true, 0, &mut None, &mut why) {
        Some(reason) => Ok(reason),
        None => Err(why.unwrap_or_else(|| Prompt::new(PromptKind::Unclassified))),
    }
}

/// Project a command line down to what classification would see.
pub fn dump_source(src: &[u8], policy: &Policy) -> Dump {
    let Some(tree) = parse_tree(src) else {
        return Dump::default();
    };
    let root = tree.root_node();
    let errored = root.has_error();
    let mut cuts = Vec::new();
    if !errored {
        // run() bails on an errored tree before ever cutting, so computing cuts
        // here would report offsets the classifier never saw.
        let _ = misparse_cuts(root, src, &mut cuts, 0);
    }
    let mut segments = Vec::new();
    if !errored {
        let mut sink = Some(&mut segments);
        let _ = run(src, policy, false, 0, &mut sink, &mut None);
    }
    Dump {
        errored,
        cuts: sorted_unique(cuts),
        segments,
    }
}
