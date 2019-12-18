//!
//! Handles all things related to ProxyServer
//! Like finding a specific connectd sender etc..
//!
use mio::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::rc::Rc;
use ws::{CloseCode, Error, ErrorKind, Handler, Message, Request, Response, Sender};

type IpAddr = String;

struct HTML;
impl HTML {
    fn get_index() -> std::io::Result<Vec<u8>> {
        let contents = fs::read_to_string("static/index.html".to_string())?;
        Ok(Vec::from(contents.as_bytes()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    clients: HashMap<Client, IpAddr>,
}

impl Clients {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }
    pub fn insert(&mut self, sender: Sender, ip: IpAddr) -> ws::Result<()> {
        let token = sender.token();
        let client = Client::new(ip.clone(), token, sender);

        self.clients.insert(client.clone(), ip);
        Ok(())
    }

    pub fn remove(&mut self, sender: &Sender) -> ws::Result<()> {
        //     let client: Option<Client>;

        //     for (client, _ip) in self.clients.iter() {
        //         if client.sender == *sender {
        //             println!("Removing :{:?}", client);
        //             client = Some(client);
        //             break;
        //         } else {
        //             client = None;
        //         }
        //     }

        //     if let Some(client) = client {
        //         self.clients
        //             .remove(client)
        //             .expect("Removing non existant client");
        // }

        // if let Some(client) = self.get(sender) {
        //     let c = client;
        //     if let Some(_) = self.clients.remove(c) {
        //         // println!("Removed client: {:?}", c);
        //     }
        // }

        Ok(())
    }
}

impl fmt::Display for Clients {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt_string = "Connected Clients: \n".to_string();

        for (client, ip) in self.clients.iter() {
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
    fn on_message(&mut self, _msg: Message) -> ws::Result<()> {
        // Broadcast to all connections
        let client_list = format!("[{}]", self.clients.borrow());
        self.out.broadcast(Message::text(client_list))
    }

    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        if let Some(ip_addr) = shake.remote_addr()? {
            println!("Connection opened from {}.", ip_addr);
            self.clients
                .borrow_mut()
                .insert(self.out.clone(), ip_addr)
                .expect("Couldn't add client to global client list");
        } else {
            println!("Unable to obtain client's IP address.");
        }
        Ok(())
    }

    fn on_request(&mut self, req: &Request) -> ws::Result<Response> {
        let contents = HTML::get_index().unwrap();
        match req.resource() {
            // The default trait implementation
            "/ws" => Response::from_request(req),

            // Create a custom response
            "/" => Ok(Response::new(200, "OK", contents)),

            _ => Ok(Response::new(404, "Not Found", b"404 - Not Found".to_vec())),
        }
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("Closing: {:?}/{}", code, reason);
        let mut client = self.clients.borrow_mut();
        client.remove(&self.out).expect("Couldn't remove client");
        // self.clients.borrow_mut().re
    }
}
