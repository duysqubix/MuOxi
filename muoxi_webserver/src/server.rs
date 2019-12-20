//!
//! Handles all things related to WebSocketServer
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
pub struct ClientData {
    ip: String,
    token: Token,
}

impl ClientData {
    fn new(ip: IpAddr, token: Token) -> Self {
        Self {
            ip: ip,
            token: token,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Clients {
    pub client_list: HashMap<Sender, ClientData>,
}

impl Clients {
    pub fn new() -> Self {
        Self {
            client_list: HashMap::new(),
        }
    }
    pub fn insert(&mut self, sender: Sender, ip: IpAddr) -> ws::Result<()> {
        let token = sender.token();
        let client_data = ClientData::new(ip, token);

        self.client_list.insert(sender, client_data);
        Ok(())
    }


    pub fn remove(&mut self, sender: &Sender) -> Option<ClientData>{
        self.client_list.remove(sender)
    }
   
}

impl fmt::Display for Clients {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt_string = "Connected Clients: \n".to_string();

        for (_sender, client) in self.client_list.iter() {
            let token = client.token;
            let ip = client.ip.clone();
            let tmp = format!("IP: {} Token: {}\n", ip , token.0);
            fmt_string.push_str(&tmp[..]);
        }

        write!(f, "{}", fmt_string)
    }
}
// WebSocketServer web application handler
pub struct WebSocketServer {
    out: Sender,
    clients: Rc<RefCell<Clients>>,
}

impl WebSocketServer {
    pub fn new(sender: Sender, clients: Rc<RefCell<Clients>>) -> Self {
        Self {
            out: sender,
            clients: clients,
        }
    }
}

impl Handler for WebSocketServer {
    // Handle messages recieved in the websocket (in this case, only on /ws)
    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        // Broadcast to all connections
        // let client_list = format!("[{}]", self.clients.borrow());
        // self.out.broadcast(Message::text(client_list))
        let to_all = format!("{} says, {}", "User", msg);
        let to_private = format!("You say, {}", msg);

        let clients = self.clients.borrow_mut();

        for (sender, _data) in clients.client_list.iter(){
            if sender == &self.out{
                // this is sender sending message
                self.out.send(to_private.clone())?;
            }else{
                sender.send(to_all.clone())?;
            }
        }

        Ok(())
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
        if let Err(e) = self.out.send(Message::text("Goodbye!".to_string())){
            println!("Sender most likely already closed! :{}", e);
        }

        let mut clients = self.clients.borrow_mut();

        if let Some(client_data) = clients.remove(&self.out){
            println!("Closing connection to {}:{} for reason: {}{:?}", client_data.ip, client_data.token.0, reason, code); 
        }
    }
}
