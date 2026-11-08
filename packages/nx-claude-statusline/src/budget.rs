use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

pub const SAME_WINDOW_SLACK: f64 = 300.0;
const ACCOUNTS: &str = "accounts";
const FIVE_HOUR: &str = "five_hour";
const SEVEN_DAY: &str = "seven_day";
const FLAT_POINTS: &str = "seven_day_flat_five_hour_points";
const DROPPED: &str = "seven_day_dropped";
const WRITTEN_BY: &str = "written_by";
const DAY_START: &str = "seven_day_day_start";
const DAY_S: f64 = 86_400.0;

pub fn snapshot(payload: &Value, now: u64) -> Option<Value> {
    let mut snap = payload.get("rate_limits")?.as_object()?.clone();
    snap.insert("captured_at".into(), json!(now));
    if let Some(id) = payload.get("session_id") {
        snap.insert("session_id".into(), id.clone());
    }
    Some(Value::Object(snap))
}

fn window_fields(value: &Value) -> Option<(f64, f64)> {
    let used = value.get("used_percentage")?.as_f64()?;
    let resets_at = value.get("resets_at")?.as_f64()?;
    Some((used, resets_at))
}

fn live(value: Option<&Value>, now: u64) -> Option<(&Value, f64, f64)> {
    let value = value?;
    let (used, resets_at) = window_fields(value)?;
    (resets_at > now as f64).then_some((value, used, resets_at))
}

pub fn merge_window(
    old: Option<&Value>,
    new: Option<&Value>,
    now: u64,
    same_writer: bool,
) -> Option<Value> {
    match (live(old, now), live(new, now)) {
        (Some((o, o_used, o_reset)), Some((n, n_used, n_reset))) => {
            let winner = if same_writer || n_reset > o_reset + SAME_WINDOW_SLACK {
                n
            } else if o_reset > n_reset + SAME_WINDOW_SLACK || n_used <= o_used {
                o
            } else {
                n
            };
            Some(winner.clone())
        }
        (Some((o, ..)), None) => Some(o.clone()),
        (None, Some((n, ..))) => Some(n.clone()),
        (None, None) => old.filter(|o| window_fields(o).is_some()).cloned(),
    }
}

fn window_keys(map: Option<&Map<String, Value>>) -> impl Iterator<Item = &String> {
    map.into_iter()
        .flatten()
        .filter(|(_, v)| window_fields(v).is_some())
        .map(|(k, _)| k)
}

fn number(value: f64) -> Value {
    if value.fract() == 0.0 && value.abs() < 9.0e15 {
        json!(value as i64)
    } else {
        json!(value)
    }
}

fn advance(before: Option<&Value>, after: Option<&Value>) -> f64 {
    let Some((after_used, after_reset)) = after.and_then(window_fields) else {
        return 0.0;
    };
    match before.and_then(window_fields) {
        Some((before_used, before_reset)) if after_reset <= before_reset + SAME_WINDOW_SLACK => {
            (after_used - before_used).max(0.0)
        }
        Some(_) => after_used,
        None => 0.0,
    }
}

fn dropped_before_reset(before: Option<&Value>, after: Option<&Value>, now: u64) -> Option<Value> {
    let (old, old_used, old_reset) = live(before, now)?;
    let (new_used, new_reset) = after.and_then(window_fields)?;
    (new_reset > old_reset + SAME_WINDOW_SLACK && new_used < old_used).then(|| {
        json!({
            "at": now,
            "from_used_percentage": old["used_percentage"],
            "from_resets_at": old["resets_at"],
        })
    })
}

fn days_left(resets_at: f64, now: u64) -> f64 {
    ((resets_at - now as f64) / DAY_S).ceil().clamp(1.0, 7.0)
}

fn day_start(before: Option<&Value>, seven: Option<&Value>, now: u64) -> Option<Value> {
    let (_, used, resets_at) = live(seven, now)?;
    let day = days_left(resets_at, now);
    let same_day = |start: &Value| {
        let from = start.get("resets_at")?.as_f64()?;
        let at = start.get("days_left")?.as_f64()?;
        let kept = start.get("used")?.as_f64()?;
        ((from - resets_at).abs() <= SAME_WINDOW_SLACK && at == day).then_some(kept)
    };
    let used = before.and_then(same_day).unwrap_or(used);
    Some(json!({"used": number(used), "resets_at": number(resets_at), "days_left": number(day)}))
}

