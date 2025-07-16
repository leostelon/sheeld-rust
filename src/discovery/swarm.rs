use std::{
    error::Error,
    fs,
    str::{self, FromStr},
};

use futures::StreamExt;
use libp2p::{
    Multiaddr, PeerId, Swarm, SwarmBuilder,
    gossipsub::{
        Behaviour as Gossipsub, Config as GossipsubConfig, IdentTopic, MessageAuthenticity,
    },
    identity,
    kad::{self, Event as KademliaEvent, Mode, store::MemoryStore},
    noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use tokio::select;

use crate::common::models::message::Message;

#[tokio::main]
pub async fn start_libp2p() -> Result<(), Box<dyn Error>> {
    #[derive(NetworkBehaviour)]
    struct Behaviour {
        kademlia: kad::Behaviour<MemoryStore>,
        gossipsub: Gossipsub,
    }

    let local_key = identity::Keypair::generate_ed25519();
    let gossipsub = Gossipsub::new(
        MessageAuthenticity::Signed(local_key.clone()),
        GossipsubConfig::default(),
    )
    .unwrap();

    let mut swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            Ok(Behaviour {
                kademlia: kad::Behaviour::new(
                    key.public().to_peer_id(),
                    MemoryStore::new(key.public().to_peer_id()),
                ),
                gossipsub,
            })
        })?
        .build();

    swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));

    // Connect to bootstrap nodes if available
    let bootnodes = get_bootnodes().await;
    for b in bootnodes {
        swarm
            .behaviour_mut()
            .kademlia
            .add_address(&b.peer_id, b.multiaddr);
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
                handle_swarm_event(&mut swarm, event).await;
            }

        }
    }

    async fn handle_swarm_event(_swarm: &mut Swarm<Behaviour>, event: SwarmEvent<BehaviourEvent>) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("✅ Listening on: {address:?}");
            }
            SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(msg)) => {
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
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(KademliaEvent::RoutingUpdated {
                peer,
                ..
            })) => {
                println!("Discovered peer: {peer}");
            }
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id: _,
                endpoint: _,
                num_established,
                concurrent_dial_errors: _,
                established_in: _,
            } => {
                println!("🔌 Connection established with {peer_id} (remaining: {num_established})");
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

async fn get_bootnodes() -> Vec<Bootnode> {
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
    let mut bootnodes: Vec<Bootnode> = Vec::new();
    if content.is_empty() {
        return bootnodes;
    };
    for n in content.split('\n').into_iter() {
        let bn = Bootnode::new(&n.split(":").nth(0).unwrap(), &n.split(":").nth(1).unwrap());
        bootnodes.push(bn);
    }
    bootnodes
}

struct Bootnode {
    multiaddr: Multiaddr,
    peer_id: PeerId,
}

impl Bootnode {
    pub fn new(multiaddr: &str, peer_id: &str) -> Self {
        Self {
            peer_id: PeerId::from_str(peer_id).expect("Invalid PeerId"),
            multiaddr: Multiaddr::from_str(multiaddr).expect("Invalid Multiaddr"),
        }
    }
}
