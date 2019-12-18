/// An example of a chat web application server
mod proxy_server;

use crate::proxy_server::{Clients, ProxyServer};
use std::cell::RefCell;
use std::rc::Rc;
use ws::listen;

fn main() {
    //Listen on an address and call the closure for each connection
    let clients = Rc::new(RefCell::new(Clients::new()));
    listen("127.0.0.1:8080", |out| {
        ProxyServer::new(out, clients.clone())
    })
    .unwrap()
}