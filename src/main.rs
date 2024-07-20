use std::net::Ipv4Addr;

use anyhow::bail;
use clap::Parser;
use tokio::net::TcpListener;

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

async fn run_listener(bind_addr: &str, fwd_addr: &str,
                      allow_addr: &[Ipv4Addr]) -> anyhow::Result<()> {
    let listener = TcpListener::bind(bind_addr).await?;

    loop {
        match listener.accept().await {
            Ok((_sock, addr)) => {
                println!("New client: {:?}", addr);
                match addr.ip() {
                    std::net::IpAddr::V4(addr4) => {
                        if allow_addr.iter()
                            .any(|a| *a == addr4) {
                            println!("Allowed address");
                        } else {
                            println!("Address not on allow list");
                        }
                    }
                    std::net::IpAddr::V6(_) => {
                        eprintln!("IPv6 not supported.");
                    }
                }
            }
            Err(e) => {
                eprintln!("Error in accept: {e}");
            }
        }
    }
//    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
