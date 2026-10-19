use std::os::unix::process::CommandExt;
use std::process::Command;

fn main() {
    let home = std::env::var("HOME").unwrap_or_default();
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let bins = match std::env::var("NX_VIBE_ISLAND_BIN") {
        Ok(bin) => vec![bin],
        Err(_) => nx_vibe_island::candidates(&home),
    };
    for bin in bins {
        let _ = Command::new(bin).args(&args).exec();
    }
}
