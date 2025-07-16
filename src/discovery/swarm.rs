use std::{
    error::Error,
    fs,
    str::{self},
};

use futures::StreamExt;
use libp2p::{
    Swarm, SwarmBuilder,
    gossipsub::{
        Behaviour as Gossipsub, Config as GossipsubConfig, IdentTopic, MessageAuthenticity,
    },
    noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use tokio::select;

use crate::common::models::{message::Message, peer_info::PeerInfo};

#[derive(NetworkBehaviour)]
struct SheeldBehaviour {
    gossipsub: Gossipsub,
}

pub struct SheeldGossip {
    peers: Vec<PeerInfo>,
}

impl SheeldGossip {
    pub fn new() -> Self {
        Self { peers: Vec::new() }
    }
    #[tokio::main]
    pub async fn start_libp2p(self: &mut Self) -> Result<(), Box<dyn Error>> {
        let mut swarm = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key| {
                let gossipsub = Gossipsub::new(
                    MessageAuthenticity::Signed(key.clone()),
                    GossipsubConfig::default(),
                )
                .unwrap();
                Ok(SheeldBehaviour { gossipsub })
            })?
            .build();

        // Connect to bootstrap nodes if available
        let bootnodes = get_bootnodes().await;
        for b in bootnodes {
            match swarm.dial(b.multiaddr) {
                Err(e) => println!("{e}"),
                Ok(_) => println!("Connected to peer: {:?}", b.peer_id),
            }
            swarm
                .behaviour_mut()
                .gossipsub
                .add_explicit_peer(&b.peer_id);
        }

        // Listen to topics
        let topic = IdentTopic::new("ping");
        swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();

        // Listen on all interfaces and whatever port the OS assigns.
        swarm.listen_on("/ip4/0.0.0.0/tcp/3000".parse()?)?;
        println!("PEER ID: {:?}", swarm.local_peer_id().to_base58());

        // Kick it off.
        loop {
            select! {
                event = swarm.select_next_some() =>  {
                    self.handle_swarm_event(&mut swarm, event).await;
                }
            }
        }
    }

    async fn handle_swarm_event(
        &mut self,
        _swarm: &mut Swarm<SheeldBehaviour>,
        event: SwarmEvent<SheeldBehaviourEvent>,
    ) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("✅ Listening on: {address:?}");
            }
            SwarmEvent::Behaviour(SheeldBehaviourEvent::Gossipsub(msg)) => {
                println!("Received: {:?}", msg);
                match msg {
                    libp2p::gossipsub::Event::Message {
                        propagation_source: _,
                        message_id: _,
                        message,
                    } => {
                        let dmsg: Result<Message<String>, serde_json::Error> =
                            serde_json::from_str(&String::from_utf8(message.data).unwrap() as &str);
                        match dmsg {
                            Err(e) => println!("{e}"),
                            Ok(e) => {
                                println!("Received: {:?}", e.data);
                            }
                        }
                    }
                    _ => {}
                }
            }
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id: _,
                endpoint,
                num_established,
                concurrent_dial_errors: _,
                established_in: _,
            } => {
                println!("🔌 Connection established with {peer_id} (remaining: {num_established})");
                self.peers.push(PeerInfo {
                    multiaddr: endpoint.get_remote_address().clone(),
                    peer_id,
                });
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id: _,
                endpoint: _,
                num_established,
                cause,
            } => {
                println!(
                    "🔌 Connection closed with {peer_id} (remaining: {num_established}) {}",
                    cause.unwrap()
                );
            }
            _ => {}
        }
    }
}

async fn get_bootnodes() -> Vec<PeerInfo> {
    let bootstrap_nodes_file_path: &str = "bootstrap_nodes.txt";
    let file_result = fs::read_to_string(bootstrap_nodes_file_path);
    let mut content = String::new();
    match file_result {
        Ok(c) => {
            content = c;
        }
        Err(e) => {
            println!("{:?}", e)
        }
    }
    let mut bootnodes: Vec<PeerInfo> = Vec::new();
    if content.is_empty() {
        return bootnodes;
    };
    for n in content.split('\n').into_iter() {
        let bn = PeerInfo::new(&n.split(":").nth(0).unwrap(), &n.split(":").nth(1).unwrap());
        bootnodes.push(bn);
    }
    bootnodes
}
