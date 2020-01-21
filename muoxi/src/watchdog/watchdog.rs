#![deny(missing_docs)]

//! WatchDog that monitors the custom defined `.json` files located within `config/` directory
//! Runs as a completely seperate process apart from all servers. Watchdogs main job is to
//! notice contents changes of file and sync them with Database
//!
//! ## Basic usage to watch a file
//! ```rust
//! let mut watchdog = Hotwatch::new_with_custom_delay(Duration::from_millis(100)?);
//!
//! watcher.watch("config/accounts.json", move |event|{
//!     if let Event::Write(_path) = event{
//!         // sync accounts.json with accounts database in MongoDB
//!         trigger_upload(JsonFile::Accounts).unwrap();
//!     }
//! })?;
//! ```

use db::structures::DatabaseHandlerExt;
use db::structures::{account::Account, character::Character};
use db::utils::{json_to_object, read_json_file, UID};
use db::DatabaseHandler;
use hotwatch::{Event, Hotwatch};
use lazy_static::lazy_static;
use serde_json;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time::Duration;

lazy_static! {
    static ref ACCOUNTS: String = {
        let mut config_dir = env::current_dir().unwrap().to_str().unwrap().to_string();
        config_dir.push_str("/json/accounts.json");
        config_dir
    };
    static ref CHARACTERS: String = {
        let mut config_dir = env::current_dir().unwrap().to_str().unwrap().to_string();
        config_dir.push_str("/json/characters.json");
        config_dir
    };
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

/// main function that triggers upload protocols for each change in file based on `JsonFile`
pub fn trigger_upload(file: JsonFile) -> Result<(), Box<dyn std::error::Error>> {
    let db = DatabaseHandler::connect();

    // set db depending on file
    match file {
        JsonFile::Accounts => {
            let accounts = read_json_file(&*ACCOUNTS).expect("Couldn't read from accounts.json");

            let accounts: HashMap<UID, Account> = json_to_object(accounts).unwrap();
            for (_uid, account) in accounts {
                db.accounts.upsert(&db.handle, &account)?;
            }

            let records = db.accounts.get_batch(&db.handle, vec![])?;
            for acct in records.0.iter() {
                println!(
                    "Found account with UID: {}....{}:{:?}",
                    acct.uid,
                    acct.name,
                    acct.characters.as_ref()
                )
            }
            // let clients =
            //     read_file("config/clients.json").expect("Couldn't read from accounts.json");

            // let clients: HashMap<UID, Client> = json_to_object(clients).unwrap();
            // // let client_vec: ClientVector = ClientHashMap(clients.clone()).into();
            // for (_uid, client) in clients {
            //     db.clients.upsert(&db.handle, &client)?;
            // }

            // let records = db.clients.get_uids(&db.handle, vec![])?;
            // for client in records.0.iter() {
            //     println!(
            //         "Found client with UID: {}...{}: {}",
            //         client.uid, client.ip, client.port
            //     );
            // }
            // println!("");
        }
        JsonFile::Characters => {
            //
            let chars = read_json_file(&*CHARACTERS).expect("Couldn't read from characters.json");

            let characters: HashMap<UID, Character> = json_to_object(chars).unwrap();
            for (_uid, character) in characters {
                // db.characters.upsert(&db.handle, &character)?;
            }

            let records = db.accounts.get_batch(&db.handle, vec![])?;
            for acct in records.0.iter() {
                println!(
                    "Found account with UID: {}....{}:{:?}",
                    acct.uid,
                    acct.name,
                    acct.characters.as_ref()
                )
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut watcher = Hotwatch::new_with_custom_delay(Duration::from_millis(100))?;

    watcher.watch(&*CHARACTERS, move |event| {
        if let Event::Write(_path) = event {
            trigger_upload(JsonFile::Characters).unwrap();
        }
    })?;
    watcher.watch(&*ACCOUNTS, move |event| {
        if let Event::Write(_path) = event {
            trigger_upload(JsonFile::Accounts).unwrap();
        }
    })?;

    println!("Watchdog runing...");
    let t = thread::spawn(|| loop {});
    t.join().unwrap();

    // trigger_upload(JsonFile::Accounts).unwrap();
    // trigger_upload(JsonFile::Characters).unwrap();

    Ok(())
}
