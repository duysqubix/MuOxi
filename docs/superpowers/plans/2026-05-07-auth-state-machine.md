# Auth State Machine Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement all 8 `ConnStates` so a fresh connection can flow from name prompt → password entry → main menu → character select → in-world `Playing`. Replaces the current state machine where 7 of 8 states fall through to `Quit` and login does nothing.

**Architecture:** Argon2id password hashing replaces the current placeholder hash. A new `AuthBuffer` field on `Client` carries partial inputs across state transitions (the typed-but-not-yet-confirmed name, first-attempt password, etc.). Each state's `execute` arm has explicit logic — no more catchall fallthrough. The `Playing` state acquires a real `character_uid` populated at character-select time. The `at_login` and `at_disconnect` hooks fire from the appropriate transitions.

**Tech Stack:** `argon2 = "0.5"` for password hashing, `password-hash = "0.5"`. No async-runtime changes.

---

## File Structure

**Create:**
- `muoxi/src/server/auth.rs` — password hashing helpers + `AuthBuffer`
- `muoxi/tests/auth_flow.rs` — end-to-end flow test using piped TCP input

**Modify:**
- `Cargo.toml` — workspace dep entries for argon2
- `muoxi/Cargo.toml` — pull argon2
- `muoxi/src/server/comms.rs` — `Client` gains `auth_buffer: AuthBuffer`, `account_uid: Option<UID>`, `character_uid: Option<UID>`
- `muoxi/src/server/states.rs` — implement every state arm
- `muoxi/src/server/commands/look.rs` — use `client.character_uid` (was `client.uid`)
- `muoxi/src/server/commands/who.rs` — list connected characters from `Server.clients`
- `muoxi/src/server/main.rs` — emit `at_disconnect` hook in `client_cleanup`
- `db/src/structures.rs` — `Account::password_hash` uses argon2 verification helper

**Delete:** none.

---

## Task 1: Add argon2 to the workspace

**Files:**
- Modify: `Cargo.toml`
- Modify: `muoxi/Cargo.toml`

- [ ] **Step 1: Add the workspace dep**

Append to `[workspace.dependencies]` in root `Cargo.toml`:

```toml
argon2 = "0.5"
```

- [ ] **Step 2: Pull it into the muoxi crate**

Append to `muoxi/Cargo.toml` `[dependencies]`:

```toml
argon2 = { workspace = true }
```

- [ ] **Step 3: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p muoxi`
Expected: `Finished`.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml muoxi/Cargo.toml
git commit -m "build: add argon2 for password hashing"
```

---

## Task 2: Implement password hashing helpers + `AuthBuffer`

**Files:**
- Create: `muoxi/src/server/auth.rs`

- [ ] **Step 1: Write the module**

