use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Output};

use nx_vibe_island::candidates;

const BIN: &str = env!("CARGO_BIN_EXE_nx-vibe-island");

fn stub(dir: &str, body: &str) -> String {
    let path = format!("{dir}/bridge");
    std::fs::create_dir_all(dir).unwrap();
    let mut file = std::fs::File::create(&path).unwrap();
    write!(file, "#!/bin/sh\n{body}\n").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn run(bridge: &str, args: &[&str]) -> Output {
    Command::new(BIN)
        .args(args)
        .env("NX_VIBE_ISLAND_BIN", bridge)
        .output()
        .unwrap()
}

#[test]
fn candidates_try_the_app_before_the_launcher() {
    assert_eq!(
        candidates("/home/x"),
        [
            "/Applications/Vibe Island.app/Contents/Helpers/vibe-island-bridge",
            "/Applications/vibe-island.app/Contents/Helpers/vibe-island-bridge",
            "/home/x/Applications/Vibe Island.app/Contents/Helpers/vibe-island-bridge",
            "/home/x/.vibe-island/bin/vibe-island-bridge",
        ]
    );
}

#[test]
fn argv_reaches_the_bridge() {
    let dir = std::env::temp_dir().join("nx-vibe-island-argv");
    let bridge = stub(dir.to_str().unwrap(), r#"echo "$@""#);
    let out = run(&bridge, &["--source", "claude"]);

    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "--source claude"
    );
}

#[test]
fn a_missing_bridge_exits_quietly() {
    let out = run("/nonexistent/vibe-island-bridge", &["--source", "claude"]);

    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.is_empty());
    assert!(out.stderr.is_empty());
}

#[test]
fn stdin_survives_the_exec() {
    let dir = std::env::temp_dir().join("nx-vibe-island-stdin");
    let bridge = stub(dir.to_str().unwrap(), "cat");

    let mut child = Command::new(BIN)
        .env("NX_VIBE_ISLAND_BIN", &bridge)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    write!(
        child.stdin.take().unwrap(),
        r#"{{"hook_event_name":"Stop"}}"#
    )
    .unwrap();
    let out = child.wait_with_output().unwrap();

    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        r#"{"hook_event_name":"Stop"}"#
    );
}
