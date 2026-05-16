//! `Script` model and `ScriptRepo` — persistent scheduled jobs.

use crate::conn::Conn;
use crate::schema::scripts;
use crate::utils::{UID, gen_uid};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// A persistent scheduled job.
#[derive(Queryable, Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    /// unique id
    pub id: UID,
    /// owning object (None for global scripts)
    pub object_uid: Option<UID>,
    /// key registered with `Registry::register_script_handler`
    pub handler_key: String,
    /// re-fire interval in milliseconds (ignored if `repeat == 0`)
    pub interval_ms: i64,
    /// unix epoch milliseconds when the script should next execute
    pub next_run_at: i64,
    /// 1 = repeating, 0 = one-shot
    pub repeat: i32,
    /// per-script JSON-encoded persistent state
    pub state: String,
    /// 1 = enabled, 0 = disabled (won't be picked up)
    pub enabled: i32,
}

/// Insert payload for a new script.
#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = scripts)]
pub struct NewScript<'a> {
    /// pre-generated id
    pub id: UID,
    /// owning object uid
    pub object_uid: Option<UID>,
    /// handler key
    pub handler_key: &'a str,
    /// re-fire interval in ms
    pub interval_ms: i64,
    /// first-run timestamp (ms epoch)
    pub next_run_at: i64,
    /// 1=repeating, 0=one-shot
    pub repeat: i32,
    /// initial state JSON
    pub state: &'a str,
    /// enabled at creation (typically 1)
    pub enabled: i32,
}

/// CRUD for the `scripts` table.
pub struct ScriptRepo;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

impl ScriptRepo {
    /// Create a one-shot script that runs `delay_ms` from now.
    pub fn create_oneshot(
        &self,
        conn: &mut Conn,
        object_uid: Option<UID>,
        handler_key: &str,
        delay_ms: i64,
        initial_state: &serde_json::Value,
    ) -> QueryResult<Script> {
        self.create(conn, object_uid, handler_key, delay_ms, 0, 0, initial_state)
    }

    /// Create a repeating script that fires every `interval_ms`. First run is
    /// `interval_ms` from now (set `delay_ms != interval_ms` for offset).
    pub fn create_repeating(
        &self,
        conn: &mut Conn,
        object_uid: Option<UID>,
        handler_key: &str,
        interval_ms: i64,
        initial_state: &serde_json::Value,
    ) -> QueryResult<Script> {
        self.create(
            conn,
            object_uid,
            handler_key,
            interval_ms,
            interval_ms,
            1,
            initial_state,
        )
    }

    fn create(
        &self,
        conn: &mut Conn,
        object_uid: Option<UID>,
        handler_key: &str,
        delay_ms: i64,
        interval_ms: i64,
        repeat: i32,
        initial_state: &serde_json::Value,
    ) -> QueryResult<Script> {
        let id = gen_uid();
        let next_run_at = now_ms() + delay_ms;
        let state_str = serde_json::to_string(initial_state)
            .map_err(|e| diesel::result::Error::SerializationError(Box::new(e)))?;
        let row = NewScript {
            id,
            object_uid,
            handler_key,
            interval_ms,
            next_run_at,
            repeat,
            state: &state_str,
            enabled: 1,
        };
        diesel::insert_into(scripts::table)
            .values(&row)
            .execute(conn)?;
        self.get(conn, id)?.ok_or(diesel::result::Error::NotFound)
    }

    /// Fetch by id.
    pub fn get(&self, conn: &mut Conn, id: UID) -> QueryResult<Option<Script>> {
        scripts::table
            .filter(scripts::id.eq(id))
            .first::<Script>(conn)
            .optional()
    }

    /// Delete a script.
    pub fn delete(&self, conn: &mut Conn, id: UID) -> QueryResult<usize> {
        diesel::delete(scripts::table.filter(scripts::id.eq(id))).execute(conn)
    }

    /// Disable (without deleting). Use when a handler returns Err.
    pub fn disable(&self, conn: &mut Conn, id: UID) -> QueryResult<usize> {
        diesel::update(scripts::table.filter(scripts::id.eq(id)))
            .set(scripts::enabled.eq(0))
            .execute(conn)
    }

    /// All enabled scripts whose `next_run_at` <= `now_ms()`.
    /// Returned in ascending `next_run_at` order so older-overdue runs first.
    pub fn list_due(&self, conn: &mut Conn) -> QueryResult<Vec<Script>> {
        let now = now_ms();
        scripts::table
            .filter(scripts::enabled.eq(1))
            .filter(scripts::next_run_at.le(now))
            .order(scripts::next_run_at.asc())
            .load::<Script>(conn)
    }

    /// After a successful run, advance `next_run_at` and persist new `state`.
    /// For one-shot scripts (`repeat == 0`), this disables the script.
    pub fn record_run(
        &self,
        conn: &mut Conn,
        id: UID,
        new_state: &serde_json::Value,
    ) -> QueryResult<usize> {
        let script = self
            .get(conn, id)?
            .ok_or(diesel::result::Error::NotFound)?;
        let state_str = serde_json::to_string(new_state)
            .map_err(|e| diesel::result::Error::SerializationError(Box::new(e)))?;
        if script.repeat == 0 {
            return diesel::update(scripts::table.filter(scripts::id.eq(id)))
                .set((scripts::enabled.eq(0), scripts::state.eq(state_str)))
                .execute(conn);
        }
        let next_run_at = now_ms() + script.interval_ms;
        diesel::update(scripts::table.filter(scripts::id.eq(id)))
            .set((
                scripts::next_run_at.eq(next_run_at),
                scripts::state.eq(state_str),
            ))
            .execute(conn)
    }
}
