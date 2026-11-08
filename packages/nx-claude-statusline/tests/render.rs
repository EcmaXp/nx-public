use nx_claude_statusline::{branch, render};
use serde_json::json;

#[test]
fn every_segment_and_the_bare_minimum() {
    let full = json!({
        "model": {"display_name": "Fable 5.1"},
        "effort": {"level": "high"},
        "workspace": {"current_dir": "/home/x/ws/worktrees/task/repo/src"},
        "context_window": {"used_percentage": 41.6},
        "rate_limits": {"five_hour": {"used_percentage": 12.0}, "seven_day": {"used_percentage": 67.5}}
    });
    assert_eq!(
        render(&full, "/home/x", Some("main")),
        "Fable 5.1 (high) | ~/ws/worktrees/task/repo/src (main) | wt:task | ctx:42% | 5h:12% | 7d:68%"
    );
    let bare = json!({"model": {"display_name": "Opus"}, "workspace": {"current_dir": "/srv"}});
    assert_eq!(render(&bare, "/home/x", None), "Opus | /srv");
}

#[test]
fn branch_reads_head_through_a_git_dir_or_a_gitdir_file() {
    let root = std::env::temp_dir().join("nx-claude-statusline-branch");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("plain/.git")).unwrap();
    std::fs::write(root.join("plain/.git/HEAD"), "ref: refs/heads/main\n").unwrap();
    assert_eq!(branch(&root.join("plain")).as_deref(), Some("main"));

    let wt_git = root.join("bare/worktrees/task");
    std::fs::create_dir_all(&wt_git).unwrap();
    std::fs::create_dir_all(root.join("wt/sub")).unwrap();
    std::fs::write(wt_git.join("HEAD"), "ref: refs/heads/feat/x\n").unwrap();
    std::fs::write(
        root.join("wt/.git"),
        format!("gitdir: {}\n", wt_git.display()),
    )
    .unwrap();
    assert_eq!(branch(&root.join("wt/sub")).as_deref(), Some("feat/x"));

    std::fs::write(wt_git.join("HEAD"), "0123456789abcdef\n").unwrap();
    assert_eq!(branch(&root.join("wt/sub")), None);
}
