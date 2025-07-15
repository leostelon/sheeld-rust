use std::{
    error::Error,
    str::{self, FromStr},
};

use futures::StreamExt;
use libp2p::{
    Swarm, SwarmBuilder,
    gossipsub::{
        Behaviour as Gossipsub, Config as GossipsubConfig, IdentTopic, MessageAuthenticity,
    },
    identity,
    kad::{self, Mode, store::MemoryStore},
    noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use tokio::select;

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

    // Listen to topics
    let topic = IdentTopic::new("ping");
    let own_topic = IdentTopic::new(swarm.local_peer_id().to_string());
    swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&own_topic)
        .unwrap();

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
