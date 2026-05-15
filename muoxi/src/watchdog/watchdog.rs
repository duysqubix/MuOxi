#![deny(missing_docs)]

//! Watchdog process. Monitors the canonical JSON files under `json/` and
//! syncs every change into Postgres via the shared `db` crate. Runs as its
//! own process, separate from the staging proxy and engine.
//!
//! ```ignore
//! let mut watcher = Hotwatch::new_with_custom_delay(Duration::from_millis(100))?;
//!
//! watcher.watch("json/accounts.json", |event| {
//!     if matches!(event.kind, EventKind::Modify(_)) {
//!         trigger_upload(JsonFile::Accounts).unwrap();
//!     }
//! })?;
//! ```

use db::DatabaseHandler;
use db::structures::DatabaseHandlerExt;
use db::structures::{account::Account, character::Character};
use db::utils::{UID, json_to_object, read_json_file};
use hotwatch::{Event, EventKind, Hotwatch};
use std::collections::HashMap;
use std::env;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

fn accounts_path() -> &'static String {
    static ACCOUNTS: OnceLock<String> = OnceLock::new();
    ACCOUNTS.get_or_init(|| {
        let mut config_dir = env::current_dir().unwrap().to_str().unwrap().to_string();
        config_dir.push_str("/json/accounts.json");
        config_dir
    })
}

fn characters_path() -> &'static String {
    static CHARACTERS: OnceLock<String> = OnceLock::new();
    CHARACTERS.get_or_init(|| {
        let mut config_dir = env::current_dir().unwrap().to_str().unwrap().to_string();
        config_dir.push_str("/json/characters.json");
        config_dir
    })
}

/// Different `.json` storage files that need to be monitored
#[derive(Debug, Clone)]
pub enum JsonFile {
    /// holds account information
    /// ex: number of characters of account, client settings.
    Accounts,
    /// holds all character information
    Characters,
}

/// Synchronize the changed JSON file into Postgres.
pub fn trigger_upload(file: JsonFile) -> Result<(), Box<dyn std::error::Error>> {
    let mut db = DatabaseHandler::connect();

    match file {
        JsonFile::Accounts => {
            let accounts = read_json_file(accounts_path()).expect("Couldn't read accounts.json");
            let accounts: HashMap<UID, Account> = json_to_object(accounts).unwrap();
            for (_uid, account) in accounts {
                db.accounts.upsert(&mut db.handle, &account)?;
            }

            let records = db.accounts.get_batch(&mut db.handle, vec![])?;
            for acct in records.0.iter() {
                println!(
                    "Found account with UID: {}....{}:{:?}",
                    acct.uid,
                    acct.name,
                    acct.characters.as_ref()
                );
            }
        }
        JsonFile::Characters => {
            let chars = read_json_file(characters_path()).expect("Couldn't read characters.json");
            let _characters: HashMap<UID, Character> = json_to_object(chars).unwrap();

            let records = db.accounts.get_batch(&mut db.handle, vec![])?;
            for acct in records.0.iter() {
                println!(
                    "Found account with UID: {}....{}:{:?}",
                    acct.uid,
                    acct.name,
                    acct.characters.as_ref()
                );
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut watcher = Hotwatch::new_with_custom_delay(Duration::from_millis(100))?;

    watcher.watch(characters_path(), |event: Event| {
        if matches!(event.kind, EventKind::Modify(_)) {
            trigger_upload(JsonFile::Characters).unwrap();
        }
    })?;
    watcher.watch(accounts_path(), |event: Event| {
        if matches!(event.kind, EventKind::Modify(_)) {
            trigger_upload(JsonFile::Accounts).unwrap();
        }
    })?;

    println!("Watchdog runing...");
    let t = thread::spawn(|| {
        loop {
            thread::park();
        }
    });
    t.join().unwrap();

    Ok(())
}
