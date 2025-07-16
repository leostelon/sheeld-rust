use crate::discovery::swarm::SheeldGossip;

mod common;
mod discovery;

fn main() {
    let mut sheeld_gossipsub = SheeldGossip::new();
    let status = sheeld_gossipsub.start_libp2p();
    match status {
        Err(e) => {
            println!("{:?}", e);
        }
        Ok(m) => {
            println!("{:?}", m);
        }
    }
}
