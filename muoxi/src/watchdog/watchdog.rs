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

use db::clients::Client;
use db::utils::{json_to_object, UID};
use db::DatabaseHandler;
use hotwatch::{Event, Hotwatch};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time::Duration;

/// static path to place of clients json file
pub static CLIENTS: &'static str = "json/clients.json";

/// Different `.json` storage files that need to be monitored
#[derive(Debug, Clone)]
pub enum JsonFile {
    /// holds account information
    /// ex: number of characters of account, client settings.
    Accounts,

    /// holds all character information
    Players,

    /// holds raw socket representation of connected clients
    Clients,
}

/// simple wrapper to read from json file and return serde_json::Value
pub fn read_file<'a>(path: &'a str) -> serde_json::Result<serde_json::Value> {
    let file = File::open(String::from(path)).unwrap();
    let reader = BufReader::new(&file);
    let json: serde_json::Value = serde_json::from_reader(reader).unwrap();
    Ok(json)
}

/// main function that triggers upload protocols for each change in file based on `JsonFile`
pub fn trigger_upload(file: JsonFile) -> Result<(), Box<dyn std::error::Error>> {
    let db = DatabaseHandler::connect();

    // set db depending on file
    match file {
        JsonFile::Clients => {
            let clients =
                read_file("config/clients.json").expect("Couldn't read from accounts.json");

            let clients: HashMap<UID, Client> = json_to_object(clients).unwrap();
            // let client_vec: ClientVector = ClientHashMap(clients.clone()).into();
            for (_uid, client) in clients {
                db.clients.upsert(&db.handle, &client)?;
            }

            let records = db.clients.get_uids(&db.handle, vec![])?;
            for client in records.0.iter() {
                println!(
                    "Found client with UID: {}...{}: {}",
                    client.uid, client.ip, client.port
                );
            }
            println!("");
        }
        JsonFile::Players => {
            //
        }

        JsonFile::Accounts => {}
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // write all initial

    let mut watcher = Hotwatch::new_with_custom_delay(Duration::from_millis(100))?;

    watcher.watch("config/people.json", move |event| {
        if let Event::Write(_path) = event {
            trigger_upload(JsonFile::Players).unwrap();
        }
    })?;

    watcher.watch(CLIENTS, move |event| {
        if let Event::Write(_path) = event {
            trigger_upload(JsonFile::Clients).unwrap();
        }
    })?;

    println!("Watchdog runing...");
    let t = thread::spawn(|| loop {});
    t.join().unwrap();

    // trigger_upload(JsonFile::Clients).unwrap();
    Ok(())
}
