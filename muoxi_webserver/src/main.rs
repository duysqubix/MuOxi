/// An example of a chat web application server
mod server;

use crate::server::{Clients, WebSocketServer};
use env_logger;
use std::sync::{Arc, Mutex};
use ws::listen;
use std::thread;
use std::net::TcpStream;
use std::io::prelude::*;


struct InternalTcpClient{
    clients: Arc<Mutex<Clients>>,
    stream: TcpStream
}

impl InternalTcpClient{
    fn new(clients: Arc<Mutex<Clients>>, stream: TcpStream) -> Self{
        Self{
            clients: clients,
            stream: stream
        }
    }
}

fn main() {
    env_logger::init();
    //Listen on an address and call the closure for each connection
    let clients = Arc::new(Mutex::new(Clients::new()));

    let websocket_listener = thread::spawn({
        let c = clients.clone();
        move || {
        listen("127.0.0.1:8080", |out| {
            WebSocketServer::new(out, c.clone())
        })
        .unwrap()
    }});

    let tcp_proxy = thread::spawn(move || {
        // connect to an existing tcp server and forward input from websocket to this channel.
        let addr = "127.0.0.1:8000";
        let stream = TcpStream::connect(&addr).unwrap();

        let mut client = InternalTcpClient::new(clients.clone(), stream);
        loop{
            // first check to see if client has messaged stored in in_buf and transfer write to port
            {
                let mut clients = client.clients.lock().unwrap();
                for (_sender, data) in clients.client_list.iter_mut(){
                    if data.in_buf.len() > 0{
                        // 
                        println!("sender: {}:{} {:?}",data.ip, data.token.0, data.in_buf);
                        client.stream.write(&data.in_buf[..]).unwrap();
                        data.in_buf = Vec::new();
                    }
                }

            }
            // second check check to see if port has been written to
            // evoke the clients.sender to send message back to websocket client..
        }
    });

    websocket_listener.join().unwrap();
    // tcp_proxy.join().unwrap();

}
