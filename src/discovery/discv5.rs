use discv5::{
    ConfigBuilder, Discv5, Error, ListenConfig,
    enr::{self, CombinedKey, NodeId},
};
use std::net::Ipv4Addr;

#[tokio::main]
pub async fn start_discv5() -> Result<(), Error> {
    let enr_key = CombinedKey::generate_secp256k1();
    let enr = enr::Enr::empty(&enr_key).unwrap();

    // configuration for the sockets to listen on
    let listen_config = ListenConfig::Ipv4 {
        ip: Ipv4Addr::UNSPECIFIED,
        port: 9000,
    };

    // default configuration
    let config = ConfigBuilder::new(listen_config).build();

    // construct the discv5 server
    let mut discv5: Discv5 = Discv5::new(enr, enr_key, config).unwrap();

    // In order to bootstrap the routing table an external ENR should be added
    // This can be done via add_enr. I.e.:
    // discv5.add_enr(<ENR>)

    // run a find_node query
    // let found_nodes = discv5.find_node(NodeId::random()).await.unwrap();
    // println!("Found nodes: {:?}", found_nodes);

    // start the discv5 server
    return discv5.start().await;
}