fn merge_record(
    old: Option<&Map<String, Value>>,
    new_map: &Map<String, Value>,
    now: u64,
) -> Map<String, Value> {
    let keys: BTreeSet<String> = window_keys(old)
        .chain(window_keys(Some(new_map)))
        .cloned()
        .collect();
    let legacy_stamp = old
        .and_then(|m| m.get("data_changed_at").or_else(|| m.get("captured_at")))
        .and_then(Value::as_u64)
        .unwrap_or(now);
    let old_changed_at = old
        .and_then(|m| m.get("changed_at"))
        .and_then(Value::as_object);
    let old_written_by = old
        .and_then(|m| m.get(WRITTEN_BY))
        .and_then(Value::as_object);
    let old_session = old.and_then(|m| m.get("session_id"));
    let new_session = new_map.get("session_id");
    let mut out = new_map.clone();
    out.remove(ACCOUNTS);
    let mut changed_at = Map::new();
    let mut written_by = Map::new();
    let mut changed = old.is_none();
    for key in keys {
        let before = old.and_then(|m| m.get(&key));
        let before_writer = old_written_by
            .and_then(|m| m.get(&key))
            .or(old_session)
            .filter(|_| before.is_some());
        let same_writer = new_session.is_some() && before_writer == new_session;
        let merged = merge_window(before, new_map.get(&key), now, same_writer);
        let moved = merged.as_ref() != before;
        let took_new = new_map.get(&key).is_some() && merged.as_ref() == new_map.get(&key);
        let writer = if took_new { new_session } else { before_writer };
        if let Some(writer) = writer {
            written_by.insert(key.clone(), writer.clone());
        }
        changed |= moved;
        let stamp = if moved {
            now
        } else {
            old_changed_at
                .and_then(|m| m.get(&key))
                .and_then(Value::as_u64)
                .unwrap_or(legacy_stamp)
        };
        match merged {
            Some(v) => {
                out.insert(key.clone(), v);
                changed_at.insert(key, json!(stamp));
            }
            None => {
                out.remove(&key);
            }
        }
    }
    let five_before = old.and_then(|m| m.get(FIVE_HOUR));
    let seven_before = old.and_then(|m| m.get(SEVEN_DAY));
    let seven_moved = out.get(SEVEN_DAY) != seven_before;
    let flat = match old {
        Some(m) if !seven_moved => {
            m.get(FLAT_POINTS).and_then(Value::as_f64).unwrap_or(0.0)
                + advance(five_before, out.get(FIVE_HOUR))
        }
        _ => 0.0,
    };
    let dropped = match dropped_before_reset(seven_before, out.get(SEVEN_DAY), now) {
        Some(marker) => Some(marker),
        None if seven_moved => None,
        None => old.and_then(|m| m.get(DROPPED)).cloned(),
    };
    match day_start(old.and_then(|m| m.get(DAY_START)), out.get(SEVEN_DAY), now) {
        Some(start) => out.insert(DAY_START.into(), start),
        None => out.remove(DAY_START),
    };
    out.insert("changed_at".into(), Value::Object(changed_at));
    out.insert(WRITTEN_BY.into(), Value::Object(written_by));
    out.insert(FLAT_POINTS.into(), number(flat));
    match dropped {
        Some(marker) => out.insert(DROPPED.into(), marker),
        None => out.remove(DROPPED),
    };
    let stamp = if changed { now } else { legacy_stamp };
    out.insert("data_changed_at".into(), json!(stamp));
    out
}

fn has_live_window(record: &Value, now: u64) -> bool {
    record
        .as_object()
        .is_some_and(|m| m.values().any(|v| live(Some(v), now).is_some()))
}

fn matches_snapshot(record: Option<&Value>, snap: &Map<String, Value>, now: u64) -> bool {
    let near: Vec<bool> = window_keys(Some(snap))
        .filter_map(|key| {
            let (_, _, held) = live(record?.get(key), now)?;
            let (_, _, reset) = live(snap.get(key), now)?;
            Some((held - reset).abs() <= SAME_WINDOW_SLACK)
        })
        .collect();
    !near.is_empty() && near.iter().all(|&n| n)
}

fn owner_of_snapshot(
    accounts: &Map<String, Value>,
    claimed: &str,
    snap: &Map<String, Value>,
    now: u64,
) -> Option<String> {
    if matches_snapshot(accounts.get(claimed), snap, now) {
        return None;
    }
    let mut owners = accounts
        .iter()
        .filter(|(name, record)| {
            name.as_str() != claimed && matches_snapshot(Some(record), snap, now)
        })
        .map(|(name, _)| name.clone());
    let owner = owners.next()?;
    owners.next().is_none().then_some(owner)
}