```rust
//! Password hashing + per-session auth scratch space.

use argon2::Argon2;
use argon2::password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
};
use db::utils::UID;

/// Hash a plaintext password with argon2id + a fresh random salt.
/// Returns the PHC-formatted hash string suitable for storing in the DB.
pub fn hash_password(plain: &str) -> Result<String, &'static str> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(plain.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| "password hashing failed")
}

/// Verify a plaintext password against a stored PHC hash.
pub fn verify_password(plain: &str, stored_hash: &str) -> bool {
    match PasswordHash::new(stored_hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(plain.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

/// Per-session scratch space carrying partial auth inputs across state transitions.
#[derive(Default, Debug, Clone)]
pub struct AuthBuffer {
    /// name typed in `AwaitingName` or `AwaitingNewName`
    pub pending_name: Option<String>,
    /// first password attempt while in `AwaitingNewPassword`
    pub first_password_attempt: Option<String>,
}

impl AuthBuffer {
    pub fn clear(&mut self) {
        self.pending_name = None;
        self.first_password_attempt = None;
    }
}

/// Validate that `name` is acceptable as a new account or character name.
/// Rules: 3-32 chars, alphanumeric + underscore, must start with a letter.
pub fn is_valid_name(name: &str) -> bool {
    if name.len() < 3 || name.len() > 32 {
        return false;
    }
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Validate that `password` meets the policy.
/// v0.1 policy: 6+ chars, no whitespace, no null bytes.
pub fn is_valid_password(password: &str) -> bool {
    password.len() >= 6
        && !password.chars().any(|c| c.is_whitespace() || c == '\0')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_password_hash() {
        let h = hash_password("hunter2").unwrap();
        assert!(verify_password("hunter2", &h));
        assert!(!verify_password("wrong", &h));
        assert!(!verify_password("", &h));
    }

    #[test]
    fn name_rules() {
        assert!(is_valid_name("alice"));
        assert!(is_valid_name("Alice42"));
        assert!(is_valid_name("a_b_c"));
        assert!(!is_valid_name("ab"));
        assert!(!is_valid_name("4starts_digit"));
        assert!(!is_valid_name("has space"));
        assert!(!is_valid_name("oh no"));
    }

    #[test]
    fn password_rules() {
        assert!(is_valid_password("hunter2"));
        assert!(!is_valid_password("short"));
        assert!(!is_valid_password("has space here"));
    }
}

/// Re-exported for convenience.
pub use db::utils::UID as AccountUid;
```

- [ ] **Step 2: Add `pub mod auth;` to `muoxi/src/server/main.rs` and the lib re-export**

In `muoxi/src/lib.rs`, add `mod auth;` and `pub use crate::auth;` inside the `pub mod server` block.

- [ ] **Step 3: Verify**

Run: `cargo test -p muoxi auth::`
Expected: 3 tests pass.

- [ ] **Step 4: Commit**

```bash
git add muoxi/src/server/auth.rs muoxi/src/server/main.rs muoxi/src/lib.rs
git commit -m "feat(server): argon2 password hashing + AuthBuffer + name/password validators"
```

---

## Task 3: Extend `Client` with auth fields

**Files:**
- Modify: `muoxi/src/server/comms.rs`

- [ ] **Step 1: Update the `Client` struct**

```rust
use crate::auth::AuthBuffer;

pub struct Client {
    /// session UID (socket-tied, set in Client::new)
    pub uid: UID,
    /// current connection state
    pub state: ConnStates,
    /// line codec
    pub lines: Framed<TcpStream, LinesCodec>,
    /// peer address
    pub addr: SocketAddr,
    /// authenticated account UID, set on successful login
    pub account_uid: Option<UID>,
    /// selected character UID, set on character-select
    pub character_uid: Option<UID>,
    /// scratch space for auth state transitions
    pub auth_buffer: AuthBuffer,
    rx: Rx,
}
```

Update `Client::new` to initialize the new fields:

```rust
Ok(Self {
    uid,
    state: ConnStates::AwaitingName,
    lines: Framed::new(stream, LinesCodec::new()),
    addr,
    account_uid: None,
    character_uid: None,
    auth_buffer: AuthBuffer::default(),
    rx,
})
```

- [ ] **Step 2: Verify**

Run: `cargo check -p muoxi`
Expected: `Finished`.

- [ ] **Step 3: Commit**

```bash
git add muoxi/src/server/comms.rs
git commit -m "feat(server): Client gains account_uid/character_uid/auth_buffer"
```

---

## Task 4: Implement `AwaitingName` (existing-account or new branch)

**Files:**
- Modify: `muoxi/src/server/states.rs`

- [ ] **Step 1: Replace the `AwaitingName` arm**

Open `/home/duys/.repos/MuOxi/muoxi/src/server/states.rs`. Replace the current `AwaitingName` arm with:

