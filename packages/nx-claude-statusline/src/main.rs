use std::io::Read;
use std::path::Path;

use nx_claude_statusline::{branch, render};
use serde_json::Value;

fn main() {
    let mut raw = String::new();
    if std::io::stdin().read_to_string(&mut raw).is_err() {
        return;
    }
    let Ok(payload) = serde_json::from_str::<Value>(&raw) else {
        return;
    };
    let home = std::env::var("HOME").unwrap_or_default();
    let dir = payload
        .pointer("/workspace/current_dir")
        .and_then(Value::as_str)
        .unwrap_or("");
    let branch = branch(Path::new(dir));
    print!("{}", render(&payload, &home, branch.as_deref()));
    let _ = nx_claude_statusline::led::sync(&payload, &home);
}
