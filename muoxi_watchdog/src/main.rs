use hotwatch::{Event, Hotwatch};
use muoxi_db::{utils::json_to_object, DatabaseHandler};
use serde_json;
use serde_json::Result as serdeResult;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time::Duration;
// use serde::{Serialize, Deserialize};

static ACCOUNTS: &'static str = "config/accounts.json";

#[derive(Debug, Clone)]
enum ConfigFile {
    Accounts,
    Players,
}

fn read_file<'a>(path: &'a str) -> serde_json::Result<serde_json::Value> {
    let file = File::open(String::from(path)).unwrap();
    let reader = BufReader::new(&file);
    let json: serde_json::Value = serde_json::from_reader(reader).unwrap();
    Ok(json)
}

fn trigger_upload(file: ConfigFile) -> Result<(), Box<dyn std::error::Error>> {
    let caller = format!("WatchDog: {:?}", file);
    let mut mongo = DatabaseHandler::new(caller)?;

    // set db depending on file
    match file {
        ConfigFile::Accounts => {
            mongo.set_db("accounts").unwrap();
            let accounts =
                read_file("config/accounts.json").expect("Couldn't read from accounts.json");

            let accounts: HashMap<u64, muoxi_db::clients::ClientDB> =
                json_to_object(accounts).unwrap();
            println!("{:?}", accounts);
        }
        ConfigFile::Players => {
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
    // let mut watcher = Hotwatch::new_with_custom_delay(Duration::from_millis(100))?;

    // watcher.watch("config/people.json", move |event| {
    //     if let Event::Write(_path) = event {
    //         trigger_upload(ConfigFile::Players).unwrap();
    //     }
    // })?;

    // watcher.watch(ACCOUNTS, move |event| {
    //     if let Event::Write(_path) = event {
    //         trigger_upload(ConfigFile::Accounts).unwrap();
    //     }
    // })?;

    // println!("Watchdog runing...");
    // let t = thread::spawn(|| {});
    // t.join().unwrap();

    trigger_upload(ConfigFile::Accounts).unwrap();
    Ok(())
}
