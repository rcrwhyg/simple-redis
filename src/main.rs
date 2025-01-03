use anyhow::Result;
use simple_redis::{network, Backend};
use tokio::net::TcpListener;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::fmt::init();

    let addr = "0.0.0.0:6379";
    info!("Simple-Redis-Server is listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;

    let backend = Backend::new();
    loop {
        let (stream, remote_addr) = listener.accept().await?;
        info!("Accepted connection from: {}", remote_addr);
        let cloned_backend = backend.clone();
        tokio::spawn(async move {
            match network::stream_handler(stream, cloned_backend).await {
                Ok(_) => {
                    info!("Connection from {} exited", remote_addr);
                }
                Err(e) => {
                    warn!("handle error for {}: {:?}", remote_addr, e);
                }
            }
        });
    }
}
