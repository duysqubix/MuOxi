#![allow(unused_imports)]

//!
//! Holds collections of regularly used functions that relate to database usage
//!

use rand::Rng;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use std::fs::File;

use std::io::{BufWriter, Read, Write};
use std::time::SystemTime;

/// unique id for each instance
pub type UID = i64;

/// custom result for decoding json values
pub type JsonDecoderResult<T> = Result<T, serde_json::error::Error>;

/// Reads JSON file and convert to JSON::Value
///
pub fn read_json_file<'a>(path: &'a str) -> serde_json::Result<serde_json::Value> {
    let mut s = Vec::new();
    File::open(path).unwrap().read_to_end(&mut s).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&s)?;
    Ok(json)
}

/// Serializes and writes structure to JSON
pub fn write_json_file<'a, T: Serialize>(path: &'a str, object: &T) -> serde_json::Result<()> {
    let errmsg = format!("Couldn't create file: {}", path);
    let file = File::create(path).expect(errmsg.as_str());
    let writer = BufWriter::new(&file);
    serde_json::to_writer_pretty(writer, object)?;
    Ok(())
}

/// Attempt converstion from JSON Value to Object T
pub fn json_to_object<T: Serialize + DeserializeOwned>(
    val: serde_json::Value,
) -> JsonDecoderResult<T> {
    serde_json::from_value(val)
}

/// Creates a unique 8 byte address first 4 bytes is timestamp
/// since UNIX_EPOCH and the last 8 bytes are randomly
/// generated values
///
pub fn gen_uid() -> UID {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("SystemTime is before UNIX_EPOCH");

    let timestamp = now.as_secs() as i64;
    let id = rand::thread_rng().gen_range(0, 0xFF_FF_FF_FF as i64);

    ((timestamp << 32) | id) as UID
}
