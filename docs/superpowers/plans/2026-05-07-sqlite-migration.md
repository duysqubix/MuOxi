# SQLite Migration + JSON/Watchdog Removal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace Postgres with SQLite as the default backend, delete the JSON-canonical / watchdog mirror model, keep Postgres available behind a Cargo feature for production deployments.

**Architecture:** Diesel 2.x with `sqlite` feature (bundled `libsqlite3-sys`) is the default. A `db-postgres` Cargo feature opts into the original Postgres backend. Connection type is selected at compile time (`SqliteConnection` or `PgConnection`) via the `db::Conn` type alias. The `account_characters` join table replaces the Postgres-only `BIGINT[]` column. The `muoxi_watchdog` binary and `json/*.json` runtime usage are deleted.

**Tech Stack:** Diesel 2.2, `libsqlite3-sys = { version = "0.30", features = ["bundled"] }` (transitively via `diesel/sqlite-bundled` is also acceptable but explicit gives clearer build errors), Cargo features for backend selection.

---

## File Structure

**Create:**
- `migrations/2026-05-07-000000_initial/up.sql` — portable schema (replaces existing migrations)
- `migrations/2026-05-07-000000_initial/down.sql`
- `db/src/conn.rs` — `Conn` type alias + connection establishment helpers (one place, feature-gated)
- `data/.gitkeep` — directory for `world.db` SQLite file (gitignored)

**Modify:**
- `Cargo.toml` — workspace deps: drop `postgres` from diesel default, add backend features
- `db/Cargo.toml` — feature flags `db-sqlite` (default) and `db-postgres`
- `db/src/lib.rs` — use `Conn` from new `conn` module; remove `extern crate diesel` was already done
- `db/src/structures.rs` — `&mut PgConnection` → `&mut Conn`; remove `characters: Option<Vec<i64>>` field
- `db/src/schema.rs` — regenerated for SQLite types (`Integer` instead of `Int8` where applicable; arrays gone)
- `muoxi/Cargo.toml` — remove `[[bin]] muoxi_watchdog` entry; remove `hotwatch` dependency
- `tester/Cargo.toml` — remove postgres-only dependencies if any
- `diesel.toml` — `[print_schema]` adds `with_docs = true` (style nit)
- `Cargo.lock` — regenerated after dep changes
- `Dockerfile` — modern rust image, no `libpq-dev`, no `diesel_cli postgres` install (use sqlite features)
- `Dockerfile.postgres` — DELETED
- `docker-compose.yml` — remove `postgres` and `migration` services; SQLite is a volume mount
- `init-muoxi-db.sql` — DELETED
- `.postgres-setup` — DELETED
- `dev-entrypoint.sh` — remove postgres wait + diesel cli install; SQLite needs no setup
- `README.md` — rewrite "Getting Started" without postgres
- `AGENTS.md` (root, `db/`, `muoxi/`) — reflect SQLite-first
- `.gitignore` — add `data/world.db*`

**Delete:**
- `muoxi/src/watchdog/` (entire directory)
- `migrations/00000000000000_diesel_initial_setup/` (PG-specific `diesel_set_updated_at` trigger function)
- `migrations/2020-01-21-170702_muoxi/` (replaced by new initial migration)
- `json/accounts.json`, `json/characters.json` — keep `json/` directory with a single `examples/seed.json` for documentation only
- `Dockerfile.postgres`, `init-muoxi-db.sql`, `.postgres-setup`

**Test:**
- `db/tests/integration_sqlite.rs` — CRUD round-trip against an in-memory SQLite
- `db/tests/integration_postgres.rs` — same suite gated on `db-postgres` feature (skipped by default)

---

## Task 1: Update workspace dependencies for SQLite-first

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Replace the workspace `[workspace.dependencies]` diesel + add libsqlite3-sys**

```toml
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
tokio-util = { version = "0.7", features = ["codec"] }
tokio-stream = "0.1"
futures = "0.3"
futures-util = "0.3"
diesel = { version = "2.2", default-features = false }
libsqlite3-sys = { version = "0.30", features = ["bundled"] }
redis = "0.27"
tokio-tungstenite = "0.24"
serde = { version = "1", features = ["derive"] }
serde_json = { version = "1", features = ["preserve_order"] }
log = "0.4"
pretty_env_logger = "0.5"
rand = "0.8"
uuid = { version = "1", features = ["serde", "v4"] }
async-trait = "0.1"
chrono = "0.4"
bytes = "1"
```

