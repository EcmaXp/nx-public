//! Why a command was allowed, or where the proof ran out.
//!
//! This is the feedback loop that makes an editable policy usable. Without it,
//! "my new rule did not fire" is guesswork against a binary whose entire design
//! is to say nothing.

use crate::parse;
use crate::policy::Policy;

pub fn explain(command: &str, policy: &Policy, show_tree: bool) -> i32 {
    let dump = parse::dump_source(command.as_bytes(), policy);

    if show_tree {
        println!("parse:");
        println!("  has_error   {}", dump.errored);
        println!("  cuts        {:?}", dump.cuts);
    }

    println!("segments:");
    if dump.segments.is_empty() {
        println!("  (none: nothing classifiable was reached)");
    }
    for segment in &dump.segments {
        let rendered: Vec<String> = segment
            .iter()
            .map(|token| token.replace(parse::OPAQUE, parse::OPAQUE_LABEL))
            .collect();
        let verdict =
            crate::policy::classify_segment(segment, policy).unwrap_or_else(|| "PROMPT".to_owned());
        println!("  {:<40} {verdict}", rendered.join(" "));
    }

    match parse::explain_source(command.as_bytes(), policy) {
        Ok(reason) => {
            println!("\nallow: {reason}");
            0
        }
        Err(prompt) => {
            println!("\nprompt: {prompt}");
            1
        }
    }
}
