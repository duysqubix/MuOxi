use bson::oid::ObjectId;
use db;
use hotwatch::{Event, Hotwatch};
use serde::{Deserialize, Serialize};
use serde_json::Result as serdeResult;
use states;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Person {
    uid: String,
    name: String,
    age: u8,
    email: String,
}

fn read_file(path: String) -> serde_json::Result<serde_json::Value> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(&file);
    let json: serde_json::Value = serde_json::from_reader(reader).unwrap();
    Ok(json)
}

fn write_to_file<'de, T: Serialize + Deserialize<'de>>(
    path: String,
    object: &T,
) -> serdeResult<()> {
    thread::sleep(Duration::from_millis(100));
    let file = File::create(path).unwrap();
    serde_json::to_writer_pretty(&file, object)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    //
    // ********** Serialzing/Deserialzing to JSON ***********************//
    //
    let mut people: HashMap<String, Person> = HashMap::new();
    let mut clients: HashMap<db::utils::UID, db::clients::ClientDB> = HashMap::new();

    let person1 = Person {
        uid: ObjectId::new().unwrap().to_hex(),
        name: "Duan".to_string(),
        age: 27,
        email: "duanuys.financials@gmail.com".to_string(),
    };

    let person2 = Person {
        uid: ObjectId::new().unwrap().to_hex(),
        name: "Tavis".to_string(),
        age: 13,
        email: "duan@gmail.com".to_string(),
    };

    people.insert(person1.uid.clone(), person1.clone());
    people.insert(person2.uid.clone(), person2.clone());

    // write to file

    let state = states::ConnStates::AwaitingName;
    let ncharacters = 0;
    let new_client = db::clients::ClientDB {
        uid: db::utils::gen_uid(),
        name: "Duan".to_string(),
        ip: "192.168.1.5".to_string(),
        port: 5756,
        state: state,
        ncharacters: ncharacters,
    };

    clients.insert(new_client.uid.clone(), new_client.clone());

    write_to_file("config/accounts.json".to_string(), &clients).expect("Couldn't write to file");
    write_to_file("config/people.json".to_string(), &people).expect("Couldn't write to file");

    let mut json = read_file("config/people.json".to_string()).expect("Couldn't read fromfile");

    let new_age: serde_json::Value = serde_json::from_str("100").unwrap();

    let person = json.get_mut(person1.uid.to_string().as_str()).unwrap();

    person["age"] = new_age;

    write_to_file("config/people.json".to_string(), &json).expect("Couldn't write to file");

    //
    // ********** BSON and MongoDB ***********************//
    //
    // let mut client = db::DatabaseHandler::new("MuOxi".to_string()).unwrap();
    // client.set_db("test").unwrap();

    // let state = states::ConnStates::AwaitingName;
    // let ncharacters = 0;
    // let new_client = db::clients::ClientDB {
    //     uid: db::utils::gen_uid(),
    //     name: "Duan".to_string(),
    //     ip: "192.168.1.5".to_string(),
    //     port: 5756,
    //     state: state,
    //     ncharacters: ncharacters,
    // };

    // let mudcrab = db::clients::Character {
    //     uid: db::utils::gen_uid(),
    //     name: "Mud Crab".to_string(),
    //     class: "Warrior".to_string(),
    //     gold: 110,
    // };

    // let accounts = client
    //     .get_collection("test")
    //     .expect("Couldn't find accounts collection");
    // let mobs = client
    //     .get_collection("mobs")
    //     .expect("Couldn't find mobs collection");

    // client
    //     .insert_one(&mudcrab, &mobs, None)
    //     .unwrap_or_else(|e| {
    //         println!("{:?}", e);
    //     });

    // let crab1 = client
    //     .get_doc(&mudcrab, &mobs, db::utils::FilterOn::UID, None)
    //     .unwrap();

    // if let Some(crab1) = crab1 {
    //     let mut result: db::clients::Character = db::utils::to_object(crab1).unwrap();
    //     println!("{:?}", result);

    //     result.name = "Large Mud Crab".to_string();
    //     client
    //         .update(&result, &mobs, db::utils::FilterOn::UID, None, None)
    //         .unwrap();
    // }

    // client
    //     .insert_one(&new_client, &accounts, None)
    //     .unwrap_or_else(|e| {
    //         println!("{:?}", e);
    //     });

    // client
    //     .update(
    //         &new_client,
    //         &accounts,
    //         db::utils::FilterOn::NAME,
    //         None,
    //         None,
    //     )
    //     .unwrap();

    // println!("{:?}", new_client);
    // hot_watch.join().unwrap();
    println!("Done with main");
    // wd.join().unwrap();
    Ok(())
}
