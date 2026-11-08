//! Which paths count as tmp, and therefore which writes are containable.
//!
//! The allow surface is pure reads plus writes provably confined to tmp. That
//! second half rests on this file, so it resolves symlinks rather than trusting
//! a prefix: `$TMPDIR` is an environment variable, and a hostile one pointing at
//! the repo would otherwise turn "write to tmp" into "write anywhere".

use std::path::{Path, PathBuf};

const SCRATCH_PREFIXES: &[&str] = &["/tmp/", "/private/tmp/"];

const SCRATCH_ENV_PREFIXES: &[(&str, &str)] = &[
    ("$TMPDIR/", "TMPDIR"),
    ("${TMPDIR}/", "TMPDIR"),
    ("$SCRATCHPAD/", "SCRATCHPAD"),
    ("${SCRATCHPAD}/", "SCRATCHPAD"),
];

/// Where a *resolved* path must live for the variable to be trusted. The
/// `/private` twins are macOS: /tmp and /var are symlinks into it.
const REAL_TMP_PREFIXES: &[&str] = &[
    "/tmp/",
    "/private/tmp/",
    "/var/folders/",
    "/private/var/folders/",
];

/// Resolve symlinks as far as the path exists, keeping the rest verbatim.
///
/// `std::fs::canonicalize` fails outright on a missing path, but python's
/// `os.path.realpath` resolves the existing prefix and appends the remainder,
/// and the prefix is the part that matters: on macOS it is what turns `/tmp`
/// into `/private/tmp`.
fn realpath(value: &str) -> PathBuf {
    let path = Path::new(value);
    if let Ok(resolved) = std::fs::canonicalize(path) {
        return resolved;
    }
    for ancestor in path.ancestors().skip(1) {
        if let Ok(resolved) = std::fs::canonicalize(ancestor)
            && let Ok(rest) = path.strip_prefix(ancestor)
        {
            return resolved.join(rest);
        }
    }
    path.to_path_buf()
}

/// Trust `$TMPDIR`/`$SCRATCHPAD` only if their live value resolves into tmp.
///
/// An unset variable is trusted: there is then nothing to expand, so no write
/// can land through it.
pub fn env_tmp_ok(var: &str) -> bool {
    let Some(value) = std::env::var_os(var) else {
        return true;
    };
    let Some(value) = value.to_str() else {
        return false;
    };
    if value.is_empty() {
        return true;
    }
    let resolved = realpath(value);
    let Some(resolved) = resolved.to_str() else {
        return false;
    };
    REAL_TMP_PREFIXES
        .iter()
        .any(|prefix| resolved.starts_with(prefix))
}

/// True when a write to this path is confined to tmp.
pub fn scratch_path(path: &str) -> bool {
    // `..` anywhere defeats the prefix test, so reject it outright rather than
    // trying to normalize a path that may not exist yet.
    if path.split('/').any(|part| part == "..") {
        return false;
    }
    if SCRATCH_PREFIXES.iter().any(|p| path.starts_with(p)) {
        return true;
    }
    for (prefix, var) in SCRATCH_ENV_PREFIXES {
        if path.starts_with(prefix) {
            return env_tmp_ok(var);
        }
    }
    false
}
