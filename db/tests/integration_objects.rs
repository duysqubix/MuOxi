//! Round-trip tests for the object/attribute/tag/character_account repos.
//! Runs in-memory SQLite under the default `db-sqlite` feature.

#![cfg(feature = "db-sqlite")]

use db::objects::{AttributeRepo, CharacterAccountRepo, ObjectRepo, TagRepo};
use db::structures::DatabaseHandlerExt;
use db::structures::account::{Account, AccountHandler};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

const SCHEMA_INITIAL: &str = include_str!("../../migrations/2026-05-07-000000_initial/up.sql");
const SCHEMA_OBJECTS: &str = include_str!("../../migrations/2026-05-07-000100_objects/up.sql");

fn strip_line_comments(sql: &str) -> String {
    sql.lines()
        .map(|line| match line.find("--") {
            Some(idx) => &line[..idx],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn apply_schema(conn: &mut SqliteConnection, sql: &str) {
    let stripped = strip_line_comments(sql);
    for stmt in stripped.split(';') {
        let trimmed = stmt.trim();
        if trimmed.is_empty() {
            continue;
        }
        diesel::sql_query(trimmed)
            .execute(conn)
            .unwrap_or_else(|e| panic!("schema stmt failed: {} | stmt={}", e, trimmed));
    }
}

fn fresh_conn() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:").expect("open memory sqlite");
    diesel::sql_query("PRAGMA foreign_keys = ON")
        .execute(&mut conn)
        .unwrap();
    apply_schema(&mut conn, SCHEMA_INITIAL);
    apply_schema(&mut conn, SCHEMA_OBJECTS);
    conn
}

fn seed_account(conn: &mut SqliteConnection, uid: i64, name: &str) {
    AccountHandler
        .insert(
            conn,
            &Account {
                uid,
                name: name.into(),
                password_hash: "x".into(),
                email: String::new(),
                created_at: 1,
            },
        )
        .expect("insert account");
}

#[test]
fn object_roundtrip_create_get_move_delete() {
    let mut conn = fresh_conn();
    let repo = ObjectRepo;

    let room = repo.create(&mut conn, "room", "Limbo", None).unwrap();
    assert_eq!(room.type_key, "room");
    assert_eq!(room.name, "Limbo");
    assert_eq!(room.location_uid, None);

    let item = repo
        .create(&mut conn, "item", "rock", Some(room.uid))
        .unwrap();
    assert_eq!(item.location_uid, Some(room.uid));

    let contents = repo.contents_of(&mut conn, room.uid).unwrap();
    assert_eq!(contents.len(), 1);
    assert_eq!(contents[0].uid, item.uid);

    repo.move_to(&mut conn, item.uid, None).unwrap();
    let after = repo.get(&mut conn, item.uid).unwrap().unwrap();
    assert_eq!(after.location_uid, None);

    let n = repo.delete(&mut conn, item.uid).unwrap();
    assert_eq!(n, 1);
    assert!(repo.get(&mut conn, item.uid).unwrap().is_none());
}

#[test]
fn attribute_roundtrip_set_get_all_delete() {
    let mut conn = fresh_conn();
    let obj_repo = ObjectRepo;
    let attr_repo = AttributeRepo;

    let mob = obj_repo.create(&mut conn, "mob", "goblin", None).unwrap();

    attr_repo
        .set(&mut conn, mob.uid, "hp", &serde_json::json!(20))
        .unwrap();
    attr_repo
        .set(
            &mut conn,
            mob.uid,
            "loot",
            &serde_json::json!(["coin", "knife"]),
        )
        .unwrap();

    let hp = attr_repo.get(&mut conn, mob.uid, "hp").unwrap().unwrap();
    assert_eq!(hp, serde_json::json!(20));

    let all = attr_repo.all(&mut conn, mob.uid).unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all["loot"], serde_json::json!(["coin", "knife"]));

    attr_repo
        .set(&mut conn, mob.uid, "hp", &serde_json::json!(15))
        .unwrap();
    let hp = attr_repo.get(&mut conn, mob.uid, "hp").unwrap().unwrap();
    assert_eq!(hp, serde_json::json!(15));

    let n = attr_repo.delete(&mut conn, mob.uid, "hp").unwrap();
    assert_eq!(n, 1);
    assert!(attr_repo.get(&mut conn, mob.uid, "hp").unwrap().is_none());
}

#[test]
fn tag_roundtrip_add_idempotent_lookup_remove() {
    let mut conn = fresh_conn();
    let obj_repo = ObjectRepo;
    let tag_repo = TagRepo;

    let r1 = obj_repo.create(&mut conn, "room", "Hall", None).unwrap();
    let r2 = obj_repo.create(&mut conn, "room", "Cellar", None).unwrap();

    tag_repo.add(&mut conn, r1.uid, "safe-zone", "perm").unwrap();
    tag_repo.add(&mut conn, r2.uid, "safe-zone", "perm").unwrap();
    tag_repo.add(&mut conn, r1.uid, "safe-zone", "perm").unwrap();

    assert!(
        tag_repo
            .has(&mut conn, r1.uid, "safe-zone", "perm")
            .unwrap()
    );
    let mut hits = tag_repo
        .objects_with(&mut conn, "safe-zone", "perm")
        .unwrap();
    hits.sort();
    assert_eq!(hits.len(), 2);

    tag_repo
        .remove(&mut conn, r1.uid, "safe-zone", "perm")
        .unwrap();
    assert!(
        !tag_repo
            .has(&mut conn, r1.uid, "safe-zone", "perm")
            .unwrap()
    );
}

#[test]
fn character_account_link_unlink_list() {
    let mut conn = fresh_conn();
    let obj_repo = ObjectRepo;
    let link_repo = CharacterAccountRepo;

    seed_account(&mut conn, 1, "alice");

    let c1 = obj_repo
        .create(&mut conn, "character", "Alice", None)
        .unwrap();
    let c2 = obj_repo
        .create(&mut conn, "character", "Alex", None)
        .unwrap();

    let l1 = link_repo.link(&mut conn, c1.uid, 1).unwrap();
    assert_eq!(l1.ordinal, 0);
    let l2 = link_repo.link(&mut conn, c2.uid, 1).unwrap();
    assert_eq!(l2.ordinal, 1);

    let listed = link_repo.list_for_account(&mut conn, 1).unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].object_uid, c1.uid);

    link_repo.unlink(&mut conn, c1.uid).unwrap();
    let listed = link_repo.list_for_account(&mut conn, 1).unwrap();
    assert_eq!(listed.len(), 1);
}
