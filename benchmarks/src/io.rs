#![allow(unused_imports)]
use db::utils::{json_to_object, JsonDecoderResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Person {
    id: usize,
    name: String,
    email: String,
    hp: usize,
    mana: usize,
    vit: usize,
}

pub fn read_file<'a>(path: &'a str) -> serde_json::Result<serde_json::Value> {
    let mut s = Vec::new();
    File::open(path).unwrap().read_to_end(&mut s).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&s)?;
    Ok(json)
}

pub fn benchmark_io_json_100_000() -> Result<(), Box<dyn std::error::Error>> {
    let now = SystemTime::now();
    let start = now.elapsed().unwrap().as_millis();
    let records = read_file("db_100_000.json")?;
    let p1 = now.elapsed().unwrap().as_millis();
    println!(
        "reading took {} us",
        now.elapsed().unwrap().as_millis() - start
    );

    let mut records: HashMap<usize, Person> = json_to_object(records)?;
    let p2 = now.elapsed().unwrap().as_millis();
    println!("deserializing took {} us", (p2 - p1));

    // change a single thing and write back to file.
    let item = records.get_mut(&1).unwrap();
    item.name = "Duan Uys".to_string();

    let file = File::create("db_100_000_altered.json")?;
    let writer = BufWriter::new(&file);
    serde_json::to_writer_pretty(writer, &records)?;
    let p3 = now.elapsed().unwrap().as_millis();

    println!("writing took {} us", p3 - p2);
    println!("Total time: {}us", now.elapsed().unwrap().as_millis());
    Ok(())
}
