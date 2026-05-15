#![deny(missing_docs)]

//! ## MuOxi Server binary entrypoint
//!
//! Builds the `Registry` + `WorldApi`, registers the framework's built-in
//! TypeClasses and commands, then spawns a `process()` task per accepted
//! TCP connection. Downstream MUDs vendor this main() body and inject
//! their own `registry.register_*` calls before the listener spawns.

use db::DatabaseHandler;
use db::cache_structures::Cachable;
use db::cache_structures::socket::CacheSocket;
use muoxi::process;
use muoxi::registry::Registry;
use muoxi::world::WorldApi;
use std::error::Error;
use std::sync::Arc;
use std::env;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        env::set_var("RUST_LOG", "info,warn,error,test");
    }
    let proxy_addr: String =
        env::var("PROXY_ADDR").unwrap_or_else(|_| "127.0.0.1:8000".to_string());

    pretty_env_logger::init();

    let world = Arc::new(WorldApi::new(DatabaseHandler::connect()));
    let registry = Arc::new(Registry::new(world.clone()));
    registry.register_builtin_types();
    muoxi::commands::register_all(&registry);

    let clients = Arc::new(Mutex::new(muoxi::comms::Server::new()));

    println!("MuOxi server listening on {}", proxy_addr);

    let listener = TcpListener::bind(&proxy_addr).await?;

    while let Ok((stream, addr)) = listener.accept().await {
        let server = Arc::clone(&clients);
        let registry = registry.clone();
        let world = world.clone();
        println!("New user! on {}", addr);

        let addr = stream.peer_addr()?;

        let mut cache_socket = CacheSocket::new();
        cache_socket.set_address(&addr).dump()?;

        tokio::spawn(async move {
            if let Err(e) = process(server, registry, world, stream, cache_socket).await {
                println!("An error occured; error={:?}", e);
            }
        });
    }

    Ok(())
}