Note: `diesel = { version = "2.2", default-features = false }` — features are picked by the `db` crate per backend.

- [ ] **Step 2: Verify `cargo metadata` parses the manifest**

Run: `cd /home/duys/.repos/MuOxi && cargo metadata --no-deps --format-version 1 > /dev/null && echo OK`
Expected: `OK`

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "build: replace workspace diesel default with backend-feature selection"
```

---

## Task 2: Add backend feature flags to the `db` crate

**Files:**
- Modify: `db/Cargo.toml`

- [ ] **Step 1: Rewrite `db/Cargo.toml` with feature flags**

```toml
[package]
name = "db"
version.workspace = true
authors.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[lib]
name = "db"
path = "src/lib.rs"

[features]
default = ["db-sqlite"]
db-sqlite = ["diesel/sqlite", "diesel/returning_clauses_for_sqlite_3_35", "dep:libsqlite3-sys"]
db-postgres = ["diesel/postgres"]

[dependencies]
diesel = { workspace = true }
libsqlite3-sys = { workspace = true, optional = true }
redis = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
rand = { workspace = true }
uuid = { workspace = true }
```

- [ ] **Step 2: Verify exactly one backend is selected at a time**

The `db-sqlite` and `db-postgres` features are mutually exclusive at runtime (different connection types). We enforce this at compile time in `db/src/conn.rs` (Task 4). Acceptable for now.

- [ ] **Step 3: Commit**

```bash
git add db/Cargo.toml
git commit -m "build(db): add db-sqlite (default) and db-postgres feature flags"
```

---

## Task 3: Add `data/` directory + .gitignore entry

**Files:**
- Create: `data/.gitkeep`
- Modify: `.gitignore`

- [ ] **Step 1: Create the data directory**

```bash
mkdir -p /home/duys/.repos/MuOxi/data
touch /home/duys/.repos/MuOxi/data/.gitkeep
```

- [ ] **Step 2: Append SQLite ignore patterns to `.gitignore`**

Append these lines to `/home/duys/.repos/MuOxi/.gitignore`:

```
data/world.db
data/world.db-wal
data/world.db-shm
data/*.sqlite
data/*.sqlite-wal
data/*.sqlite-shm
```

- [ ] **Step 3: Commit**

```bash
git add data/.gitkeep .gitignore
git commit -m "chore: add data/ for SQLite world db, gitignore wal/shm files"
```

---

## Task 4: Create `db/src/conn.rs` — backend-agnostic connection type

**Files:**
- Create: `db/src/conn.rs`
- Modify: `db/src/lib.rs`

- [ ] **Step 1: Write `db/src/conn.rs`**

```rust
//! Backend-selectable connection type.
//!
//! Exactly one of the `db-sqlite` / `db-postgres` features must be enabled;
//! a compile_error! is emitted otherwise.

#[cfg(all(feature = "db-sqlite", feature = "db-postgres"))]
compile_error!(
    "db crate: features `db-sqlite` and `db-postgres` are mutually exclusive. \
     Pick one with --no-default-features --features db-postgres."
);

#[cfg(not(any(feature = "db-sqlite", feature = "db-postgres")))]
compile_error!(
    "db crate: enable exactly one of `db-sqlite` (default) or `db-postgres`."
);

use diesel::prelude::*;
use std::env;

#[cfg(feature = "db-sqlite")]
pub type Conn = diesel::sqlite::SqliteConnection;

#[cfg(feature = "db-postgres")]
pub type Conn = diesel::pg::PgConnection;

/// Default database URL used when `DATABASE_URL` is not set.
pub fn default_url() -> &'static str {
    #[cfg(feature = "db-sqlite")]
    {
        "data/world.db"
    }
    #[cfg(feature = "db-postgres")]
    {
        "postgres://muoxi:muoxi@localhost/muoxi"
    }
}

/// Open a new connection. Reads `DATABASE_URL` env var, falls back to
/// `default_url()`. Panics on failure.
pub fn establish() -> Conn {
    let url = env::var("DATABASE_URL").unwrap_or_else(|_| default_url().to_string());
    Conn::establish(&url).unwrap_or_else(|e| panic!("db::establish failed for {}: {}", url, e))
}

/// Apply runtime PRAGMAs that the SQLite backend needs.
/// No-op on Postgres.
#[cfg(feature = "db-sqlite")]
pub fn configure(conn: &mut Conn) -> QueryResult<()> {
    diesel::sql_query("PRAGMA journal_mode = WAL").execute(conn)?;
    diesel::sql_query("PRAGMA synchronous = NORMAL").execute(conn)?;
    diesel::sql_query("PRAGMA foreign_keys = ON").execute(conn)?;
    diesel::sql_query("PRAGMA busy_timeout = 5000").execute(conn)?;
    Ok(())
}

#[cfg(feature = "db-postgres")]
pub fn configure(_conn: &mut Conn) -> QueryResult<()> {
    Ok(())
}
```

- [ ] **Step 2: Add `pub mod conn;` to `db/src/lib.rs`**

In `/home/duys/.repos/MuOxi/db/src/lib.rs`, replace the existing module list with:

```rust
//! Diesel-powered ORM management library for MuOxi.

pub mod cache;
pub mod cache_structures;
pub mod conn;
pub mod schema;
pub mod structures;
pub mod utils;

pub use conn::{Conn, establish, configure, default_url};
```

Then update the `DatabaseHandler::connect` body:

```rust
use crate::conn::{Conn, establish, configure};

pub struct DatabaseHandler {
    pub handle: Conn,
    pub accounts: structures::account::AccountHandler,
    pub characters: structures::character::CharacterHandler,
}

impl DatabaseHandler {
    pub fn connect() -> Self {
        let mut handle = establish();
        configure(&mut handle).expect("configure() pragmas failed");
        Self {
            handle,
            accounts: structures::account::AccountHandler,
            characters: structures::character::CharacterHandler,
        }
    }
}
```

Remove the now-unused imports (`use diesel::pg::PgConnection;`, `use std::env;`).

- [ ] **Step 3: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p db`
Expected: errors only from `structures.rs` referencing `&mut PgConnection` (Task 6 fixes those). `lib.rs`/`conn.rs` themselves compile.

- [ ] **Step 4: Commit**

```bash
git add db/src/conn.rs db/src/lib.rs
git commit -m "feat(db): introduce backend-agnostic Conn type with WAL pragmas"
```

---

## Task 5: Write the new initial migration (portable schema, SQLite + Postgres)

**Files:**
- Delete: `migrations/00000000000000_diesel_initial_setup/`
- Delete: `migrations/2020-01-21-170702_muoxi/`
- Create: `migrations/2026-05-07-000000_initial/up.sql`
- Create: `migrations/2026-05-07-000000_initial/down.sql`

- [ ] **Step 1: Remove the old migrations**

```bash
cd /home/duys/.repos/MuOxi
rm -rf migrations/00000000000000_diesel_initial_setup
rm -rf migrations/2020-01-21-170702_muoxi
```

- [ ] **Step 2: Create the new migration directory**

```bash
mkdir -p /home/duys/.repos/MuOxi/migrations/2026-05-07-000000_initial
```

- [ ] **Step 3: Write `up.sql`**

```sql
-- accounts: an authenticatable user, separate from in-game character(s).
CREATE TABLE accounts (
    uid           BIGINT       NOT NULL CHECK (uid > 0),
    name          VARCHAR(64)  NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    email         VARCHAR(255) NOT NULL DEFAULT '',
    created_at    BIGINT       NOT NULL,
    PRIMARY KEY (uid)
);

CREATE INDEX idx_accounts_name ON accounts(name);

-- characters: an in-world avatar. One account can own many.
CREATE TABLE characters (
    uid         BIGINT       NOT NULL CHECK (uid > 0),
    account_uid BIGINT       NOT NULL CHECK (account_uid > 0),
    name        VARCHAR(64)  NOT NULL UNIQUE,
    created_at  BIGINT       NOT NULL,
    PRIMARY KEY (uid),
    FOREIGN KEY (account_uid) REFERENCES accounts(uid) ON DELETE CASCADE
);

CREATE INDEX idx_characters_account ON characters(account_uid);

-- account_characters: ordered membership view (replaces the BIGINT[] column).
-- The character.account_uid FK is the source of truth; this table just
-- preserves user-facing ordering ("character 1, 2, 3 in slot order").
CREATE TABLE account_characters (
    account_uid   BIGINT  NOT NULL,
    character_uid BIGINT  NOT NULL,
    ordinal       INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_uid, character_uid),
    FOREIGN KEY (account_uid) REFERENCES accounts(uid) ON DELETE CASCADE,
    FOREIGN KEY (character_uid) REFERENCES characters(uid) ON DELETE CASCADE
);

CREATE INDEX idx_account_characters_ordinal
    ON account_characters(account_uid, ordinal);
```

- [ ] **Step 4: Write `down.sql`**

```sql
DROP TABLE IF EXISTS account_characters;
DROP TABLE IF EXISTS characters;
DROP TABLE IF EXISTS accounts;
```

- [ ] **Step 5: Verify against SQLite by feeding the file into a fresh in-memory db**

```bash
cd /home/duys/.repos/MuOxi
rustup run stable bash -c 'cargo install --quiet diesel_cli --no-default-features --features sqlite-bundled || true'
sqlite3 :memory: < migrations/2026-05-07-000000_initial/up.sql && echo "schema OK"
```

Expected: `schema OK`. (If `sqlite3` is not installed, skip and rely on Task 11's integration test.)

- [ ] **Step 6: Commit**

```bash
git add migrations/2026-05-07-000000_initial/ migrations/00000000000000_diesel_initial_setup migrations/2020-01-21-170702_muoxi
git commit -m "feat(db): replace pg-specific migrations with portable initial schema"
```

---

## Task 6: Regenerate `db/src/schema.rs` for the new schema

**Files:**
- Modify: `db/src/schema.rs`

- [ ] **Step 1: Write the new `db/src/schema.rs` by hand (no diesel CLI dependency)**

```rust
diesel::table! {
    accounts (uid) {
        uid -> BigInt,
        name -> Text,
        password_hash -> Text,
        email -> Text,
        created_at -> BigInt,
    }
}

diesel::table! {
    characters (uid) {
        uid -> BigInt,
        account_uid -> BigInt,
        name -> Text,
        created_at -> BigInt,
    }
}

diesel::table! {
    account_characters (account_uid, character_uid) {
        account_uid -> BigInt,
        character_uid -> BigInt,
        ordinal -> Integer,
    }
}

diesel::joinable!(characters -> accounts (account_uid));
diesel::joinable!(account_characters -> accounts (account_uid));
diesel::joinable!(account_characters -> characters (character_uid));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    characters,
    account_characters,
);
```

Note: `BigInt` is the portable Diesel SQL type alias for both SQLite's `INTEGER`-as-i64 and Postgres's `BIGINT`/`Int8`. `Text` works for both `VARCHAR(N)` and Postgres `VARCHAR`.

- [ ] **Step 2: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p db`
Expected: errors come from `structures.rs` only (BIGINT[] field referenced + `&mut PgConnection`). The schema file itself compiles for both backends.

- [ ] **Step 3: Commit**

```bash
git add db/src/schema.rs
git commit -m "feat(db): rewrite schema.rs for portable BigInt/Text types"
```

---

## Task 7: Update `db/src/structures.rs` for the new schema and `Conn` type

**Files:**
- Modify: `db/src/structures.rs`

- [ ] **Step 1: Replace `PgConnection` with `Conn` and drop the `characters` array field**

Open `db/src/structures.rs`. Change the imports block at the top to:

```rust
#![deny(missing_docs)]

//! Diesel ORM models for MuOxi's stable core tables.

use crate::conn::Conn;
use crate::utils::UID;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::From;
use std::iter::FromIterator;
```

Find every occurrence of `&mut PgConnection` in the file and replace with `&mut Conn`. There are six occurrences in `DatabaseHandlerExt` trait methods plus implementations.

- [ ] **Step 2: Update the `Account` struct: drop the `characters: Option<Vec<i64>>` field, add `password_hash` + `created_at`**

```rust
#[derive(Queryable, Insertable, Debug, AsChangeset, Clone, Serialize, Deserialize)]
#[diesel(table_name = accounts)]
pub struct Account {
    /// unique id for each account
    pub uid: UID,
    /// account name (login identifier)
    pub name: String,
    /// hashed password (argon2id; bare blob to the DB)
    pub password_hash: String,
    /// email associated with account; empty string if not set
    pub email: String,
    /// unix epoch seconds when the account was created
    pub created_at: i64,
}
```

- [ ] **Step 3: Update the `Character` struct similarly**

```rust
#[derive(Queryable, Insertable, Debug, AsChangeset, Clone, Serialize, Deserialize)]
#[diesel(table_name = characters)]
pub struct Character {
    /// unique id for each character
    pub uid: UID,
    /// owning account UID
    pub account_uid: UID,
    /// character display name (must be globally unique)
    pub name: String,
    /// unix epoch seconds
    pub created_at: i64,
}
```

- [ ] **Step 4: Add a new `AccountCharacterLink` struct + handler for the join table**

After the `character` module, append:

```rust
/// Many-to-many link between `accounts` and `characters` with user-facing ordinal.
pub mod account_character_link {
    use super::super::schema::account_characters;
    use super::*;

    /// Join row.
    #[derive(Queryable, Insertable, Debug, Clone, Serialize, Deserialize)]
    #[diesel(table_name = account_characters)]
    pub struct AccountCharacter {
        /// account uid
        pub account_uid: UID,
        /// character uid
        pub character_uid: UID,
        /// display position (0-indexed) within the account's character list
        pub ordinal: i32,
    }

    /// CRUD for the join table.
    pub struct AccountCharacterHandler;

    impl AccountCharacterHandler {
        /// Add an existing character to an account at the next ordinal.
        pub fn link(
            &self,
            conn: &mut Conn,
            account_uid: UID,
            character_uid: UID,
        ) -> QueryResult<AccountCharacter> {
            use self::account_characters::dsl;
            let next_ord: i32 = dsl::account_characters
                .filter(dsl::account_uid.eq(account_uid))
                .select(diesel::dsl::max(dsl::ordinal))
                .first::<Option<i32>>(conn)?
                .unwrap_or(-1)
                + 1;
            let row = AccountCharacter {
                account_uid,
                character_uid,
                ordinal: next_ord,
            };
            diesel::insert_into(account_characters::table)
                .values(&row)
                .execute(conn)?;
            Ok(row)
        }

        /// Return all characters of an account, ordered by `ordinal`.
        pub fn list_for_account(
            &self,
            conn: &mut Conn,
            account_uid: UID,
        ) -> QueryResult<Vec<AccountCharacter>> {
            use self::account_characters::dsl;
            dsl::account_characters
                .filter(dsl::account_uid.eq(account_uid))
                .order(dsl::ordinal.asc())
                .load::<AccountCharacter>(conn)
        }
    }
}
```

- [ ] **Step 5: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p db --no-default-features --features db-sqlite`
Expected: `Finished` with no errors.

Run: `cd /home/duys/.repos/MuOxi && cargo check -p db --no-default-features --features db-postgres`
Expected: `Finished` with no errors. (If libpq isn't installed locally, this step is allowed to fail at the linker stage but `cargo check` is link-free, so it should still pass.)

- [ ] **Step 6: Commit**

```bash
git add db/src/structures.rs
git commit -m "refactor(db): port models to backend-agnostic Conn + drop array column"
```

---

## Task 8: Update `db/src/lib.rs` to wire `AccountCharacterHandler` into `DatabaseHandler`

**Files:**
- Modify: `db/src/lib.rs`

- [ ] **Step 1: Add the new handler field**

Replace the `DatabaseHandler` struct + impl with:

```rust
pub struct DatabaseHandler {
    /// actual connection to the database
    pub handle: Conn,
    /// handle to the Accounts table
    pub accounts: structures::account::AccountHandler,
    /// handle to the Characters table
    pub characters: structures::character::CharacterHandler,
    /// handle to the account_characters join table
    pub account_characters: structures::account_character_link::AccountCharacterHandler,
}

impl DatabaseHandler {
    /// Connect to the configured database and apply runtime pragmas.
    pub fn connect() -> Self {
        let mut handle = establish();
        configure(&mut handle).expect("configure() pragmas failed");
        Self {
            handle,
            accounts: structures::account::AccountHandler,
            characters: structures::character::CharacterHandler,
            account_characters:
                structures::account_character_link::AccountCharacterHandler,
        }
    }
}
```

- [ ] **Step 2: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p db`
Expected: `Finished` with no errors.

- [ ] **Step 3: Commit**

```bash
git add db/src/lib.rs
git commit -m "feat(db): expose AccountCharacterHandler on DatabaseHandler"
```

---

## Task 9: Delete the watchdog binary and JSON canonical paths

**Files:**
- Delete: `muoxi/src/watchdog/`
- Modify: `muoxi/Cargo.toml`
- Delete: `json/accounts.json`, `json/characters.json`
- Create: `json/README.md` (explains json/ is for examples only)

- [ ] **Step 1: Remove the binary source**

```bash
cd /home/duys/.repos/MuOxi
rm -rf muoxi/src/watchdog
```

- [ ] **Step 2: Remove the `[[bin]] muoxi_watchdog` entry from `muoxi/Cargo.toml` and the `hotwatch` dep**

Open `/home/duys/.repos/MuOxi/muoxi/Cargo.toml`. Delete:

```toml
hotwatch = { workspace = true }
```

And the entire block:

```toml
[[bin]]
name = "muoxi_watchdog"
path = "src/watchdog/watchdog.rs"
```

- [ ] **Step 3: Remove `hotwatch` from the workspace dependencies**

Open `/home/duys/.repos/MuOxi/Cargo.toml` and delete the line `hotwatch = "0.5"` from `[workspace.dependencies]`.

- [ ] **Step 4: Delete the JSON canonical files; replace with a README**

```bash
cd /home/duys/.repos/MuOxi
rm json/accounts.json json/characters.json
```

Write `/home/duys/.repos/MuOxi/json/README.md`:

```markdown
# json/ — Seed and Example Data

This directory is for **import / export** payloads only — not runtime canonical state.
The canonical store is the SQLite database at `data/world.db` (or Postgres when the
`db-postgres` feature is enabled).

Place sample worlds here for tests, demos, or migration scripts to consume. They are
not loaded automatically.
```

- [ ] **Step 5: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check --workspace`
Expected: `Finished` with no errors.

- [ ] **Step 6: Commit**

```bash
git add -A muoxi/src/watchdog muoxi/Cargo.toml Cargo.toml json/
git commit -m "feat: remove muoxi_watchdog binary and JSON-canonical files"
```

---

## Task 10: Update Docker stack — drop Postgres service, embed SQLite path

**Files:**
- Modify: `Dockerfile`
- Delete: `Dockerfile.postgres`, `init-muoxi-db.sql`, `.postgres-setup`
- Modify: `docker-compose.yml`
- Modify: `dev-entrypoint.sh`

- [ ] **Step 1: Replace the Dockerfile with a minimal modern image**

Overwrite `/home/duys/.repos/MuOxi/Dockerfile`:

```dockerfile
FROM rust:1.85-slim AS builder
ARG MUOXI_INSTALL_DIR=/opt/muoxi
ENV LANG=C.UTF-8 MUOXI_INSTALL_DIR=${MUOXI_INSTALL_DIR}
WORKDIR ${MUOXI_INSTALL_DIR}

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY . ${MUOXI_INSTALL_DIR}
RUN cargo build --release --workspace

FROM debian:bookworm-slim AS runtime
ARG MUOXI_INSTALL_DIR=/opt/muoxi
ENV MUOXI_INSTALL_DIR=${MUOXI_INSTALL_DIR}
WORKDIR ${MUOXI_INSTALL_DIR}

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    useradd -r -M -d ${MUOXI_INSTALL_DIR} muoxi

COPY --from=builder ${MUOXI_INSTALL_DIR}/target/release/muoxi_server /usr/local/bin/muoxi_server
COPY --from=builder ${MUOXI_INSTALL_DIR}/target/release/muoxi_web /usr/local/bin/muoxi_web
COPY --from=builder ${MUOXI_INSTALL_DIR}/migrations ${MUOXI_INSTALL_DIR}/migrations
COPY --from=builder ${MUOXI_INSTALL_DIR}/resources ${MUOXI_INSTALL_DIR}/resources

USER muoxi
EXPOSE 8000 8080
CMD ["muoxi_server"]
```

Note: this assumes Plan 2 has merged staging+engine into `muoxi_server`. If you execute Plan 1 before Plan 2, temporarily replace `muoxi_server` with `muoxi_staging` and update again in Plan 2.

- [ ] **Step 2: Delete `Dockerfile.postgres`, `init-muoxi-db.sql`, `.postgres-setup`**

```bash
cd /home/duys/.repos/MuOxi
rm -f Dockerfile.postgres init-muoxi-db.sql .postgres-setup
```

- [ ] **Step 3: Rewrite `docker-compose.yml`**

```yaml
version: "3.7"

services:
  server:
    build:
      context: .
      dockerfile: Dockerfile
      target: runtime
    image: muoxi:dev
    volumes:
      - muoxi_data:/opt/muoxi/data
    environment:
      DATABASE_URL: /opt/muoxi/data/world.db
      RUST_LOG: info
    ports:
      - "${MUOXI_SERVER_PORT:-8000}:8000"
      - "${MUOXI_WEB_PORT:-8080}:8080"
    depends_on:
      - redis
    networks:
      - backend

  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    networks:
      - backend

volumes:
  muoxi_data:
  redis_data:

networks:
  backend:
```

- [ ] **Step 4: Replace `dev-entrypoint.sh` with a no-op-friendly version (no postgres wait, no `cargo install`)**

```bash
#!/bin/sh
set -e
exec "$@"
```

Make it executable: `chmod +x /home/duys/.repos/MuOxi/dev-entrypoint.sh`. Or delete it entirely and adjust the Dockerfile if you no longer need an entrypoint wrapper.

- [ ] **Step 5: Verify the compose file parses**

Run: `cd /home/duys/.repos/MuOxi && docker compose config > /dev/null && echo OK`
Expected: `OK`. (Skip if Docker isn't installed; the file is small and obvious.)

- [ ] **Step 6: Commit**

```bash
git add Dockerfile Dockerfile.postgres init-muoxi-db.sql .postgres-setup docker-compose.yml dev-entrypoint.sh
git commit -m "build(docker): drop postgres service, switch to SQLite + redis only"
```

---

## Task 11: Add an integration test that round-trips an Account against in-memory SQLite

**Files:**
- Create: `db/tests/integration_sqlite.rs`

- [ ] **Step 1: Write the integration test**

```rust
//! End-to-end CRUD test on an in-memory SQLite database.
//!
//! Runs only with the `db-sqlite` feature (the default). Skipped otherwise.

#![cfg(feature = "db-sqlite")]

use db::structures::account::{Account, AccountHandler};
use db::structures::DatabaseHandlerExt;
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
                .unwrap_or_else(|e| panic!("running schema stmt failed: {} | stmt={}", e, trimmed));
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
```

- [ ] **Step 2: Run it**

```bash
cd /home/duys/.repos/MuOxi && cargo test -p db --features db-sqlite
```

Expected: `test account_roundtrip ... ok` and overall `test result: ok. 1 passed`.

- [ ] **Step 3: Commit**

```bash
git add db/tests/integration_sqlite.rs
git commit -m "test(db): SQLite in-memory CRUD roundtrip for Account"
```

---

## Task 12: Update tester crate to compile against SQLite default

**Files:**
- Modify: `tester/Cargo.toml`
- Modify: `tester/src/main.rs`

- [ ] **Step 1: Tester only uses Redis + db-via-cache, doesn't need diesel directly**

Open `/home/duys/.repos/MuOxi/tester/Cargo.toml` and remove `diesel = { workspace = true }`. The remaining dep on `db = { path = "../db" }` will compile against whichever backend the workspace selected.

- [ ] **Step 2: Make sure `tester/src/main.rs` doesn't import any `diesel::` paths directly**

Open `/home/duys/.repos/MuOxi/tester/src/main.rs`. Confirm the `use` lines reference only `db::cache`, `db::cache_structures`, `redis`, `serde`, `serde_json`. If any `use diesel::*` or similar remains, delete it.

- [ ] **Step 3: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p tester`
Expected: `Finished`.

- [ ] **Step 4: Commit**

```bash
git add tester/Cargo.toml tester/src/main.rs
git commit -m "build(tester): drop direct diesel dep; rely on db crate"
```

---

## Task 13: Update README's "Getting Started" — drop all Postgres steps

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Replace the entire setup section**

Find the heading `## Getting Started` in `/home/duys/.repos/MuOxi/README.md` and replace from there through the end of the `### Install Diesel Cli for migrations and database management` block with:

```markdown
## Getting Started

MuOxi uses SQLite by default — no external database service required. The default
build needs only `cargo` and a Rust toolchain. The `rust-toolchain.toml` file pins
the project to the matching stable channel.

### Quick start

```bash
git clone https://github.com/duysqubix/MuOxi.git
cd MuOxi
cargo run --bin muoxi_server          # binds 127.0.0.1:8000 (telnet)
# in another terminal:
cargo run --bin muoxi_web             # binds 127.0.0.1:8080 (websocket)
```

The first run creates `data/world.db` (override with `DATABASE_URL`). Migrations
run automatically on startup. Connect with any telnet client:

```bash
telnet 127.0.0.1 8000
```

### Optional: Postgres backend

For larger deployments, opt into the Postgres backend at compile time:

```bash
sudo apt install libpq-dev                    # or your platform's libpq package
cargo build --no-default-features --features db-postgres
DATABASE_URL=postgres://muoxi:muoxi@localhost/muoxi cargo run --bin muoxi_server
```

You'll need to provision the Postgres database yourself (`createdb muoxi`,
`createuser muoxi`, etc.). The same migrations under `migrations/` apply to both
backends.

### Optional: Redis (transient session cache)

MuOxi caches per-connection socket state in Redis when available, but starts up
fine without it (cache writes become no-ops). To enable, run:

```bash
redis-server                                  # default port 6379
REDIS_SERVER=redis://127.0.0.1 cargo run --bin muoxi_server
```

### Docker

```bash
docker compose up server
```
```

- [ ] **Step 2: Remove the now-stale "Database Design Architecture" section's mention of MongoDB**

Find and delete the bullet about MongoDB in the watchdog description (already done in modernization, but double-check).

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs(readme): rewrite Getting Started for SQLite-default workflow"
```

---

## Task 14: Update root + db AGENTS.md

**Files:**
- Modify: `AGENTS.md`
- Modify: `db/AGENTS.md`

- [ ] **Step 1: Update the OVERVIEW + STRUCTURE + COMMANDS sections of root `AGENTS.md`**

In `/home/duys/.repos/MuOxi/AGENTS.md`:

- Change the OVERVIEW line "Tokio 1.x + Diesel 2.x + Redis 0.27 + tokio-tungstenite" to "Tokio 1.x + Diesel 2.x (SQLite default, Postgres optional) + Redis 0.27 + tokio-tungstenite".
- In the STRUCTURE block, remove the `muoxi_watchdog` entry under `muoxi/` and the `dev-entrypoint.sh` / `init-muoxi-db.sql` lines.
- Replace the `cargo run --bin muoxi_watchdog` line in COMMANDS with: `cargo run --bin muoxi_server  # combined staging+engine, 127.0.0.1:8000` (Plan 2 will add this binary).
- In ENV VARS, change `DATABASE_URL`'s default to `data/world.db` and add a note `(SQLite path; for Postgres set to postgres://...)`.
- In NOTES, add: `**Default backend is SQLite**. Building with `--no-default-features --features db-postgres` selects Postgres (requires libpq).` Remove the existing libpq notes since they no longer apply to default builds.

- [ ] **Step 2: Rewrite `db/AGENTS.md` STRUCTURE + CONVENTIONS sections to match the new code**

Replace the file's STRUCTURE listing and "DATABASE SCHEMA" section to describe the new schema (`accounts`, `characters`, `account_characters`) and remove all references to `BIGINT[]`. Add a "BACKENDS" section summarizing the `db-sqlite` / `db-postgres` feature pair.

(Concrete diffs are mechanical given the new schema in Task 5; either subagent or human can produce.)

- [ ] **Step 3: Commit**

```bash
git add AGENTS.md db/AGENTS.md
git commit -m "docs(agents): SQLite default; portable schema; remove watchdog"
```

---

## Task 15: Final cross-cutting verification

**Files:** none (verification only).

- [ ] **Step 1: Build the workspace with default features (SQLite)**

```bash
cd /home/duys/.repos/MuOxi
rm -rf target
cargo build --workspace 2>&1 | tail -10
```

Expected: `Finished` and binaries `muoxi_engine`, `muoxi_staging`, `muoxi_web`, `muoxi_sandbox`, `muoxi_benchmarks` exist in `target/debug/`. **No `libpq` link errors.** (`muoxi_watchdog` no longer exists — that's the goal.)

- [ ] **Step 2: Run the integration test**

```bash
cargo test -p db --features db-sqlite 2>&1 | tail -10
```

Expected: `test result: ok. 1 passed; 0 failed`.

- [ ] **Step 3: Build with the Postgres feature to confirm the alternative backend still type-checks**

```bash
cargo check -p db --no-default-features --features db-postgres 2>&1 | tail -5
```

Expected: `Finished` with no errors. (Linker may fail without libpq; that's acceptable for `cargo check`.)

- [ ] **Step 4: Smoke-test the engine binary**

```bash
cd /home/duys/.repos/MuOxi
./target/debug/muoxi_engine &
ENGINE_PID=$!
sleep 1
echo "ping" | timeout 3 nc -q1 127.0.0.1 4567
kill $ENGINE_PID 2>/dev/null
```

Expected: `Game Server > ping`.

- [ ] **Step 5: Commit any verification-driven fixups**

If any verification step needed a fix, commit it now with a `fix:` prefix.

---

## Verification Summary

A successful run of this plan ends with:

- [ ] No `libpq-dev` requirement for default builds.
- [ ] `data/world.db` is created on first run; gitignored.
- [ ] `accounts` and `characters` tables join via `account_characters`.
- [ ] `muoxi_watchdog` binary, `json/*.json` runtime files, `Dockerfile.postgres`, `init-muoxi-db.sql`, `.postgres-setup`, and old migrations are deleted.
- [ ] `cargo build --workspace` succeeds without system-package prerequisites.
- [ ] `cargo test -p db` passes the SQLite roundtrip test.
- [ ] Postgres backend still compiles with `--no-default-features --features db-postgres`.
- [ ] README reflects the SQLite-first workflow.