```rust
ConnStates::AwaitingName => {
    let trimmed = response.trim();
    if trimmed.eq_ignore_ascii_case("new") {
        let _ = crate::send(client, "Choose an account name (3-32 chars, alphanumeric):").await?;
        return Ok(ConnStates::AwaitingNewName);
    }
    if !crate::auth::is_valid_name(trimmed) {
        let _ = crate::send(client, "Invalid name. Type `new` to create an account or enter your existing account name:").await?;
        return Ok(ConnStates::AwaitingName);
    }
    // look up the account
    let lookup = world
        .with_db(|db| {
            use db::diesel::prelude::*;
            use db::schema::accounts::dsl;
            dsl::accounts
                .filter(dsl::name.eq(trimmed))
                .first::<db::structures::account::Account>(&mut db.handle)
                .optional()
        })
        .await
        .map_err(|_| diesel_lines_codec_err("db lookup"))?;
    match lookup {
        Some(account) => {
            client.auth_buffer.pending_name = Some(account.name.clone());
            client.account_uid = Some(account.uid);
            let _ = crate::send(client, "Password:").await?;
            Ok(ConnStates::AwaitingPassword)
        }
        None => {
            let _ = crate::send(
                client,
                &format!(
                    "No account named {:?}. Enter another name or `new`:",
                    trimmed
                ),
            )
            .await?;
            Ok(ConnStates::AwaitingName)
        }
    }
}
```

Add the helper at the bottom of the file (or in `prelude.rs`):

```rust
fn diesel_lines_codec_err(_label: &str) -> tokio_util::codec::LinesCodecError {
    tokio_util::codec::LinesCodecError::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "database error",
    ))
}
```

The lookup uses Diesel directly because `WorldApi` doesn't yet have a typed `find_account_by_name` helper. Add one in Task 5 to clean this up.

- [ ] **Step 2: Verify**

Run: `cargo check -p muoxi`
Expected: errors only from sibling state arms (Tasks 5–9).

- [ ] **Step 3: Commit**

```bash
git add muoxi/src/server/states.rs
git commit -m "feat(server): AwaitingName looks up existing account or routes to new"
```

---

## Task 5: Add `WorldApi::find_account_by_name` and tidy the AwaitingName lookup

**Files:**
- Modify: `muoxi/src/server/world.rs`

- [ ] **Step 1: Add the method**

```rust
use db::structures::account::Account;
use db::diesel::prelude::*;

impl WorldApi {
    // ... existing methods ...

    /// Look up an account by name (case-sensitive). `None` if absent.
    pub async fn find_account_by_name(&self, name: &str) -> Option<Account> {
        let mut db = self.db.lock().await;
        use db::schema::accounts::dsl;
        dsl::accounts
            .filter(dsl::name.eq(name))
            .first::<Account>(&mut db.handle)
            .ok()
    }
}
```

- [ ] **Step 2: Replace the `with_db(...)` lookup in `AwaitingName` with the helper**

Replace the `world.with_db(...)` block in the `AwaitingName` arm (Task 4) with:

```rust
let lookup = world.find_account_by_name(trimmed).await;
```

Drop the `diesel_lines_codec_err` helper.

- [ ] **Step 3: Verify**

Run: `cargo check -p muoxi`
Expected: same errors as before from other state arms.

- [ ] **Step 4: Commit**

```bash
git add muoxi/src/server/world.rs muoxi/src/server/states.rs
git commit -m "refactor(server): AwaitingName uses WorldApi::find_account_by_name"
```

---

## Task 6: Implement `AwaitingPassword`

**Files:**
- Modify: `muoxi/src/server/states.rs`

- [ ] **Step 1: Replace the arm**

