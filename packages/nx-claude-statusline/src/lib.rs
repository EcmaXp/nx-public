pub mod led;

use std::path::Path;

use serde_json::Value;

pub fn render(payload: &Value, home: &str, branch: Option<&str>) -> String {
    let text = |pointer: &str| payload.pointer(pointer).and_then(Value::as_str);
    let dir = text("/workspace/current_dir").unwrap_or("");
    let mut out = text("/model/display_name").unwrap_or("?").to_string();
    if let Some(level) = text("/effort/level") {
        out.push_str(&format!(" ({level})"));
    }
    out.push_str(&format!(" | {}", tilde(dir, home)));
    if let Some(branch) = branch {
        out.push_str(&format!(" ({branch})"));
    }
    if let Some(name) = worktree(dir) {
        out.push_str(&format!(" | wt:{name}"));
    }
    if let Some(pct) = payload
        .pointer("/context_window/used_percentage")
        .and_then(Value::as_f64)
    {
        out.push_str(&format!(" | ctx:{}%", pct.round()));
    }
    for (label, pointer) in [
        ("5h", "/rate_limits/five_hour"),
        ("7d", "/rate_limits/seven_day"),
    ] {
        if let Some(pct) = payload
            .pointer(&format!("{pointer}/used_percentage"))
            .and_then(Value::as_f64)
        {
            out.push_str(&format!(" | {label}:{}%", pct.round()));
        }
    }
    out
}

pub fn branch(dir: &Path) -> Option<String> {
    let dot_git = dir
        .ancestors()
        .map(|d| d.join(".git"))
        .find(|p| p.exists())?;
    let git_dir = if dot_git.is_file() {
        let text = std::fs::read_to_string(&dot_git).ok()?;
        dot_git.parent()?.join(text.strip_prefix("gitdir:")?.trim())
    } else {
        dot_git
    };
    let head = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    Some(head.trim().strip_prefix("ref: refs/heads/")?.to_string())
}

fn tilde(dir: &str, home: &str) -> String {
    match dir.strip_prefix(home) {
        Some(rest) if !home.is_empty() && (rest.is_empty() || rest.starts_with('/')) => {
            format!("~{rest}")
        }
        _ => dir.to_string(),
    }
}

fn worktree(dir: &str) -> Option<&str> {
    let (_, rest) = dir.split_once("/worktrees/")?;
    let (name, _) = rest.split_once('/')?;
    Some(name)
}
