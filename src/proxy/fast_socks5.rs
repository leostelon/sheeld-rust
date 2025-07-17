use std::net::{IpAddr, Ipv4Addr};

use fast_socks5::{
    Result, SocksError,
    server::{DnsResolveHelper, Socks5ServerProtocol, run_tcp_proxy},
};
use structopt::StructOpt;
use tokio::net::TcpListener;
use tokio::task;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "socks5-server",
    about = "A simple implementation of a socks5-server."
)]
enum AuthMode {
    NoAuth,
    Password {
        #[structopt(short, long)]
        username: String,

        #[structopt(short, long)]
        password: String,
    },
}

pub async fn spawn_socks_server() -> Result<()> {
    let listen_addr = String::from("127.0.0.1:3001");
    let listener = TcpListener::bind(&listen_addr).await?;

    println!("Listen for socks connections @ {}", &listen_addr);

    // Standard TCP loop
    loop {
        match listener.accept().await {
            Ok((socket, s)) => {
                println!("{s}");
                spawn_and_log_error(serve_socks5(socket));
            }
            Err(err) => {
                println!("accept error = {:?}", err);
            }
        }
    }
}

async fn serve_socks5(socket: tokio::net::TcpStream) -> Result<(), SocksError> {
    let public_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let (proto, cmd, target_addr) = Socks5ServerProtocol::accept_no_auth(socket)
        .await?
        .read_command()
        .await?
        .resolve_dns()
        .await?;

    println!("Target address: {target_addr}, cmd: {cmd:?}");
    run_tcp_proxy(proto, &target_addr, 3, false).await?;
    Ok(())
}

fn spawn_and_log_error<F>(fut: F) -> task::JoinHandle<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    task::spawn(async move {
        match fut.await {
            Ok(()) => {}
            Err(err) => println!("{:#}", &err),
        }
    })
}
