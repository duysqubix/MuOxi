#![cfg(feature = "db-sqlite")]

use db::structures::DatabaseHandlerExt;
use db::structures::account::{Account, AccountHandler};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

const SCHEMA: &str = include_str!("../../migrations/2026-05-07-000000_initial/up.sql");

fn fresh_conn() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:").expect("open in-memory sqlite");
    diesel::sql_query("PRAGMA foreign_keys = ON")
        .execute(&mut conn)
        .unwrap();
    for stmt in SCHEMA.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            diesel::sql_query(trimmed)
                .execute(&mut conn)
                .unwrap_or_else(|e| panic!("schema stmt failed: {} | stmt={}", e, trimmed));
        }
    }
    conn
}

#[test]
fn account_roundtrip() {
    let mut conn = fresh_conn();
    let h = AccountHandler;

    let a = Account {
        uid: 1,
        name: "alice".into(),
        password_hash: "argon2-blob".into(),
        email: "alice@example.com".into(),
        created_at: 1_700_000_000,
    };
    let inserted = h.insert(&mut conn, &a).expect("insert");
    assert_eq!(inserted.name, "alice");

    let fetched = h.get(&mut conn, 1).expect("get");
    assert_eq!(fetched.0.len(), 1);
    assert_eq!(fetched.0[0].password_hash, "argon2-blob");

    let removed = h.remove(&mut conn, 1).expect("remove");
    assert_eq!(removed, 1);

    let after = h.get(&mut conn, 1).expect("get after remove");
    assert!(after.0.is_empty());
}
