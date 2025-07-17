use std::error::Error;

use tokio::select;
use tokio::signal::ctrl_c;

use crate::discovery::swarm::SheeldGossip;

mod common;
mod discovery;
mod proxy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

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
    
    // Start SOCKS5 server
    let fasts5 = proxy::fast_socks5::spawn_socks_server().await;
    match fasts5 {
        Err(e) => {
            println!("{:?}", e);
        }
        Ok(m) => {
            println!("{:?}", m);
        }
    }

    loop {
        select! {
            _ = ctrl_c()=> {
                println!("Received Ctrl-C, shutting down");
                return Ok(());
            }
        }
    }
}
