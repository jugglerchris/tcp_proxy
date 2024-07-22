use std::net::{Ipv4Addr, SocketAddr};

use anyhow::bail;
use clap::Parser;
use tokio::net::{lookup_host, tcp, TcpListener, TcpStream};
use tracing::{debug, debug_span, error, info, trace, Instrument};

#[derive(Parser)]
struct Cli {
    /// The address to bind to
    #[arg(long)]
    bind_addr: Option<String>,

    /// The address to forward to
    #[arg(long)]
    fwd_addr: Option<String>,

    /// A list of allowed source addresses
    #[arg(long)]
    allow_addr: Vec<Ipv4Addr>,
}

async fn do_proxy_oneway(mut reader: tcp::OwnedReadHalf, mut writer: tcp::OwnedWriteHalf) {
    match tokio::io::copy(&mut reader, &mut writer).await {
        Ok(count) => {
            debug!("Finished proxying {count} bytes");
        }
        Err(e) => {
            error!("Proxy error: {e}");
        }
    }
}

async fn do_proxy(dest: &SocketAddr, sock: TcpStream, peer: Ipv4Addr) {
    let dest_result = TcpStream::connect(dest).await; 
    info!("Connecting to {peer:?}");
    match dest_result {
        Ok(dest_sock) => {
            let (dest_read, dest_write) = dest_sock.into_split();
            let (src_read, src_write) = sock.into_split();
            tokio::spawn(do_proxy_oneway(dest_read, src_write)
                         .instrument(debug_span!("copy1")));
            tokio::spawn(do_proxy_oneway(src_read, dest_write)
                         .instrument(debug_span!("copy2")));
        }
        Err(e) => {
            error!("Error connecting: {e}");
        }
    }
}

async fn run_listener(bind_addr: &str, fwd_addr: &str,
                      allow_addr: &[Ipv4Addr]) -> anyhow::Result<()> {
    let listener = TcpListener::bind(bind_addr).await?;

    let dest = lookup_host(fwd_addr).await?.next().unwrap();
    debug!("dest = {dest:?}");

    let mut accepts = 0;
    loop {
        match listener.accept().await {
            Ok((sock, addr)) => {
                accepts += 1;
                debug!("New client {accepts}: {:?}", addr);
                match addr.ip() {
                    std::net::IpAddr::V4(addr4) => {
                        if allow_addr.iter()
                            .any(|a| *a == addr4) {
                            trace!("Allowed address");
                            do_proxy(&dest, sock, addr4)
                                .instrument(debug_span!("do_proxy", num = accepts))
                                .await;
                        } else {
                            info!("Address {addr:?} not on allow list");
                        }
                    }
                    std::net::IpAddr::V6(_) => {
                        debug!("IPv6 not supported.");
                    }
                }
            }
            Err(e) => {
                info!("Error in accept: {e}");
            }
        }
    }
//    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let bind_addr = match cli.bind_addr {
        Some(bind_addr) => bind_addr,
        None => {
            bail!("Error, --bind-addr not specified");
        }
    };

    let fwd_addr = match cli.fwd_addr {
        Some(fwd_addr) => fwd_addr,
        None => {
            bail!("Error, --fwd-addr not specified");
        }
    };

    run_listener(&bind_addr, &fwd_addr, &cli.allow_addr).await
}
