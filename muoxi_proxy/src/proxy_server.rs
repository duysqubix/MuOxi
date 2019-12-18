//!
//! Handles all things related to ProxyServer
//! Like finding a specific connectd sender etc..
//!
use mio::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use ws::{Handler, Message, Sender};

type IpAddr = String;

#[derive(Debug, Clone)]
struct Client {
    ip: String,
    token: Token,
    sender: Sender,
}

impl Client {
    fn new(ip: IpAddr, token: Token, sender: Sender) -> Self {
        Self {
            ip: ip,
            token: token,
            sender: sender,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Clients {
    clients: HashMap<IpAddr, Client>,
}

impl Clients {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }
    pub fn insert(&mut self, ip: IpAddr, sender: Sender) -> ws::Result<()> {
        let token = sender.token();
        let client = Client::new(ip.clone(), token, sender);

        self.clients.insert(ip, client.clone());
        Ok(())
    }
}

impl fmt::Display for Clients {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt_string = "Connected Clients: \n".to_string();

        for (ip, client) in self.clients.iter() {
            let token = client.token;
            let tmp = format!("IP: {} Token: {}\n", ip, token.0);
            fmt_string.push_str(&tmp[..]);
        }

        write!(f, "{}", fmt_string)
    }
}
// ProxyServer web application handler
pub struct ProxyServer {
    out: Sender,
    clients: Rc<RefCell<Clients>>,
}

impl ProxyServer {
    pub fn new(sender: Sender, clients: Rc<RefCell<Clients>>) -> Self {
        Self {
            out: sender,
            clients: clients,
        }
    }
}

impl Handler for ProxyServer {
    // Handle messages recieved in the websocket (in this case, only on /ws)
    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        // Broadcast to all connections
        println!("{}", self.clients.borrow());
        self.out.broadcast(msg)
    }

    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        if let Some(ip_addr) = shake.remote_addr()? {
            println!("Connection opened from {}.", ip_addr);
            self.clients
                .borrow_mut()
                .insert(ip_addr, self.out.clone())
                .expect("Couldn't add client to global client list");
        // .insert(self.out.token(), self.out.clone());
        } else {
            println!("Unable to obtain client's IP address.");
        }
        Ok(())
    }
}
