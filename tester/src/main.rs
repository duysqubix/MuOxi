use bson::oid::ObjectId;
use muoxi_db as db;
use muoxi_states as states;

fn main() {
    let mut client = db::DatabaseHandler::new("MuOxi".to_string()).unwrap();
    client.set_db("test").unwrap();

    let state = states::ConnStates::AwaitingName;
    let ncharacters = 0;

    let new_client = db::clients::ClientDB {
        uid: 1222,
        name: "Duan".to_string(),
        ip: "192.168.1.5".to_string(),
        port: 5756,
        state: state,
        ncharacters: ncharacters,
    };

    let mudcrab = db::clients::Character {
        uid: 2,
        name: "Mud Crab".to_string(),
        class: "Warrior".to_string(),
        gold: 110,
    };

    let accounts = client.get_db().unwrap().collection("test");
    let mobs = client.get_db().unwrap().collection("mobs");

    client
        .insert_one(&mudcrab, &mobs, None)
        .unwrap_or_else(|e| {
            println!("{:?}", e);
        });

    let crab1 = client
        .get_doc(&mudcrab, &mobs, db::utils::FilterOn::UID, None)
        .unwrap();

    if let Some(crab1) = crab1 {
        let mut result: db::clients::Character = db::utils::to_object(crab1).unwrap();
        println!("{:?}", result);

        result.name = "Large Mud Crab".to_string();
        client
            .update(result, mobs, db::utils::FilterOn::UID, None, None)
            .unwrap();
    }

    // client
    //     .update(new_client, accounts, db::utils::FilterOn::NAME, None, None)
    //     .unwrap();
    // client
    //     .insert_one(new_client, accounts, None)
    //     .unwrap_or_else(|e| {
    //         println!("{:?}", e);
    //     });

    // println!("{:?}", new_client);
}
