use fast_socks5::{
    Result, SocksError,
    server::{DnsResolveHelper, Socks5ServerProtocol, run_tcp_proxy},
};
use tokio::net::TcpListener;

use crate::common::utils::spawn_and_log_error::{spawn_and_log_error};

pub struct SheeldFastSocks5 {
    listener_addr: String,
}

impl SheeldFastSocks5 {
    pub fn new(listener_addr: String) -> Self {
        SheeldFastSocks5 {
            listener_addr: listener_addr,
        }
    }

    pub async fn spawn_socks_server(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.listener_addr).await?;

        println!("Listen for socks connections @ {}", &self.listener_addr);

        // Standard TCP loop
        loop {
            match listener.accept().await {
                Ok((socket, s)) => {
                    println!("{s}");
                    spawn_and_log_error(SheeldFastSocks5::serve_socks5(socket));
                }
                Err(err) => {
                    println!("accept error = {:?}", err);
                }
            }
        }
    }

    async fn serve_socks5(socket: tokio::net::TcpStream) -> Result<(), SocksError> {
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
}
