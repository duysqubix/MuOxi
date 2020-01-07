//! WatchDog that monitors the custom defined `.json` files located within `config/` directory
//! Runs as a completely seperate process apart from all servers. Watchdogs main job is to
//! notice contents changes of file and sync them with MongoDB
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

use db::{utils::json_to_object, DatabaseHandler};
use hotwatch::{Event, Hotwatch};
use serde_json;
use serde_json::Result as serdeResult;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time::Duration;
// use serde::{Serialize, Deserialize};

pub static ACCOUNTS: &'static str = "config/accounts.json";

/// Different `.json` storage files that need to be monitored
#[derive(Debug, Clone)]
pub enum JsonFile {
    /// holds account information
    /// ex: number of characters of account, client settings.
    Accounts,

    /// holds all character information
    Players,
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
    let caller = format!("WatchDog: {:?}", file);
    let mut mongo = DatabaseHandler::new(caller)?;

    // set db depending on file
    match file {
        JsonFile::Accounts => {
            mongo.set_db("accounts")?;
            // mongo.drop_collection("accounts", None)?;

            let accounts =
                read_file("config/accounts.json").expect("Couldn't read from accounts.json");

            let accounts: HashMap<u64, db::templates::ClientDB> = json_to_object(accounts).unwrap();
            let account_collection = mongo.get_collection("accounts")?;

            for (_uid, account) in accounts.iter() {
                mongo.insert_one(account, &account_collection, None, true)?;
            }
        }
        JsonFile::Players => {
            mongo.set_db("players").unwrap()
            //
        }
    }

    // load json into vec<T>; where T: object

    // convert to bson
    // use db tools to iterate through db and insert in-memory structures
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

    watcher.watch(ACCOUNTS, move |event| {
        if let Event::Write(_path) = event {
            trigger_upload(JsonFile::Accounts).unwrap();
        }
    })?;

    println!("Watchdog runing...");
    let t = thread::spawn(|| loop {});
    t.join().unwrap();

    // trigger_upload(JsonFile::Accounts).unwrap();
    Ok(())
}
