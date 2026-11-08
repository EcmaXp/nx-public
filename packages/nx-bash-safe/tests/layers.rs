//! How the policy layers are discovered.
//!
//! The layers are declared in nix and handed over as a manifest: one absolute
//! directory per line, lowest precedence first. Two of these tests pin failures
//! that were real rather than imagined — a duplicate directory used to report
//! dozens of phantom collisions, and an over-long file list used to be
//! truncated in complete silence, which reads exactly like a policy that never
//! mentioned the missing commands.

use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_nx-bash-safe");

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("nx-bash-safe-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create scratch");
        Self(dir)
    }

    fn layer(&self, name: &str, body: &str) -> PathBuf {
        let dir = self.0.join(name);
        std::fs::create_dir_all(&dir).expect("create layer");
        std::fs::write(dir.join("policy.toml"), body).expect("write layer");
        dir
    }

    fn manifest(&self, dirs: &[&Path]) -> PathBuf {
        let path = self.0.join("manifest");
        let text: String = dirs
            .iter()
            .map(|dir| format!("{}\n", dir.display()))
            .collect();
        std::fs::write(&path, text).expect("write manifest");
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Run `validate` with an explicit manifest and no home, and return its stdout.
fn validate(manifest: Option<&Path>, extra: &[&str]) -> String {
    let mut command = Command::new(BIN);
    command
        .arg("validate")
        .args(extra)
        .env("HOME", "/nonexistent");
    match manifest {
        Some(path) => command.env("NX_BASH_SAFE_LAYERS", path),
        None => command.env_remove("NX_BASH_SAFE_LAYERS"),
    };
    let output = command.output().expect("run validate");
    String::from_utf8(output.stdout).expect("utf-8")
}

fn field<'a>(text: &'a str, key: &str) -> &'a str {
    text.lines()
        .find_map(|line| line.strip_prefix(key))
        .unwrap_or_else(|| panic!("no {key} line in:\n{text}"))
        .trim()
}

const READER: &str = r#"
version = 1
[[command]]
name = "mytool"
kind = "reader"
"#;

const OTHER: &str = r#"
version = 1
[[command]]
name = "othertool"
kind = "reader"
"#;

#[test]
fn the_manifest_decides_which_layers_load_and_in_what_order() {
    let scratch = Scratch::new("order");
    let first = scratch.layer("first", READER);
    let second = scratch.layer("second", OTHER);
    let manifest = scratch.manifest(&[&first, &second]);

    let text = validate(Some(&manifest), &[]);
    assert_eq!(field(&text, "files:"), "2 (manifest)");
    assert_eq!(field(&text, "rules:"), "2");
    assert_eq!(field(&text, "discarded:"), "none");
}

/// The earlier definition survives a name collision, which is why the layer
/// that must never lose one is declared first. Without this, a later layer
/// could restate a command and quietly relax it.
#[test]
fn an_earlier_layer_wins_a_collision_and_the_later_one_is_reported() {
    let scratch = Scratch::new("collision");
    let first = scratch.layer("first", READER);
    let second = scratch.layer("second", READER);
    let manifest = scratch.manifest(&[&first, &second]);

    let text = validate(Some(&manifest), &[]);
    assert_eq!(field(&text, "rules:"), "1");
    assert!(
        text.contains("mytool is already defined"),
        "the collision must name the offender:\n{text}"
    );
    assert!(
        text.contains(&second.display().to_string()),
        "the later layer is the offender, not the earlier one:\n{text}"
    );
}

/// A packaged build runs its tests before the install step that would create a
/// manifest, and a checkout that has not been switched has none either. Both
/// must still see the baseline.
#[test]
fn an_absent_manifest_falls_back_to_the_shipped_baseline() {
    let text = validate(None, &[]);
    assert_eq!(field(&text, "files:"), "5 (baseline fallback, no manifest)");
    assert!(
        field(&text, "rules:").parse::<usize>().expect("a count") > 0,
        "the baseline must still load:\n{text}"
    );
}

/// Naming the same directory twice is what comparing two policies looks like.
/// It used to report every rule in the second copy as already defined.
#[test]
fn a_directory_named_twice_loads_once() {
    let scratch = Scratch::new("dup");
    let layer = scratch.layer("only", READER);
    let manifest = scratch.manifest(&[&layer, &layer]);

    let text = validate(Some(&manifest), &["--policy", &layer.display().to_string()]);
    assert_eq!(field(&text, "files:"), "1 (manifest)");
    assert_eq!(field(&text, "discarded:"), "none");
}

/// Truncation used to be invisible, which is the worst way for a policy to go
/// missing: it looks identical to a policy that never granted anything.
#[test]
fn more_files_than_the_cap_is_reported_rather_than_dropped_quietly() {
    let scratch = Scratch::new("cap");
    let dir = scratch.0.join("many");
    std::fs::create_dir_all(&dir).expect("create layer");
    for index in 0..20 {
        std::fs::write(
            dir.join(format!("{index:02}.toml")),
            format!("version = 1\n[[command]]\nname = \"tool{index}\"\nkind = \"reader\"\n"),
        )
        .expect("write");
    }
    let manifest = scratch.manifest(&[&dir]);

    let text = validate(Some(&manifest), &[]);
    assert!(
        text.contains("more than"),
        "truncation must be reported:\n{text}"
    );
}
