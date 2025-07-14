use std::{error::Error, str::FromStr};

use futures::StreamExt;
use libp2p::{
    Swarm, SwarmBuilder,
    kad::{
        self, Mode, RecordKey,
        store::{MemoryStore, RecordStore},
    },
    mdns, noise,
    swarm::{self, NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use tokio::{
    io::{self, AsyncBufReadExt},
    select,
};

#[tokio::main]
pub async fn start_libp2p() -> Result<(), Box<dyn Error>> {
    #[derive(Debug, PartialEq)]
    enum InputCommand {
        P,  // Connected peers
        PN, // Parent Node
    }

    impl FromStr for InputCommand {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "p" => Ok(InputCommand::P),
                "pn" => Ok(InputCommand::PN),
                _ => Err(String::from("Invalid parameter")),
            }
        }
    }

    #[derive(NetworkBehaviour)]
    struct Behaviour {
        kademlia: kad::Behaviour<MemoryStore>,
        mdns: mdns::tokio::Behaviour,
    }

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
                mdns: mdns::tokio::Behaviour::new(
                    mdns::Config {
                        ..Default::default()
                    },
                    key.public().to_peer_id(),
                )?,
            })
        })?
        .build();

    swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));

    // Listen on all interfaces and whatever port the OS assigns.
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    println!("PEER ID: {:?}", swarm.local_peer_id().to_base58());
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // Kick it off.
    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                handle_input_line(&swarm, line);
            }
            event = swarm.select_next_some() =>  {
                handle_swarm_event(&mut swarm, event).await;
            }

        }
    }

    fn handle_input_line(swarm: &Swarm<Behaviour>, input: String) {
        match InputCommand::from_str(&input) {
            Ok(v) => match v {
                InputCommand::P => {
                    println!("Entered commnad: {:?}", v);
                    for p in swarm.connected_peers() {
                        println!("{:?}", p)
                    }
                }
                InputCommand::PN => {
                    println!("Entered commnad: {:?}", v);
                }
            },
            Err(e) => {
                println!("{}", e)
            }
        }
    }

    async fn handle_swarm_event(swarm: &mut Swarm<Behaviour>, event: SwarmEvent<BehaviourEvent>) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("✅ Listening on: {address:?}");
            }

            SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, multi_address) in list {
                    println!("Self: {}, Peer: {}", swarm.local_peer_id(), peer_id);
                    swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, multi_address);
                    let ps = swarm.behaviour_mut().kademlia.get_closest_peers(peer_id);
                    println!("🟢 Discovered peer: {peer_id}, {:?}", ps);
                }
            }

            SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                for (peer_id, _) in list {
                    println!("⚠️ Peer expired (mdns timeout): {peer_id}");
                }
            }

            SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id,
                endpoint,
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
