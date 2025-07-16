use libp2p::{Multiaddr, PeerId};
use std::str::FromStr;

pub struct PeerInfo {
    pub multiaddr: Multiaddr,
    pub peer_id: PeerId,
}

impl PeerInfo {
    pub fn new(multiaddr: &str, peer_id: &str) -> Self {
        Self {
            peer_id: PeerId::from_str(peer_id).expect("Invalid PeerId"),
            multiaddr: Multiaddr::from_str(multiaddr).expect("Invalid Multiaddr"),
        }
    }
}
