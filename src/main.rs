use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::{port_publication::PortPublication, session::Session};

pub mod logging;
pub mod port_publication;
pub mod session;
pub mod streaming_json_reader;
pub mod types;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init_logger(tracing::Level::DEBUG, true);

    let listener =
        TcpListener::bind(SocketAddrV4::new(Ipv4Addr::from_str("127.0.0.1")?, 0)).await?;

    info!("Server started on addr: {}", listener.local_addr()?);

    let _port_publication_handle =
        PortPublication::new(listener.local_addr()?.port(), "ServerPort")?;

    info!("Port published");

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("Accepted conn from {}", addr);

                let mut session = Session::new(stream);
                tokio::spawn(async move {
                    match session.run().await {
                        Ok(_) => info!("Session with {} finished", addr),
                        Err(e) => error!("Session with {} ended with error: {}", addr, e),
                    }
                });
            },
            Err(e) => error!("Failed to accept connection: {}", e),
        }
    }
}
