/// An example of a chat web application server
extern crate ws;
use mio::Token as Id;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use ws::{listen, Handler, Message, Result, Sender};

type IpAddr = String;

#[derive(Debug)]
struct Clients {
    clients: HashMap<Id, Sender>,
}

impl Clients {
    fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }
}
// Server web application handler
struct Server {
    out: Sender,
    clients: Rc<RefCell<Clients>>,
}

impl Server {
    fn new(sender: Sender, clients: Rc<RefCell<Clients>>) -> Self {
        Self {
            out: sender,
            clients: clients,
        }
    }
}

impl Handler for Server {
    // Handle messages recieved in the websocket (in this case, only on /ws)
    fn on_message(&mut self, msg: Message) -> Result<()> {
        // Broadcast to all connections
        println!("{:?}", self.clients.borrow());
        self.out.broadcast(msg)
    }

    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        if let Some(ip_addr) = shake.remote_addr()? {
            println!("Connection opened from {}.", ip_addr);
            self.clients
                .borrow_mut()
                .clients
                .insert(self.out.token(), self.out.clone());
        } else {
            println!("Unable to obtain client's IP address.");
        }
        Ok(())
    }
}

fn main() {
    //Listen on an address and call the closure for each connection
    let clients = Rc::new(RefCell::new(Clients::new()));
    listen("127.0.0.1:8080", |out| Server::new(out, clients.clone())).unwrap()
}
