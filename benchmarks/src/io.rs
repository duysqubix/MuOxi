#![allow(unused_imports)]
use crate::report::{Report, ReportBuilder};
use db::utils::{json_to_object, read_json_file, write_json_file, JsonDecoderResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{remove_file, File};
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

pub fn benchmark_io_json_100_000() -> Result<(), Box<dyn std::error::Error>> {
    let mut s = String::new();
    let now = SystemTime::now();
    let start = now.elapsed().unwrap().as_millis();
    let records = read_json_file("benchmarks/db_100_000.json")?;
    let p1 = now.elapsed().unwrap().as_millis();
    let t = format!(
        "reading took {} ms\n",
        now.elapsed().unwrap().as_millis() - start
    );

    s.push_str(&t);

    let mut records: HashMap<usize, Person> = json_to_object(records)?;
    let p2 = now.elapsed().unwrap().as_millis();
    let t = format!("deserializing took {} ms\n", (p2 - p1));
    s.push_str(&t);
    // change a single thing and write back to file.
    let item = records.get_mut(&1).unwrap();
    item.name = "Duan Uys".to_string();

    write_json_file("benchmarks/db_100_000_altered.json", &records)?;
    let p3 = now.elapsed().unwrap().as_millis();

    let t = format!(
        "writing took {} ms\nTotal time: {}ms",
        p3 - p2,
        now.elapsed().unwrap().as_millis()
    );
    s.push_str(&t);

    let mut report = ReportBuilder::new();
    report
        .with_title("I/O Benchmark with 100_000 elements")
        .with_body(&s)
        .with_footnotes("")
        .build_report()
        .write_report("benchmarks/results/io_benchmarks.txt")?;

    // clean up and remove altered file
    remove_file("benchmarks/db_100_000_altered.json")?;
    Ok(())
}