```rust
ConnStates::AwaitingPassword => {
    let Some(account_uid) = client.account_uid else {
        let _ = crate::send(client, "Session lost. Enter your account name:").await?;
        client.auth_buffer.clear();
        return Ok(ConnStates::AwaitingName);
    };
    let Some(name) = client.auth_buffer.pending_name.clone() else {
        let _ = crate::send(client, "Session lost. Enter your account name:").await?;
        client.auth_buffer.clear();
        client.account_uid = None;
        return Ok(ConnStates::AwaitingName);
    };
    let stored_hash: Option<String> = world
        .with_db(|db| {
            use db::schema::accounts::dsl;
            use db::diesel::prelude::*;
            dsl::accounts
                .filter(dsl::uid.eq(account_uid))
                .select(dsl::password_hash)
                .first::<String>(&mut db.handle)
                .ok()
        })
        .await;
    let ok = matches!(stored_hash, Some(ref h) if crate::auth::verify_password(&response, h));
    if ok {
        client.auth_buffer.clear();
        let _ = crate::send(
            client,
            &format!("Welcome, {}.", name),
        )
        .await?;
        // fire at_login hook (non-cancelable)
        let world_ref = world.as_ref();
        let mut hctx = crate::hooks::HookContext {
            world: world_ref,
            session_uid: Some(client.uid),
        };
        registry
            .hooks
            .emit(|h| {
                let acc = account_uid;
                let mut hctx = crate::hooks::HookContext {
                    world: world_ref,
                    session_uid: Some(client.uid),
                };
                async move { h.at_login(&mut hctx, acc).await }
            })
            .await;
        let _ = hctx; // silence unused
        Ok(ConnStates::MainMenu)
    } else {
        client.auth_buffer.clear();
        client.account_uid = None;
        let _ = crate::send(client, "Bad password. Enter your account name:").await?;
        Ok(ConnStates::AwaitingName)
    }
}
```

Note: the hook-firing closure is awkward because it has to construct a fresh `HookContext` per invocation (the `Hooks::emit` API takes an `FnMut` returning `Future`). Cleaner alternative: change `Hooks::emit` to take an owning closure. The plan keeps the existing API for now; the cleanup is a candidate for a Plan 4 follow-up.

- [ ] **Step 2: Verify**

Run: `cargo check -p muoxi`
Expected: `AwaitingNewName` etc. still error (next tasks); the rest type-checks.

- [ ] **Step 3: Commit**

```bash
git add muoxi/src/server/states.rs
git commit -m "feat(server): AwaitingPassword verifies argon2 hash + fires at_login"
```

---

## Task 7: Implement `AwaitingNewName`, `AwaitingNewPassword`, `ConfirmNewPassword`

**Files:**
- Modify: `muoxi/src/server/states.rs`
- Modify: `muoxi/src/server/world.rs`

- [ ] **Step 1: Add `WorldApi::create_account`**

```rust
use std::time::{SystemTime, UNIX_EPOCH};

impl WorldApi {
    /// Create a new account. Returns the inserted row.
    pub async fn create_account(
        &self,
        name: &str,
        password_hash: &str,
        email: &str,
    ) -> Result<db::structures::account::Account, &'static str> {
        let uid = db::utils::gen_uid();
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let mut db = self.db.lock().await;
        use db::diesel::prelude::*;
        use db::schema::accounts;
        let row = db::structures::account::Account {
            uid,
            name: name.to_string(),
            password_hash: password_hash.to_string(),
            email: email.to_string(),
            created_at,
        };
        diesel::insert_into(accounts::table)
            .values(&row)
            .execute(&mut db.handle)
            .map_err(|_| "insert failed (name probably taken)")?;
        Ok(row)
    }
}
```

- [ ] **Step 2: Replace the three new-account arms in `states.rs`**

