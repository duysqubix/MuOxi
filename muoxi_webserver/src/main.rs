/// An example of a chat web application server
mod server;

use crate::server::{Clients, WebSocketServer};
use env_logger;
use std::cell::RefCell;
use std::rc::Rc;
use ws::listen;

fn main() {
    env_logger::init();
    //Listen on an address and call the closure for each connection
    let clients = Rc::new(RefCell::new(Clients::new()));
    listen("127.0.0.1:8080", |out| {
        WebSocketServer::new(out, clients.clone())
    })
    .unwrap()
}
