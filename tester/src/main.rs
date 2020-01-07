use db;
use redis::{Commands, PipelineCommands};
use serde::{Deserialize, Serialize};
use serde_json::Result as serdeResult;
use states;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Person {
    uid: String,
    name: String,
    age: u8,
    email: String,
}

fn fetch_an_integer() -> redis::RedisResult<isize> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;

    let _: () = con.set("my_key", 42)?;
    con.get("my_key")
}

fn fetch_an_hmap() -> redis::RedisResult<HashMap<String, String>> {
    let client = redis::Client::open("redis://127.0.0.1")?;
    let mut con = client.get_connection()?;
    let map: HashMap<String, String> = con.hgetall("person")?;
    Ok(map)
}

fn main() -> Result<(), Box<dyn Error>> {
    //************************** PostgreSQL/Desiel*****************

    let mut db = db::DatabaseHandler::connect();
    let new_client = db::clients::Client {
        uid: 124,
        ip: "192.168.0.1".to_string(),
        port: 8002,
    };

    let result = db.clients.upsert(&db.handle, new_client.clone()).unwrap();
    println!("{:?}", result);
    let filter_criteria = db::clients::FilterCriteria("uid", 124, 5);
    let records = db.clients.get(&db.handle, filter_criteria);
    for record in records {
        println!("{:?}", record);
    }
    // ******************* Redis Read/Write *********************
    // let client = redis::Client::open("redis://127.0.0.1")?;
    // let mut con = client.get_connection()?;

    // redis::cmd("SET").arg("the_key").arg(23).query(&mut con)?;
    // let key = "the_key";
    // let (new_val,): (isize,) = redis::transaction(&mut con, &[key], |con, pipe| {
    //     let old_val: isize = con.get(key)?;
    //     pipe.set(key, old_val + 1).ignore().get(key).query(con)
    // })?;

    // println!("{}", new_val);
    //
    // ********** Serialzing/Deserialzing to JSON ***********************//
    //
    // let mut people: HashMap<String, Person> = HashMap::new();
    // let mut clients: HashMap<db::utils::UID, db::clients::ClientDB> = HashMap::new();

    // let person1 = Person {
    //     uid: ObjectId::new().unwrap().to_hex(),
    //     name: "Duan".to_string(),
    //     age: 27,
    //     email: "duanuys.financials@gmail.com".to_string(),
    // };

    // let person2 = Person {
    //     uid: ObjectId::new().unwrap().to_hex(),
    //     name: "Tavis".to_string(),
    //     age: 13,
    //     email: "duan@gmail.com".to_string(),
    // };

    // people.insert(person1.uid.clone(), person1.clone());
    // people.insert(person2.uid.clone(), person2.clone());

    // // write to file

    // let state = states::ConnStates::AwaitingName;
    // let ncharacters = 0;
    // let new_client = db::templates::ClientDB {
    //     uid: db::utils::gen_uid(),
    //     name: "Duan".to_string(),
    //     ip: "192.168.1.5".to_string(),
    //     port: 5756,
    //     state: state,
    //     ncharacters: ncharacters,
    // };

    // clients.insert(new_client.uid.clone(), new_client.clone());

    // write_to_file("config/accounts.json".to_string(), &clients).expect("Couldn't write to file");
    // write_to_file("config/people.json".to_string(), &people).expect("Couldn't write to file");

    // let mut json = read_file("config/people.json".to_string()).expect("Couldn't read fromfile");

    // let new_age: serde_json::Value = serde_json::from_str("100").unwrap();

    // let person = json.get_mut(person1.uid.to_string().as_str()).unwrap();

    // person["age"] = new_age;

    // write_to_file("config/people.json".to_string(), &json).expect("Couldn't write to file");

    println!("Done with main");
    Ok(())
}