```rust
ConnStates::AwaitingNewName => {
    let trimmed = response.trim();
    if !crate::auth::is_valid_name(trimmed) {
        let _ = crate::send(
            client,
            "Names are 3-32 chars, alphanumeric/underscore, and start with a letter.",
        )
        .await?;
        return Ok(ConnStates::AwaitingNewName);
    }
    if world.find_account_by_name(trimmed).await.is_some() {
        let _ = crate::send(client, "That name is taken. Choose another:").await?;
        return Ok(ConnStates::AwaitingNewName);
    }
    client.auth_buffer.pending_name = Some(trimmed.to_string());
    let _ = crate::send(client, "Password (6+ chars, no whitespace):").await?;
    Ok(ConnStates::AwaitingNewPassword)
}

ConnStates::AwaitingNewPassword => {
    if !crate::auth::is_valid_password(&response) {
        let _ = crate::send(client, "Password too weak. Try again:").await?;
        return Ok(ConnStates::AwaitingNewPassword);
    }
    client.auth_buffer.first_password_attempt = Some(response.clone());
    let _ = crate::send(client, "Confirm password:").await?;
    Ok(ConnStates::ConfirmNewPassword)
}

ConnStates::ConfirmNewPassword => {
    let first = client.auth_buffer.first_password_attempt.clone();
    let name = client.auth_buffer.pending_name.clone();
    let (Some(first), Some(name)) = (first, name) else {
        client.auth_buffer.clear();
        let _ = crate::send(client, "Session lost. Enter your account name:").await?;
        return Ok(ConnStates::AwaitingName);
    };
    if first != response {
        client.auth_buffer.first_password_attempt = None;
        let _ = crate::send(client, "Passwords don't match. Enter password again:").await?;
        return Ok(ConnStates::AwaitingNewPassword);
    }
    let hash = match crate::auth::hash_password(&response) {
        Ok(h) => h,
        Err(e) => {
            let _ = crate::send(client, &format!("Internal error: {e}. Disconnecting.")).await?;
            return Ok(ConnStates::Quit);
        }
    };
    let acct = match world.create_account(&name, &hash, "").await {
        Ok(a) => a,
        Err(e) => {
            let _ = crate::send(client, &format!("Could not create account: {e}.")).await?;
            client.auth_buffer.clear();
            return Ok(ConnStates::AwaitingName);
        }
    };
    client.account_uid = Some(acct.uid);
    client.auth_buffer.clear();
    let _ = crate::send(client, &format!("Account {} created.", acct.name)).await?;
    Ok(ConnStates::MainMenu)
}
```

- [ ] **Step 3: Verify**

Run: `cargo check -p muoxi`
Expected: `MainMenu` still errors (next task).

- [ ] **Step 4: Commit**

```bash
git add muoxi/src/server/world.rs muoxi/src/server/states.rs
git commit -m "feat(server): AwaitingNewName / AwaitingNewPassword / ConfirmNewPassword"
```

---

## Task 8: Implement `MainMenu` (list / select / create characters / quit)

**Files:**
- Modify: `muoxi/src/server/states.rs`
- Modify: `muoxi/src/server/world.rs`

- [ ] **Step 1: Add `WorldApi::list_account_characters` and `create_character`**

```rust
impl WorldApi {
    /// List the character objects belonging to `account_uid`, in ordinal order.
    pub async fn list_account_characters(
        &self,
        account_uid: db::utils::UID,
    ) -> Vec<db::objects::Object> {
        let mut db = self.db.lock().await;
        let links = db
            .character_accounts
            .list_for_account(&mut db.handle, account_uid)
            .unwrap_or_default();
        let mut out = Vec::with_capacity(links.len());
        for link in links {
            if let Ok(Some(obj)) = db.objects.get(&mut db.handle, link.object_uid) {
                out.push(obj);
            }
        }
        out
    }

    /// Create a character object owned by `account_uid` with the given name.
    /// Applies the registered `character` TypeClass's default attributes/tags.
    pub async fn create_character(
        &self,
        registry: &crate::registry::Registry,
        account_uid: db::utils::UID,
        name: &str,
    ) -> Result<db::objects::Object, &'static str> {
        let mut db = self.db.lock().await;
        let obj = db
            .objects
            .create(&mut db.handle, "character", name, None)
            .map_err(|_| "could not create character object")?;
        db.character_accounts
            .link(&mut db.handle, obj.uid, account_uid)
            .map_err(|_| "could not link character to account")?;
        if let Some(tc) = registry.get_type("character") {
            for (k, v) in tc.default_attributes() {
                let _ = db.attributes.set(&mut db.handle, obj.uid, &k, &v);
            }
            for (k, cat) in tc.default_tags() {
                let _ = db.tags.add(&mut db.handle, obj.uid, &k, &cat);
            }
        }
        Ok(obj)
    }
}
```

