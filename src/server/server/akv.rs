use std::collections::HashMap;

use tokio::{net::TcpListener, sync::oneshot};

use crate::server::{config, handler::handler, state::State};

pub async fn start(
    shutdown_receiver: oneshot::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::new().unwrap();

    let state = State {
        config: config.clone(),
        databases: HashMap::new(),
    };

    tokio::spawn(async move {
        stop(shutdown_receiver).await;
    });

    let addr = config.get_address();
    let listener = TcpListener::bind(&addr).await?;
    println!("Server running on {}", addr);

    loop {
        let (socket, _) = listener.accept().await?;
        let s = state.clone();
        tokio::spawn(async move {
            handler(socket, s).await;
        });
    }
}

async fn stop(shutdown_receiver: oneshot::Receiver<()>) {
    let _ = shutdown_receiver.await;
    println!("Shutting down...");
    std::process::exit(0);
}
