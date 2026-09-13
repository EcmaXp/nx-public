//! The abort channel: why a command could not be proven safe.
//!
//! The python implementation raised an exception from anywhere in a recursive
//! tree walk and caught it in exactly one place. `Safe<T>` is that control flow
//! as a `Result`, with the same single catch site in `classify_source`.
//!
//! Aborting is the *common* path (most commands prompt), so `detail` is only
//! filled in when someone asked for an explanation. The hook path never
//! allocates a reason string for a command it is about to stay silent about.

use std::fmt;

/// Recursion bound for the tree walk.
///
/// This is not decoration. Python unwound a `RecursionError` and turned it into
/// a prompt; a Rust stack overflow is SIGSEGV, which `catch_unwind` cannot
/// catch, so the hook would die on a signal instead of exiting 0. The deepest
/// tree in the frozen corpus is 27 levels, so 200 is roughly 7x real traffic
/// while still being far below anything that threatens the stack.
pub const MAX_NODE_DEPTH: u32 = 200;

/// How many times a mis-parse may be re-split before giving up.
pub const MAX_SPLIT_DEPTH: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptKind {
    /// A statement node the walker has no explicit branch for.
    StatementNode,
    /// An argument node the walker has no explicit branch for.
    ArgumentNode,
    /// A redirect node the walker has no explicit branch for.
    RedirectNode,
    /// A write redirect whose target is not /dev/null or verified tmp.
    RedirectTarget,
    /// A redirect target that is an expansion, so its destination is unknown.
    RedirectOpaque,
    /// The command name itself is an expansion, so what runs is unknown.
    CommandNameExpansion,
    /// `>(...)` feeds a command rather than producing a value.
    ProcessSubstOutput,
    /// A backgrounded command.
    Backgrounded,
    /// An expansion reached a command whose safety rests on per-flag
    /// validation, where it could smuggle a flag past classification.
    ExpansionInFlagValidatedCommand,
    /// The policy layer had no rule that accepts this segment.
    Unclassified,
    /// The command did not parse as bash at all, which is where fish syntax
    /// pasted into the Bash tool lands.
    ParseError,
    /// The walk exceeded MAX_NODE_DEPTH or MAX_SPLIT_DEPTH.
    TooDeep,
}

impl PromptKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StatementNode => "statement-node",
            Self::ArgumentNode => "argument-node",
            Self::RedirectNode => "redirect-node",
            Self::RedirectTarget => "redirect-target",
            Self::RedirectOpaque => "redirect-opaque",
            Self::CommandNameExpansion => "command-name-expansion",
            Self::ProcessSubstOutput => "process-substitution-output",
            Self::Backgrounded => "backgrounded",
            Self::ExpansionInFlagValidatedCommand => "expansion-in-flag-validated-command",
            Self::Unclassified => "unclassified",
            Self::ParseError => "parse-error",
            Self::TooDeep => "too-deep",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Prompt {
    pub kind: PromptKind,
    pub detail: Option<Box<str>>,
}

impl Prompt {
    pub fn new(kind: PromptKind) -> Self {
        Self { kind, detail: None }
    }

    /// Attach a human-readable detail, but only when explaining: `f` is not
    /// called otherwise, so the hook path stays allocation-free here.
    pub fn detailed(kind: PromptKind, explain: bool, f: impl FnOnce() -> String) -> Self {
        Self {
            kind,
            detail: explain.then(|| f().into_boxed_str()),
        }
    }
}

impl fmt::Display for Prompt {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(out, "{} ({detail})", self.kind.as_str()),
            None => out.write_str(self.kind.as_str()),
        }
    }
}

/// `Ok` means "still provable"; `Err` means the command falls through to a
/// prompt. There is no third state: the hook never denies.
pub type Safe<T> = Result<T, Prompt>;
