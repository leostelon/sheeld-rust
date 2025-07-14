mod discovery;

fn main() {
    let status = discovery::swarm::start_libp2p();
    match status {
        Err(e) => {
            println!("{:?}", e);
        }
        Ok(m) => {
            println!("{:?}", m);
        }
    }
}
