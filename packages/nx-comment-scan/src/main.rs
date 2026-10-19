use std::io::Read;

use nx_comment_scan::{output, scan};
use serde_json::Value;

fn main() {
    if let Some(line) = report() {
        println!("{line}");
    }
}

fn report() -> Option<String> {
    let mut raw = String::new();
    std::io::stdin().read_to_string(&mut raw).ok()?;
    let payload: Value = serde_json::from_str(raw.trim()).ok()?;
    let reports = scan(&payload);
    (!reports.is_empty()).then(|| output(&reports).to_string())
}
