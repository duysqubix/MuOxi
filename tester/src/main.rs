use lazy_static::lazy_static;
use muoxi_db as db;
use muoxi_states as states;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Result as serdeResult;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufReader, SeekFrom};
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Person {
    uid: usize,
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
    let mut file = File::create(path).unwrap();
    serde_json::to_writer_pretty(&file, object)?;
    Ok(())
}

fn main() {
    let thread1 = thread::spawn(move || {
        let mut cur_sha: [u8; 32] = [0; 32];
        let mut times_changed: usize = 0;
        let mut digest: [u8; 32];
        let sleep = Duration::from_micros(500);

        let mut cur_buf: Vec<u8> = Vec::new();
        {
            let mut file = File::open("config/people.json").unwrap();
            file.read_to_end(&mut cur_buf).unwrap();
        }
        loop {
            // let mut sha256 = sha2::Sha256::new();
            let mut buf: Vec<u8> = Vec::new();

            let mut file = File::open("config/people.json").unwrap();

            file.read_to_end(&mut buf).unwrap();

            if cur_buf != buf {
                times_changed += 1;
                println!("Filed Changed: {}", times_changed,);
                cur_buf = buf;
            }

            // std::io::copy(&mut file, &mut sha256).unwrap();
            // let shdigest = sha256.result();

            // digest = shdigest.into();
            // if digest != cur_sha {
            //     times_changed += 1;
            //     println!("File changed!, {} / {:X}", times_changed, shdigest);

            //     //kick off load to database
            //     cur_sha = digest;
            // }

            thread::sleep(sleep);
        }
    });

    thread::sleep(Duration::from_millis(100));

    let mut people: HashMap<usize, Person> = HashMap::new();

    let person1 = Person {
        uid: 0,
        name: "Duan".to_string(),
        age: 27,
        email: "duanuys.financials@gmail.com".to_string(),
    };

    let person2 = Person {
        uid: 1,
        name: "Tavis".to_string(),
        age: 13,
        email: "duan@gmail.com".to_string(),
    };

    people.insert(person1.uid, person1.clone());
    people.insert(person2.uid, person2.clone());

    // write to file

    let pjson = serde_json::to_string(&people).unwrap();
    //println!("Write 1");
    write_to_file("config/people.json".to_string(), &people).expect("Coulnd't write to file");
    // read and write to file
    //println!("Read 1");

    let mut json = read_file("config/people.json".to_string()).expect("Couldn't read fromfile");

    //println!("{}", json.get(person1.uid.to_string().as_str()).unwrap());
    let new_age: serde_json::Value = serde_json::from_str("100").unwrap();

    let person = json.get_mut(person1.uid.to_string().as_str()).unwrap();

    person["age"] = new_age;
    //println!("{}", json.get(person1.uid.to_string().as_str()).unwrap());
    //println!("Write 2");

    //println!("{}", json);
    write_to_file("config/people.json".to_string(), &json).expect("Couldn't write to file");

    // file.seek(SeekFrom::Start(0)).unwrap();
    // file.set_len(0).unwrap();
    // println!("Writing");
    // serde_json::to_writer_pretty(&file, &json).unwrap();
    // }
    // let mut client = db::DatabaseHandler::new("MuOxi".to_string()).unwrap();
    // client.set_db("test").unwrap();

    // let state = states::ConnStates::AwaitingName;
    // let ncharacters = 0;

    // let new_client = db::clients::ClientDB {
    //     uid: 1222,
    //     name: "Duan".to_string(),
    //     ip: "192.168.1.5".to_string(),
    //     port: 5756,
    //     state: state,
    //     ncharacters: ncharacters,
    // };

    // let mudcrab = db::clients::Character {
    //     uid: 2,
    //     name: "Mud Crab".to_string(),
    //     class: "Warrior".to_string(),
    //     gold: 110,
    // };

    // let accounts = client.get_db().unwrap().collection("test");
    // let mobs = client.get_db().unwrap().collection("mobs");

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
    thread1.join().unwrap();
}