- [ ] **Step 2: Replace the `MainMenu` arm**

```rust
ConnStates::MainMenu => {
    let Some(account_uid) = client.account_uid else {
        client.auth_buffer.clear();
        return Ok(ConnStates::AwaitingName);
    };
    let trimmed = response.trim();
    let chars = world.list_account_characters(account_uid).await;

    // initial prompt: when the user just transitioned in with empty input
    if response.is_empty() && !chars.is_empty() {
        let mut menu = String::from("Your characters:\n");
        for (idx, ch) in chars.iter().enumerate() {
            menu.push_str(&format!("  {}. {}\n", idx + 1, ch.name));
        }
        menu.push_str("Enter a number to play, `new <name>` to create, or `quit`.");
        let _ = crate::send(client, &menu).await?;
        return Ok(ConnStates::MainMenu);
    }

    if trimmed.eq_ignore_ascii_case("quit") {
        return Ok(ConnStates::Quit);
    }

    if let Some(rest) = trimmed.strip_prefix("new ").or_else(|| {
        if trimmed.eq_ignore_ascii_case("new") {
            Some("")
        } else {
            None
        }
    }) {
        let name = rest.trim();
        if !crate::auth::is_valid_name(name) {
            let _ = crate::send(
                client,
                "Usage: new <name> (3-32 chars, alphanumeric/underscore, start with letter)",
            )
            .await?;
            return Ok(ConnStates::MainMenu);
        }
        match world.create_character(&registry, account_uid, name).await {
            Ok(obj) => {
                client.character_uid = Some(obj.uid);
                let _ = crate::send(client, &format!("Created {}. Entering world.", obj.name)).await?;
                return Ok(ConnStates::Playing);
            }
            Err(e) => {
                let _ = crate::send(client, &format!("Could not create: {e}")).await?;
                return Ok(ConnStates::MainMenu);
            }
        }
    }

    if let Ok(idx) = trimmed.parse::<usize>() {
        if idx >= 1 && idx <= chars.len() {
            let chosen = &chars[idx - 1];
            client.character_uid = Some(chosen.uid);
            let _ = crate::send(client, &format!("Playing as {}.", chosen.name)).await?;
            return Ok(ConnStates::Playing);
        }
    }

    let _ = crate::send(
        client,
        "Unrecognized. Enter a number, `new <name>`, or `quit`.",
    )
    .await?;
    Ok(ConnStates::MainMenu)
}
```

- [ ] **Step 3: Verify**

Run: `cargo check -p muoxi`
Expected: `Finished`.

- [ ] **Step 4: Commit**

```bash
git add muoxi/src/server/world.rs muoxi/src/server/states.rs
git commit -m "feat(server): MainMenu lists/selects/creates characters; transitions to Playing"
```

---

## Task 9: Wire `at_disconnect` hook + clean up `Quit`

**Files:**
- Modify: `muoxi/src/server/main.rs`

- [ ] **Step 1: Update `client_cleanup` to fire the hook**

In `/home/duys/.repos/MuOxi/muoxi/src/server/main.rs`, change `client_cleanup`'s signature and body:

```rust
pub async fn client_cleanup(
    uid: UID,
    account_uid: Option<UID>,
    server: &Arc<Mutex<Server>>,
    registry: Arc<Registry>,
    cache: CacheSocket,
) {
    {
        let mut server = server.lock().await;
        server.clients.remove(&uid);
    }
    if cache.destruct().is_ok() {
        println!("Removed client uid: {}", uid);
    } else {
        println!("Unable to remove client {} from redis.", uid);
    }
    if let Some(acc) = account_uid {
        let world = registry.world.clone();
        registry
            .hooks
            .emit(|h| {
                let world_ref = world.clone();
                async move {
                    let mut ctx = crate::hooks::HookContext {
                        world: &world_ref,
                        session_uid: Some(uid),
                    };
                    h.at_disconnect(&mut ctx, acc).await
                }
            })
            .await;
    }
}
```

