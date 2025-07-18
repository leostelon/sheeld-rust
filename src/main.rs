use std::error::Error;

use tokio::select;
use tokio::signal::ctrl_c;

use crate::discovery::swarm::SheeldGossip;
use crate::proxy::fast_socks5::SheeldFastSocks5;

mod common;
mod discovery;
mod proxy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Start SOCKS5 server
    tokio::spawn(async move {
        let listener_addr = String::from("127.0.0.1:3001");
        let fasts5 = SheeldFastSocks5::new(listener_addr);
        let res = fasts5.spawn_socks_server().await;
        match res {
            Err(e) => {
                println!("{:?}", e);
            }
            Ok(m) => {
                println!("{:?}", m);
            }
        }
    });

    // Start gossip last
    tokio::spawn(async move {
        let mut sheeld_gossipsub = SheeldGossip::new();
        let status = sheeld_gossipsub.start_libp2p().await;
        match status {
            Err(e) => {
                println!("{:?}", e);
            }
            Ok(m) => {
                println!("{:?}", m);
            }
        }
    });

    loop {
        select! {
            _ = ctrl_c()=> {
                println!("Received Ctrl-C, shutting down");
                return Ok(());
            }
        }
    }
}
