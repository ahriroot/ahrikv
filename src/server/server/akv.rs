use std::collections::HashMap;
use std::sync::Arc;

use akv::{
    persistence::{config::PersistenceConfig, PersistenceEngine},
};
use tokio::{
    net::TcpListener,
    sync::{oneshot, RwLock},
};

use crate::server::{config, handler::handler, state::State};

use super::state::interval;

pub async fn start(
    shutdown_receiver: oneshot::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::new().unwrap();

    let persistence_config = PersistenceConfig::default();
    let persistence_engine = PersistenceEngine::new(persistence_config).await?;

    let mut state = State {
        config: config.clone(),
        databases: Arc::new(RwLock::new(HashMap::new())),
        persistence_engine: Some(persistence_engine.clone()),
    };

    recover_data(&mut state).await;

    let sstop = state.clone();
    tokio::spawn(async move {
        stop(sstop, shutdown_receiver).await;
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

async fn recover_data(state: &mut State) {
    if let Some(ref engine) = state.persistence_engine {
        if let Ok(databases) = engine.recover().await {
            let mut state_dbs = state.databases.write().await;
            for (db, entries) in databases.iter() {
                let map = Arc::new(RwLock::new(entries.clone()));
                let jh = interval(map.clone());
                state_dbs.insert(db.clone(), (map, jh));
            }
            println!("Data recovered from WAL and snapshot");
        }
    }
}

async fn stop(state: State, shutdown_receiver: oneshot::Receiver<()>) {
    let _ = shutdown_receiver.await;

    if let Some(ref engine) = state.persistence_engine {
        let mut cache = HashMap::new();
        let databases = state.databases.read().await;
        for (db, (map, jh)) in databases.iter() {
            jh.abort();
            cache.insert(db.clone(), map.read().await.clone());
        }

        let _ = engine.create_snapshot(&cache).await;
        println!("Snapshot created");
    }

    println!("Shutting down...");
    std::process::exit(0);
}