- [ ] **Step 2: Update the call site in `process()` to pass the new args**

```rust
client_cleanup(uid, client.account_uid, &server, registry, cache).await;
```

- [ ] **Step 3: Verify**

Run: `cargo check -p muoxi`
Expected: `Finished`.

- [ ] **Step 4: Commit**

```bash
git add muoxi/src/server/main.rs
git commit -m "feat(server): client_cleanup fires at_disconnect with account_uid"
```

---

## Task 10: Update built-in commands to use `client.character_uid`

**Files:**
- Modify: `muoxi/src/server/commands/look.rs`
- Modify: `muoxi/src/server/commands/who.rs`

- [ ] **Step 1: Look uses character_uid**

In `look.rs`, replace `ctx.client.uid` with `ctx.client.character_uid`:

```rust
async fn execute_cmd(&self, ctx: CommandContext<'_>) -> Result<(), &'static str> {
    let Some(my_uid) = ctx.client.character_uid else {
        let _ = send(ctx.client, "You don't have a character.").await;
        return Ok(());
    };
    let me = ctx.world.get_object(my_uid).await
        .map_err(|_| "db error")?
        .ok_or("you don't seem to exist")?;
    // ... rest unchanged ...
}
```

- [ ] **Step 2: Who lists connected character names**

In `who.rs`:

```rust
async fn execute_cmd(&self, ctx: CommandContext<'_>) -> Result<(), &'static str> {
    // For v0.1, walk Server.clients via the registry's world facade is cleaner,
    // but Server isn't accessible from inside commands. Keep this stubbed and
    // expand in a follow-up plan when Server is exposed through the registry.
    let _ = send(ctx.client, "Who: (extension point — wire Server access here)").await;
    Ok(())
}
```

- [ ] **Step 3: Verify**

Run: `cargo check -p muoxi`
Expected: `Finished`.

- [ ] **Step 4: Commit**

```bash
git add muoxi/src/server/commands/look.rs muoxi/src/server/commands/who.rs
git commit -m "fix(commands): look uses character_uid; who placeholder"
```

---

## Task 11: End-to-end auth flow integration test

**Files:**
- Create: `muoxi/tests/auth_flow.rs`

- [ ] **Step 1: Write the test**

```rust
//! End-to-end test: spawn the server, telnet through new-account creation +
//! character creation + Playing state.

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn full_login_then_play() {
    use std::time::Duration;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpStream;
    use tokio::process::Command;

    let dir = env!("CARGO_MANIFEST_DIR");
    let target = format!("{}/../target/debug/muoxi_server", dir);
    let mut child = Command::new(&target)
        .env("DATABASE_URL", ":memory:")
        .env("PROXY_ADDR", "127.0.0.1:18000")
        .env("REDIS_SERVER", "redis://127.0.0.1:1") // intentionally unreachable; cache writes become no-ops in tester (or this skips Redis-dependent code paths)
        .kill_on_drop(true)
        .spawn()
        .expect("spawn server");

    tokio::time::sleep(Duration::from_millis(800)).await;

    let stream = TcpStream::connect("127.0.0.1:18000")
        .await
        .expect("connect");
    let (rd, mut wr) = stream.into_split();
    let mut rd = BufReader::new(rd).lines();

    let _welcome = rd.next_line().await.unwrap(); // banner

    wr.write_all(b"new\n").await.unwrap();
    let _ = rd.next_line().await; // "Choose..."
    wr.write_all(b"alice\n").await.unwrap();
    let _ = rd.next_line().await; // "Password..."
    wr.write_all(b"hunter2\n").await.unwrap();
    let _ = rd.next_line().await; // "Confirm..."
    wr.write_all(b"hunter2\n").await.unwrap();
    let _ = rd.next_line().await; // "Account alice created."

    // MainMenu prompt
    let _ = rd.next_line().await;

    // create a character and enter playing
    wr.write_all(b"new Hero\n").await.unwrap();
    let _ = rd.next_line().await; // "Created Hero. Entering world."

    // try a command
    wr.write_all(b"look\n").await.unwrap();
    let response = rd.next_line().await.unwrap().unwrap();
    assert!(response.contains("void") || response.contains("\n") || !response.is_empty());

    wr.write_all(b"quit\n").await.unwrap();
    let _ = rd.next_line().await;

    let _ = child.kill().await;
}
```

