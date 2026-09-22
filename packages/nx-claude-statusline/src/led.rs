use std::path::{Path, PathBuf};

use serde_json::Value;

pub fn band(pct: f64) -> &'static str {
    match pct {
        p if p >= 85.0 => "#FF0000",
        p if p >= 70.0 => "#FF6A00",
        p if p >= 50.0 => "#FFD000",
        _ => "#00FF66",
    }
}

pub fn program(state: &str, meters: &[&str], brightness: u8) -> String {
    let (on, off) = match state {
        "Working" => ("#00E5FF", "#000000"),
        "Ask" => ("#FF3A00", "#000000"),
        "Done" => ("#00FF66", "#00FF66"),
        _ => ("#002020", "#000000"),
    };
    let mut lines = vec![format!("brightness {brightness}")];
    let mut led0 = [on, off].into_iter().cycle();
    let mut push = |color: &str, ms: u32| {
        lines.push(format!(
            "0:{} 1:{color} {ms}ms ease",
            led0.next().unwrap_or(on)
        ));
    };
    for color in meters {
        for _ in 0..4 {
            push(color, 400);
        }
        push("#000000", 200);
    }
    push("#000000", 400);
    push("#000000", 400);
    lines.push("repeat".into());
    lines.join("\n") + "\n"
}

pub fn agent_state(log_tail: &str) -> Option<&'static str> {
    log_tail
        .lines()
        .rev()
        .find_map(|line| match line.rsplit_once(" state=") {
            Some((_, "Idle")) => Some("Idle"),
            Some((_, "Working")) => Some("Working"),
            Some((_, "Ask")) => Some("Ask"),
            Some((_, "Done")) => Some("Done"),
            _ => None,
        })
}

fn last_agent_state(log: &Path) -> &'static str {
    let len = std::fs::metadata(log).map(|m| m.len()).unwrap_or(0);
    let mut n = 64 * 1024;
    loop {
        if let Some(state) = agent_state(&tail_bytes(log, n)) {
            return state;
        }
        if n >= len {
            return "Idle";
        }
        n *= 4;
    }
}

pub fn sync(payload: &Value, home: &str) -> Option<()> {
    let (target, brightness) = custom_device(Path::new(home))?;
    let meters: Vec<&str> = ["/rate_limits/five_hour", "/rate_limits/seven_day"]
        .iter()
        .map(|p| {
            payload
                .pointer(&format!("{p}/used_percentage"))
                .and_then(Value::as_f64)
                .map_or("#000000", band)
        })
        .collect();
    let log = Path::new(home).join(".local/state/sidepulse/agent-monitor/status-bar.out.log");
    let text = program(last_agent_state(&log), &meters, brightness);
    if std::fs::read_to_string(&target).ok().as_deref() != Some(&text) {
        std::fs::write(&target, text).ok()?;
    }
    Some(())
}

fn custom_device(home: &Path) -> Option<(PathBuf, u8)> {
    let settings = home.join(".config/sidepulse/agent-monitor/settings.json");
    let settings: Value = serde_json::from_str(&std::fs::read_to_string(settings).ok()?).ok()?;
    settings.get("devices")?.as_array()?.iter().find_map(|d| {
        if d.get("led_display").and_then(Value::as_str) != Some("custom") {
            return None;
        }
        let target = Path::new(d.get("path")?.as_str()?).join("LEDS.LED");
        if !target.exists() {
            return None;
        }
        let brightness = d
            .get("brightness")
            .and_then(Value::as_u64)
            .unwrap_or(255)
            .min(255) as u8;
        Some((target, brightness))
    })
}

fn tail_bytes(path: &Path, n: u64) -> String {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut file) = std::fs::File::open(path) else {
        return String::new();
    };
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let _ = file.seek(SeekFrom::Start(len.saturating_sub(n)));
    let mut buf = Vec::new();
    let _ = file.read_to_end(&mut buf);
    String::from_utf8_lossy(&buf).into_owned()
}
