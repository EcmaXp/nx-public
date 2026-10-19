const HELPER: &str = "Contents/Helpers/vibe-island-bridge";

// The app bundles are the ones `~/.vibe-island/bin/vibe-island-bridge` probes before
// it falls back to `mdfind`, so that launcher stays last rather than being replaced.
pub fn candidates(home: &str) -> Vec<String> {
    [
        "/Applications/Vibe Island.app",
        "/Applications/vibe-island.app",
        &format!("{home}/Applications/Vibe Island.app"),
    ]
    .iter()
    .map(|app| format!("{app}/{HELPER}"))
    .chain([format!("{home}/.vibe-island/bin/vibe-island-bridge")])
    .collect()
}