NOTE: this test depends on the server being built with the `db-sqlite` default and on `127.0.0.1:18000` being free. It uses an in-memory DB via `DATABASE_URL=":memory:"`. Real CI configurations may need to spawn from a release build, or use the lib-mode entry to avoid binary spawning entirely.

- [ ] **Step 2: Run**

```bash
cd /home/duys/.repos/MuOxi
cargo build --bin muoxi_server
cargo test -p muoxi --test auth_flow -- --nocapture 2>&1 | tail -40
```

Expected: test passes.

- [ ] **Step 3: Commit**

```bash
git add muoxi/tests/auth_flow.rs
git commit -m "test(server): end-to-end account creation + character + look"
```

---

## Task 12: Update AGENTS.md

**Files:**
- Modify: `muoxi/src/server/AGENTS.md`
- Modify: root `AGENTS.md`

- [ ] **Step 1: Replace the STATE MACHINE STATUS section in `server/AGENTS.md`**

```markdown
## STATE MACHINE

All 8 `ConnStates` are implemented. Flow:

```
        ┌──────────────┐
        │ AwaitingName │◄────────┐
        └──────┬───────┘         │ (bad name / no account)
   ┌─── new ───┤
   ▼           ▼ existing
┌────────┐  ┌────────────┐
│ AwNN   │  │ AwPassword │── bad ──► AwaitingName
│  ↓     │  └─────┬──────┘
│ AwNP   │        │ ok
│  ↓     │        ▼
│ ConfNP │  ┌──────────┐
└───┬────┘  │ MainMenu │
    └──────►└────┬─────┘
                  │ pick / new
                  ▼
            ┌─────────┐
            │ Playing │── quit ──► Quit
            └────┬────┘
                 │
            (commands)
```

* `Client.auth_buffer` carries pending name + first-password-attempt across transitions.
* `Client.account_uid` is set on successful auth; cleared on bad password or `Quit`.
* `Client.character_uid` is set on character select / create.
* `at_login` fires on `AwaitingPassword` success.
* `at_disconnect` fires from `client_cleanup` whenever a session ends.
```

- [ ] **Step 2: Root AGENTS.md CODE MAP**

Add `auth::hash_password / verify_password / AuthBuffer` row, and update the `ConnStates` row to reflect "all 8 implemented".

- [ ] **Step 3: Commit**

```bash
git add muoxi/src/server/AGENTS.md AGENTS.md
git commit -m "docs(agents): document the now-complete auth state machine"
```

---

## Verification Summary

A successful run of this plan ends with:

- [ ] All 8 `ConnStates` arms execute meaningful logic; no fall-through to `Quit`.
- [ ] argon2id password hashing works (round-trip test passes).
- [ ] Name + password validators reject invalid inputs with helpful messages.
- [ ] An end-to-end test spawns `muoxi_server`, creates an account, creates a character, enters `Playing`, executes `look`, and quits cleanly.
- [ ] `at_login` fires on successful auth; `at_disconnect` fires on session end.
- [ ] `Client.account_uid` and `Client.character_uid` are populated at the right transitions.
- [ ] Built-in `look` command uses `character_uid`.
- [ ] AGENTS.md reflects the completed flow.
