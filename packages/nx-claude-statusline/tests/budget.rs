use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use nx_claude_statusline::budget::{account_key, config_path, merge, ram_dir, snapshot, store};
use serde_json::{Value, json};

const NOW: u64 = 1_000_000;
const T: u64 = NOW + 3600;
const T7: u64 = NOW + 500_000;

fn w(used: u64, resets: u64) -> Value {
    json!({"used_percentage": used, "resets_at": resets})
}

fn snap(fields: Value) -> Value {
    let mut map = fields.as_object().cloned().unwrap_or_default();
    map.insert("captured_at".into(), json!(NOW));
    map.insert("session_id".into(), json!("new"));
    Value::Object(map)
}

fn snap_from(fields: Value, session: &str) -> Value {
    let mut out = snap(fields);
    out["session_id"] = json!(session);
    out
}

fn stored(fields: Value, data_changed_at: u64) -> Value {
    let mut map = fields.as_object().cloned().unwrap_or_default();
    map.insert("captured_at".into(), json!(500));
    map.insert("data_changed_at".into(), json!(data_changed_at));
    Value::Object(map)
}

fn fresh_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nx-statusline-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn read_file(dir: &std::path::Path) -> Value {
    let text = std::fs::read_to_string(dir.join("rate-limits.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

#[test]
fn snapshot_copies_rate_limits_verbatim_and_stamps_time_and_session() {
    let payload = json!({
        "session_id": "abc",
        "rate_limits": {
            "five_hour": {"used_percentage": 42.5, "resets_at": 1793000000},
            "seven_day": {"used_percentage": 12.0, "resets_at": 1793500000}
        }
    });
    assert_eq!(
        snapshot(&payload, 100),
        Some(json!({
            "five_hour": {"used_percentage": 42.5, "resets_at": 1793000000},
            "seven_day": {"used_percentage": 12.0, "resets_at": 1793500000},
            "captured_at": 100,
            "session_id": "abc"
        }))
    );
}

#[test]
fn snapshot_skips_payloads_without_rate_limits() {
    assert_eq!(snapshot(&json!({"session_id": "abc"}), 100), None);
}

#[test]
fn same_window_keeps_the_higher_percentage() {
    let old = stored(json!({"five_hour": w(60, T)}), 900);
    let out = merge(Some(&old), &snap(json!({"five_hour": w(40, T)})), NOW);
    assert_eq!(out["five_hour"], w(60, T));
    assert_eq!(out["data_changed_at"], json!(900));
    assert_eq!(out["captured_at"], json!(NOW));
    assert_eq!(out["session_id"], json!("new"));
}

#[test]
fn higher_percentage_wins_and_stamps_change() {
    let old = stored(json!({"five_hour": w(40, T)}), 900);
    let out = merge(Some(&old), &snap(json!({"five_hour": w(41, T)})), NOW);
    assert_eq!(out["five_hour"], w(41, T));
    assert_eq!(out["data_changed_at"], json!(NOW));
}

#[test]
fn later_window_replaces_higher_old() {
    let old = stored(json!({"five_hour": w(89, T)}), 900);
    let out = merge(
        Some(&old),
        &snap(json!({"five_hour": w(3, T + 18000)})),
        NOW,
    );
    assert_eq!(out["five_hour"], w(3, T + 18000));
    assert_eq!(out["data_changed_at"], json!(NOW));
}

#[test]
fn older_live_window_never_replaces_later() {
    let old = stored(json!({"five_hour": w(3, T + 18000)}), 900);
    let out = merge(Some(&old), &snap(json!({"five_hour": w(89, T)})), NOW);
    assert_eq!(out["five_hour"], w(3, T + 18000));
    assert_eq!(out["data_changed_at"], json!(900));
}

#[test]
fn expired_incoming_is_dropped() {
    let out = merge(None, &snap(json!({"five_hour": w(95, NOW - 1)})), NOW);
    assert!(out.get("five_hour").is_none());
    let old = stored(json!({"five_hour": w(10, T)}), 900);
    let out = merge(Some(&old), &snap(json!({"five_hour": w(95, NOW - 1)})), NOW);
    assert_eq!(out["five_hour"], w(10, T));
    assert_eq!(out["data_changed_at"], json!(900));
}

#[test]
fn expired_stored_stays_until_live_arrives() {
    let old = stored(json!({"five_hour": w(89, NOW - 1)}), 900);
    let out = merge(Some(&old), &snap(json!({})), NOW);
    assert_eq!(out["five_hour"], w(89, NOW - 1));
    assert_eq!(out["data_changed_at"], json!(900));
    let out = merge(
        Some(&out),
        &snap(json!({"five_hour": w(2, NOW + 18000)})),
        NOW,
    );
    assert_eq!(out["five_hour"], w(2, NOW + 18000));
    assert_eq!(out["data_changed_at"], json!(NOW));
}

#[test]
fn windows_merge_independently() {
    let old = stored(json!({"five_hour": w(50, T), "seven_day": w(10, T7)}), 900);
    let out = merge(
        Some(&old),
        &snap(json!({"five_hour": w(60, T), "seven_day": w(8, T7)})),
        NOW,
    );
    assert_eq!(out["five_hour"], w(60, T));
    assert_eq!(out["seven_day"], w(10, T7));
    assert_eq!(out["data_changed_at"], json!(NOW));
}

#[test]
fn jitter_within_slack_is_same_window() {
    let old = stored(json!({"five_hour": w(60, T)}), 900);
    let out = merge(Some(&old), &snap(json!({"five_hour": w(40, T + 5)})), NOW);
    assert_eq!(out["five_hour"], w(60, T));
    let out = merge(Some(&old), &snap(json!({"five_hour": w(70, T - 5)})), NOW);
    assert_eq!(out["five_hour"], w(70, T - 5));
}

#[test]
fn first_write_and_legacy_file_stamps() {
    let out = merge(None, &snap(json!({"five_hour": w(40, T)})), NOW);
    assert_eq!(out["data_changed_at"], json!(NOW));
    let legacy = json!({"five_hour": w(40, T), "captured_at": 700});
    let out = merge(Some(&legacy), &snap(json!({"five_hour": w(40, T)})), NOW);
    assert_eq!(out["data_changed_at"], json!(700));
}

#[test]
fn integer_percentages_stay_integers() {
    let out = merge(None, &snap(json!({"five_hour": w(40, T)})), NOW);
    assert!(!out.to_string().contains("40.0"));
    assert!(out["five_hour"]["used_percentage"].is_u64());
}

#[test]
fn store_round_trips_and_leaves_no_temp_files() {
    let dir = fresh_dir("store");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("rate-limits.json"), "{").unwrap();
    store(&dir, &snap(json!({"five_hour": w(50, T)})), NOW).unwrap();
    let out = store(
        &dir,
        &snap_from(json!({"five_hour": w(30, T)}), "other"),
        NOW,
    )
    .unwrap();
    assert_eq!(out["five_hour"], w(50, T));
    assert_eq!(read_file(&dir)["five_hour"], w(50, T));
    let names: BTreeSet<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        names,
        BTreeSet::from(["rate-limits.json".into(), ".rate-limits.lock".into()])
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn concurrent_writers_end_at_the_max() {
    let dir = Arc::new(fresh_dir("concurrent"));
    let handles: Vec<_> = (1..=16)
        .map(|used| {
            let dir = Arc::clone(&dir);
            std::thread::spawn(move || {
                let session = format!("s{used}");
                store(
                    &dir,
                    &snap_from(json!({"five_hour": w(used, T)}), &session),
                    NOW,
                )
                .unwrap();
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }
    assert_eq!(read_file(&dir)["five_hour"], w(16, T));
    let _ = std::fs::remove_dir_all(&*dir);
}

#[test]
fn empty_ramdir_falls_back_like_the_python_reader() {
    assert_eq!(ram_dir(None), PathBuf::from("/tmp/ram"));
    assert_eq!(ram_dir(Some("".into())), PathBuf::from("/tmp/ram"));
    assert_eq!(ram_dir(Some("/x".into())), PathBuf::from("/x"));
}

const ORG_A: &str = "org-a";
const ORG_B: &str = "org-b";

fn keyed(fields: Value, account: &str) -> Value {
    let mut out = snap(fields);
    out["account"] = json!(account);
    out
}

fn config(org: &str, account: &str) -> Value {
    json!({"oauthAccount": {"organizationUuid": org, "accountUuid": account, "emailAddress": "x"}})
}

#[test]
fn account_key_prefers_the_env_then_the_oauth_account() {
    let cfg = config("o1", "a1");
    assert_eq!(
        account_key(Some(&cfg), None, None).as_deref(),
        Some("o1/a1")
    );
    assert_eq!(
        account_key(Some(&cfg), Some("o2"), Some("")).as_deref(),
        Some("o2/a1")
    );
    assert_eq!(account_key(None, None, Some("a3")).as_deref(), Some("-/a3"));
    assert_eq!(account_key(Some(&json!({})), None, None), None);
    assert_eq!(account_key(None, Some(""), None), None);
}

#[test]
fn config_path_follows_claude_config_dir_then_home() {
    assert_eq!(
        config_path(Some("/c".into()), Some("/h".into())),
        Some(PathBuf::from("/c/.claude.json"))
    );
    assert_eq!(
        config_path(Some("".into()), Some("/h".into())),
        Some(PathBuf::from("/h/.claude.json"))
    );
    assert_eq!(config_path(None, None), None);
}

#[test]
fn a_login_switch_never_carries_one_accounts_reading_into_another() {
    let a = merge(
        None,
        &keyed(
            json!({"five_hour": w(30, T), "seven_day": w(70, T7)}),
            ORG_A,
        ),
        NOW,
    );
    let b = merge(
        Some(&a),
        &keyed(
            json!({"five_hour": w(10, T), "seven_day": w(24, T7 - 86_400)}),
            ORG_B,
        ),
        NOW,
    );
    assert_eq!(b["account"], json!(ORG_B));
    assert_eq!(b["five_hour"], w(10, T));
    assert_eq!(b["accounts"][ORG_A]["five_hour"], w(30, T));
    assert_eq!(b["seven_day"], w(24, T7 - 86_400));
    assert_eq!(b["accounts"][ORG_A]["seven_day"], w(70, T7));
    assert_eq!(b["accounts"][ORG_B]["seven_day"], w(24, T7 - 86_400));
    let mut from_another_session = keyed(
        json!({"five_hour": w(31, T), "seven_day": w(65, T7)}),
        ORG_A,
    );
    from_another_session["session_id"] = json!("other");
    let back = merge(Some(&b), &from_another_session, NOW);
    assert_eq!(back["seven_day"], w(70, T7));
    assert_eq!(back["five_hour"], w(31, T));
    assert_eq!(back["accounts"][ORG_B]["seven_day"], w(24, T7 - 86_400));
}

#[test]
fn an_unkeyed_legacy_file_is_not_merged_into_a_keyed_account() {
    let legacy = stored(json!({"five_hour": w(40, T), "seven_day": w(70, T7)}), 900);
    let out = merge(
        Some(&legacy),
        &keyed(
            json!({"five_hour": w(12, T), "seven_day": w(30, T7)}),
            ORG_A,
        ),
        NOW,
    );
    assert_eq!(out["seven_day"], w(30, T7));
    assert_eq!(out["five_hour"], w(12, T));
    assert_eq!(out["data_changed_at"], json!(NOW));
}

#[test]
fn accounts_without_a_live_window_are_pruned() {
    let old =
        json!({"accounts": {ORG_B: {"five_hour": w(50, NOW - 10), "seven_day": w(9, NOW - 1)}}});
    let out = merge(
        Some(&old),
        &keyed(json!({"five_hour": w(1, T)}), ORG_A),
        NOW,
    );
    assert!(out["accounts"].get(ORG_B).is_none());
    assert!(out["accounts"].get(ORG_A).is_some());
}

#[test]
fn each_window_keeps_its_own_changed_time() {
    let mut old = stored(json!({"five_hour": w(40, T), "seven_day": w(70, T7)}), 900);
    old["changed_at"] = json!({"five_hour": 900, "seven_day": 700});
    let out = merge(
        Some(&old),
        &snap(json!({"five_hour": w(45, T), "seven_day": w(70, T7)})),
        NOW,
    );
    assert_eq!(
        out["changed_at"],
        json!({"five_hour": NOW, "seven_day": 700})
    );
    assert_eq!(out["data_changed_at"], json!(NOW));
}

#[test]
fn a_record_without_window_times_seeds_them_from_its_data_stamp() {
    let old = stored(json!({"five_hour": w(40, T), "seven_day": w(70, T7)}), 900);
    let out = merge(
        Some(&old),
        &snap(json!({"five_hour": w(41, T), "seven_day": w(70, T7)})),
        NOW,
    );
    assert_eq!(
        out["changed_at"],
        json!({"five_hour": NOW, "seven_day": 900})
    );
}

#[test]
fn flat_points_add_up_five_hour_use_until_seven_day_moves() {
    let first = merge(
        None,
        &keyed(
            json!({"five_hour": w(10, T), "seven_day": w(70, T7)}),
            ORG_A,
        ),
        NOW,
    );
    assert_eq!(first["seven_day_flat_five_hour_points"], json!(0));
    let second = merge(
        Some(&first),
        &keyed(
            json!({"five_hour": w(25, T), "seven_day": w(70, T7)}),
            ORG_A,
        ),
        NOW,
    );
    assert_eq!(second["seven_day_flat_five_hour_points"], json!(15));
    let lagging = merge(
        Some(&second),
        &keyed(
            json!({"five_hour": w(20, T), "seven_day": w(70, T7)}),
            ORG_A,
        ),
        NOW,
    );
    assert_eq!(lagging["seven_day_flat_five_hour_points"], json!(15));
    let rolled = merge(
        Some(&lagging),
        &keyed(
            json!({"five_hour": w(6, T + 18_000), "seven_day": w(70, T7)}),
            ORG_A,
        ),
        NOW,
    );
    assert_eq!(rolled["seven_day_flat_five_hour_points"], json!(21));
    let moved = merge(
        Some(&rolled),
        &keyed(
            json!({"five_hour": w(9, T + 18_000), "seven_day": w(71, T7)}),
            ORG_A,
        ),
        NOW,
    );
    assert_eq!(moved["seven_day_flat_five_hour_points"], json!(0));
    assert_eq!(moved["changed_at"]["seven_day"], json!(NOW));
}

#[test]
fn flat_points_keep_fractions_and_integers_apart() {
    let old = stored(json!({"five_hour": w(10, T), "seven_day": w(70, T7)}), 900);
    let out = merge(
        Some(&old),
        &snap(
            json!({"five_hour": {"used_percentage": 12.5, "resets_at": T}, "seven_day": w(70, T7)}),
        ),
        NOW,
    );
    assert_eq!(out["seven_day_flat_five_hour_points"], json!(2.5));
    assert!(out["seven_day_flat_five_hour_points"].is_f64());
}

#[test]
fn a_seven_day_drop_before_its_reset_is_marked_until_it_moves() {
    let old = stored(json!({"five_hour": w(40, T), "seven_day": w(54, T7)}), 900);
    let dropped = merge(
        Some(&old),
        &snap(json!({"five_hour": w(40, T), "seven_day": w(24, T7 + 86_400)})),
        NOW,
    );
    assert_eq!(dropped["seven_day"], w(24, T7 + 86_400));
    assert_eq!(
        dropped["seven_day_dropped"],
        json!({"at": NOW, "from_used_percentage": 54, "from_resets_at": T7})
    );
    let still = merge(
        Some(&dropped),
        &snap(json!({"five_hour": w(41, T), "seven_day": w(24, T7 + 86_400)})),
        NOW + 60,
    );
    assert_eq!(still["seven_day_dropped"], dropped["seven_day_dropped"]);
    let moved = merge(
        Some(&still),
        &snap(json!({"five_hour": w(42, T), "seven_day": w(25, T7 + 86_400)})),
        NOW + 120,
    );
    assert!(moved.get("seven_day_dropped").is_none());
}

#[test]
fn a_lower_reading_inside_the_same_window_or_after_reset_is_not_a_drop() {
    let old = stored(json!({"seven_day": w(54, T7)}), 900);
    let lagging = merge(Some(&old), &snap(json!({"seven_day": w(40, T7)})), NOW);
    assert_eq!(lagging["seven_day"], w(54, T7));
    assert!(lagging.get("seven_day_dropped").is_none());
    let expired = stored(json!({"seven_day": w(54, NOW - 1)}), 900);
    let reset = merge(Some(&expired), &snap(json!({"seven_day": w(3, T7)})), NOW);
    assert_eq!(reset["seven_day"], w(3, T7));
    assert!(reset.get("seven_day_dropped").is_none());
}

#[test]
fn store_keeps_accounts_apart_on_disk() {
    let dir = fresh_dir("accounts");
    store(&dir, &keyed(json!({"seven_day": w(70, T7)}), ORG_A), NOW).unwrap();
    store(
        &dir,
        &keyed(json!({"seven_day": w(20, T7 - 3_600)}), ORG_B),
        NOW,
    )
    .unwrap();
    let file = read_file(&dir);
    assert_eq!(file["account"], json!(ORG_B));
    assert_eq!(file["seven_day"], w(20, T7 - 3_600));
    assert_eq!(file["accounts"][ORG_A]["seven_day"], w(70, T7));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_session_replaces_its_own_reading_even_when_lower() {
    let stale = merge(None, &snap(json!({"seven_day": w(87, T7)})), NOW);
    let fresh = merge(
        Some(&stale),
        &snap(json!({"seven_day": w(12, T7 - 86_400)})),
        NOW,
    );
    assert_eq!(fresh["seven_day"], w(12, T7 - 86_400));
    assert_eq!(fresh["written_by"]["seven_day"], json!("new"));
    let other = merge(
        Some(&fresh),
        &snap_from(json!({"seven_day": w(5, T7 - 86_400)}), "other"),
        NOW,
    );
    assert_eq!(other["seven_day"], w(12, T7 - 86_400));
    assert_eq!(other["written_by"]["seven_day"], json!("new"));
}

#[test]
fn a_legacy_record_without_writers_trusts_its_last_session() {
    let mut legacy = stored(json!({"seven_day": w(87, T7)}), 900);
    legacy["session_id"] = json!("new");
    let out = merge(Some(&legacy), &snap(json!({"seven_day": w(20, T7)})), NOW);
    assert_eq!(out["seven_day"], w(20, T7));
}

#[test]
fn a_window_matching_another_accounts_reset_is_routed_there() {
    let a = merge(
        None,
        &keyed(
            json!({"five_hour": w(38, T), "seven_day": w(87, T7)}),
            ORG_A,
        ),
        NOW,
    );
    let b = merge(
        Some(&a),
        &keyed(
            json!({"five_hour": w(5, T - 1800), "seven_day": w(5, T7 - 7200)}),
            ORG_B,
        ),
        NOW,
    );
    let mut stale = keyed(
        json!({"five_hour": w(39, T), "seven_day": w(86, T7)}),
        ORG_B,
    );
    stale["session_id"] = json!("stale");
    let routed = merge(Some(&b), &stale, NOW);
    assert_eq!(routed["account"], json!(ORG_B));
    assert_eq!(routed["seven_day"], w(5, T7 - 7200));
    assert_eq!(routed["five_hour"], w(5, T - 1800));
    assert_eq!(routed["accounts"][ORG_A]["seven_day"], w(87, T7));
    assert_eq!(routed["accounts"][ORG_A]["five_hour"], w(39, T));
    let renewed = merge(
        Some(&routed),
        &keyed(json!({"seven_day": w(1, T7 + 600_000)}), ORG_B),
        NOW,
    );
    assert_eq!(renewed["seven_day"], w(1, T7 + 600_000));
}

#[test]
fn a_stale_first_write_for_a_new_account_goes_to_the_account_it_matches() {
    let a = merge(None, &keyed(json!({"seven_day": w(87, T7)}), ORG_A), NOW);
    let mut stale = keyed(json!({"seven_day": w(86, T7)}), ORG_B);
    stale["session_id"] = json!("stale");
    let routed = merge(Some(&a), &stale, NOW);
    assert_eq!(routed["accounts"][ORG_A]["seven_day"], w(87, T7));
    assert!(routed["accounts"][ORG_B].get("seven_day").is_none());
    let real = merge(
        Some(&routed),
        &keyed(json!({"seven_day": w(5, T7 - 7200)}), ORG_B),
        NOW,
    );
    assert_eq!(real["seven_day"], w(5, T7 - 7200));
}

#[test]
fn a_new_five_hour_window_after_its_own_expired_one_stays_with_its_account() {
    let old = json!({"accounts": {
        ORG_A: {"account": ORG_A, "five_hour": w(60, NOW - 10), "seven_day": w(40, T7)},
        ORG_B: {"account": ORG_B, "five_hour": w(20, T), "seven_day": w(10, T7 - 86_400)},
    }});
    let out = merge(
        Some(&old),
        &keyed(
            json!({"five_hour": w(93, T), "seven_day": w(41, T7)}),
            ORG_A,
        ),
        NOW,
    );
    assert_eq!(out["accounts"][ORG_A]["five_hour"], w(93, T));
    assert_eq!(out["accounts"][ORG_B]["five_hour"], w(20, T));
}

#[test]
fn a_stale_sessions_new_five_hour_window_follows_its_seven_day_to_its_account() {
    let old = json!({"accounts": {
        ORG_A: {"account": ORG_A, "five_hour": w(60, NOW - 10), "seven_day": w(40, T7)},
        ORG_B: {"account": ORG_B, "five_hour": w(20, T - 1800), "seven_day": w(10, T7 - 86_400)},
    }});
    let mut stale = keyed(json!({"five_hour": w(5, T), "seven_day": w(41, T7)}), ORG_B);
    stale["session_id"] = json!("stale");
    let out = merge(Some(&old), &stale, NOW);
    assert_eq!(out["accounts"][ORG_A]["five_hour"], w(5, T));
    assert_eq!(out["accounts"][ORG_B]["five_hour"], w(20, T - 1800));
}

#[test]
fn a_snapshot_routed_to_another_account_leaves_the_claimed_record_untouched() {
    let mut claimed = stored(
        json!({"five_hour": w(5, T - 1800), "seven_day": w(5, T7 - 7200)}),
        NOW - 3600,
    );
    claimed["account"] = json!(ORG_B);
    claimed["session_id"] = json!("b");
    let old = json!({"accounts": {
        ORG_A: {"account": ORG_A, "five_hour": w(38, T), "seven_day": w(87, T7)},
        ORG_B: claimed.clone(),
    }});
    let mut stale = keyed(
        json!({"five_hour": w(39, T), "seven_day": w(86, T7)}),
        ORG_B,
    );
    stale["session_id"] = json!("stale");
    let out = merge(Some(&old), &stale, NOW);
    assert_eq!(out["accounts"][ORG_B], claimed);
    assert_eq!(out["accounts"][ORG_A]["five_hour"], w(39, T));
}

#[test]
fn the_first_capture_of_a_window_day_pins_that_days_seven_day_start() {
    let start =
        |used: u64, days_left: u64| json!({"used": used, "resets_at": T7, "days_left": days_left});
    let first = merge(None, &keyed(json!({"seven_day": w(14, T7)}), ORG_A), NOW);
    assert_eq!(first["seven_day_day_start"], start(14, 6));
    let later = merge(
        Some(&first),
        &keyed(json!({"seven_day": w(30, T7)}), ORG_A),
        NOW + 3600,
    );
    assert_eq!(later["seven_day_day_start"], start(14, 6));
    let next_day = merge(
        Some(&later),
        &keyed(json!({"seven_day": w(35, T7)}), ORG_A),
        NOW + 86_400,
    );
    assert_eq!(next_day["seven_day_day_start"], start(35, 5));
    let other = merge(
        Some(&next_day),
        &keyed(json!({"seven_day": w(5, T7 - 7200)}), ORG_B),
        NOW + 86_400,
    );
    assert_eq!(
        other["accounts"][ORG_A]["seven_day_day_start"],
        start(35, 5)
    );
    let expired = merge(Some(&next_day), &keyed(json!({}), ORG_A), T7);
    assert!(expired.get("seven_day_day_start").is_none());
}

#[test]
fn an_expired_window_in_a_stale_payload_never_keeps_it_out_of_its_account() {
    let old = json!({"accounts": {
        ORG_A: {"account": ORG_A, "five_hour": w(10, T), "seven_day": w(40, T7)},
        ORG_B: {"account": ORG_B, "five_hour": w(20, T - 1800), "seven_day": w(10, T7 - 86_400)},
    }});
    let mut stale = keyed(
        json!({"five_hour": w(95, NOW - 600), "seven_day": w(41, T7)}),
        ORG_B,
    );
    stale["session_id"] = json!("stale");
    let out = merge(Some(&old), &stale, NOW);
    assert_eq!(out["accounts"][ORG_B]["seven_day"], w(10, T7 - 86_400));
    assert_eq!(out["accounts"][ORG_A]["seven_day"], w(41, T7));
}

#[test]
fn a_stale_first_write_with_a_new_five_hour_window_goes_whole_to_the_account_it_matches() {
    let old = json!({"accounts": {
        ORG_A: {"account": ORG_A, "five_hour": w(60, NOW - 10), "seven_day": w(40, T7)},
    }});
    let mut stale = keyed(json!({"five_hour": w(7, T), "seven_day": w(41, T7)}), ORG_B);
    stale["session_id"] = json!("stale");
    let out = merge(Some(&old), &stale, NOW);
    assert_eq!(out["accounts"][ORG_A]["five_hour"], w(7, T));
    assert_eq!(out["accounts"][ORG_A]["seven_day"], w(41, T7));
    assert!(out["accounts"].get(ORG_B).is_none());
}
