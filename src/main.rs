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
}

async fn run_listener(bind_addr: &str, fwd_addr: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(bind_addr).await?;

    loop {
        match listener.accept().await {
            Ok((_sock, addr)) => {
                println!("New client: {:?}", addr);
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

    run_listener(&bind_addr, &fwd_addr).await
}