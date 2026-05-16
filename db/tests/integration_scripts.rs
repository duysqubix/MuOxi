//! Round-trip tests for ScriptRepo on in-memory SQLite.

#![cfg(feature = "db-sqlite")]

use db::objects::{ObjectRepo, ScriptRepo};
use diesel::prelude::*;
use diesel::connection::SimpleConnection;
use diesel::sqlite::SqliteConnection;
use std::time::{SystemTime, UNIX_EPOCH};

const SCHEMA_INITIAL: &str = include_str!("../../migrations/2026-05-07-000000_initial/up.sql");
const SCHEMA_OBJECTS: &str = include_str!("../../migrations/2026-05-07-000100_objects/up.sql");
const SCHEMA_SCRIPTS: &str = include_str!("../../migrations/2026-05-07-000200_scripts/up.sql");

fn fresh_conn() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:").expect("memory sqlite");
    conn.batch_execute("PRAGMA foreign_keys = ON").unwrap();
    for src in [SCHEMA_INITIAL, SCHEMA_OBJECTS, SCHEMA_SCRIPTS] {
        conn.batch_execute(src)
            .unwrap_or_else(|e| panic!("schema stmt failed: {}", e));
    }
    conn
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[test]
fn oneshot_roundtrip_and_disable_after_run() {
    let mut conn = fresh_conn();
    let repo = ScriptRepo;
    let s = repo
        .create_oneshot(
            &mut conn,
            None,
            "demo",
            0,
            &serde_json::json!({"k": 1}),
        )
        .unwrap();
    assert_eq!(s.repeat, 0);
    assert_eq!(s.enabled, 1);

    // due immediately (delay 0)
    let due = repo.list_due(&mut conn).unwrap();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].id, s.id);

    // simulate a successful run
    repo.record_run(&mut conn, s.id, &serde_json::json!({"k": 2}))
        .unwrap();
    let after = repo.get(&mut conn, s.id).unwrap().unwrap();
    assert_eq!(after.enabled, 0); // one-shot disables itself
    assert_eq!(after.state, "{\"k\":2}");

    let due_after = repo.list_due(&mut conn).unwrap();
    assert!(due_after.is_empty());
}

#[test]
fn repeating_advances_next_run_at() {
    let mut conn = fresh_conn();
    let repo = ScriptRepo;
    let s = repo
        .create_repeating(
            &mut conn,
            None,
            "tick",
            1000,
            &serde_json::json!({}),
        )
        .unwrap();
    let started_at = now_ms();
    assert!(s.next_run_at >= started_at + 999);

    // record a run -> next_run_at advances by interval_ms
    let pre_run_at_max = now_ms() + 1100;
    repo.record_run(&mut conn, s.id, &serde_json::json!({"ran": 1}))
        .unwrap();
    let after = repo.get(&mut conn, s.id).unwrap().unwrap();
    assert_eq!(after.enabled, 1);
    assert!(after.next_run_at >= pre_run_at_max - 200);
    assert!(after.next_run_at <= pre_run_at_max + 200);
}

#[test]
fn disable_removes_from_due_list() {
    let mut conn = fresh_conn();
    let repo = ScriptRepo;
    let s = repo
        .create_oneshot(&mut conn, None, "demo", 0, &serde_json::json!({}))
        .unwrap();
    assert_eq!(repo.list_due(&mut conn).unwrap().len(), 1);
    repo.disable(&mut conn, s.id).unwrap();
    assert!(repo.list_due(&mut conn).unwrap().is_empty());
}

#[test]
fn fk_cascade_deletes_object_scripts() {
    let mut conn = fresh_conn();
    let obj_repo = ObjectRepo;
    let script_repo = ScriptRepo;

    let mob = obj_repo.create(&mut conn, "mob", "goblin", None).unwrap();
    let s = script_repo
        .create_repeating(&mut conn, Some(mob.uid), "ai", 5000, &serde_json::json!({}))
        .unwrap();

    obj_repo.delete(&mut conn, mob.uid).unwrap();
    let after = script_repo.get(&mut conn, s.id).unwrap();
    assert!(after.is_none(), "ON DELETE CASCADE should drop the script");
}