pub fn merge(old: Option<&Value>, snap: &Value, now: u64) -> Value {
    let Some(new_map) = snap.as_object() else {
        return snap.clone();
    };
    let old_map = old.and_then(Value::as_object);
    let account = new_map.get("account").and_then(Value::as_str);
    let mut accounts = old_map
        .and_then(|m| m.get(ACCOUNTS))
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let owner = account.and_then(|key| owner_of_snapshot(&accounts, key, new_map, now));
    if let Some(owner) = &owner {
        let mut routed = new_map.clone();
        routed.insert("account".into(), json!(owner));
        let record = merge_record(accounts.get(owner).and_then(Value::as_object), &routed, now);
        accounts.insert(owner.clone(), Value::Object(record));
    }
    let previous = match account {
        Some(key) => accounts.get(key).and_then(Value::as_object),
        None => old_map.filter(|m| !m.contains_key("account")),
    };
    let record = match owner {
        Some(_) => previous.cloned(),
        None => Some(merge_record(previous, new_map, now)),
    };
    accounts.retain(|_, v| has_live_window(v, now));
    if let (Some(key), Some(record)) = (account, &record) {
        accounts.insert(key.to_owned(), Value::Object(record.clone()));
    }
    let mut out = record.unwrap_or_default();
    if !accounts.is_empty() {
        out.insert(ACCOUNTS.into(), Value::Object(accounts));
    }
    Value::Object(out)
}

pub fn config_path(config_dir: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    config_dir
        .filter(|dir| !dir.is_empty())
        .or_else(|| home.filter(|dir| !dir.is_empty()))
        .map(|dir| PathBuf::from(dir).join(".claude.json"))
}

pub fn account_key(
    config: Option<&Value>,
    org_env: Option<&str>,
    account_env: Option<&str>,
) -> Option<String> {
    let oauth = config.and_then(|c| c.get("oauthAccount"));
    let pick = |env: Option<&str>, field: &str| match env.filter(|v| !v.is_empty()) {
        Some(value) => Some(value.to_owned()),
        None => oauth?
            .get(field)?
            .as_str()
            .filter(|v| !v.is_empty())
            .map(str::to_owned),
    };
    let org = pick(org_env, "organizationUuid");
    let account = pick(account_env, "accountUuid");
    if org.is_none() && account.is_none() {
        return None;
    }
    Some(format!(
        "{}/{}",
        org.as_deref().unwrap_or("-"),
        account.as_deref().unwrap_or("-")
    ))
}

pub fn current_account() -> Option<String> {
    let config = config_path(
        std::env::var_os("CLAUDE_CONFIG_DIR"),
        std::env::var_os("HOME"),
    )
    .and_then(|path| std::fs::read_to_string(path).ok())
    .and_then(|text| serde_json::from_str::<Value>(&text).ok());
    let org = std::env::var("CLAUDE_CODE_ORGANIZATION_UUID").ok();
    let account = std::env::var("CLAUDE_CODE_ACCOUNT_UUID").ok();
    account_key(config.as_ref(), org.as_deref(), account.as_deref())
}

pub fn store(dir: &Path, snap: &Value, now: u64) -> Option<Value> {
    std::fs::create_dir_all(dir).ok()?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join(".rate-limits.lock"))
        .ok()?;
    lock.lock().ok()?;
    let target = dir.join("rate-limits.json");
    let old = std::fs::read_to_string(&target)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .filter(Value::is_object);
    let merged = merge(old.as_ref(), snap, now);
    let tmp = dir.join(format!(".rate-limits.{}.json", std::process::id()));
    let written =
        std::fs::write(&tmp, merged.to_string()).and_then(|()| std::fs::rename(&tmp, &target));
    if written.is_err() {
        let _ = std::fs::remove_file(&tmp);
        return None;
    }
    Some(merged)
}

pub fn ram_dir(var: Option<OsString>) -> PathBuf {
    var.filter(|value| !value.is_empty())
        .map_or_else(|| PathBuf::from("/tmp/ram"), PathBuf::from)
}

pub fn sync(payload: &Value) -> Option<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let mut snap = snapshot(payload, now)?;
    let account = current_account()?;
    snap.as_object_mut()?
        .insert("account".into(), json!(account));
    let ram = ram_dir(std::env::var_os("RAMDIR"));
    if !ram.is_dir() {
        return None;
    }
    store(&ram.join("claude"), &snap, now).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_removes_temp_file_when_rename_fails() {
        let dir =
            std::env::temp_dir().join(format!("nx-statusline-rename-fail-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("rate-limits.json")).unwrap();
        let now = 1_000_000;
        let snap = json!({"five_hour": {"used_percentage": 10, "resets_at": now + 3600}});

        assert!(store(&dir, &snap, now).is_none());
        assert!(store(&dir, &snap, now).is_none());

        let leftovers: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with(".rate-limits.") && name.ends_with(".json"))
            .collect();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            leftovers.is_empty(),
            "temp files left behind: {leftovers:?}"
        );
    }
}
