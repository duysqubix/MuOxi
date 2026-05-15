#![allow(unused_imports, dead_code, unused_variables)]

//! Sandbox / scratchpad. NOT a test suite. Manually exercises `db` features.

use db;
use db::cache;
use db::cache_structures::Cachable;
use db::cache_structures::socket::CacheSocket;
use redis::{Commands, Connection, FromRedisValue, Value};
use serde::{Deserialize, Serialize};

use serde_json::Result as serdeResult;
use std::collections::HashMap;
use std::error::Error;
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Person {
    uid: String,
    name: String,
    age: u8,
    email: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut socket = CacheSocket::new();
    socket.set_ip("192.168.0.1").set_port(8001).dump()?;
    let ip = socket.get_value::<String>("ip");
    println!("{:?}", ip);
    thread::sleep(Duration::from_secs(30));
    socket = socket.load()?;
    println!("{}", socket.port);

    println!("Done with main");
    Ok(())
}
