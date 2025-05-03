use std::{collections::HashMap, env, ffi::OsString, path::Path, sync::Arc};

use akv::value::Entry;
use bincode::{config::standard, Decode, Encode};
use tokio::{
    fs,
    net::TcpListener,
    sync::{oneshot, RwLock},
};

use crate::server::{config, handler::handler, state::State};

use super::state::interval;

pub async fn start(
    shutdown_receiver: oneshot::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::new().unwrap();

    let mut state = State {
        config: config.clone(),
        databases: Arc::new(RwLock::new(HashMap::new())),
    };

    read_cache_file(&mut state).await;

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

#[derive(Debug, Clone, Encode, Decode)]
struct Cache {
    cache: HashMap<String, HashMap<String, Entry>>,
}

async fn read_cache_file(state: &mut State) {
    let config = standard().with_variable_int_encoding().with_little_endian();

    let home_dir = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE")) // Windows 兼容
        .unwrap_or(OsString::from("./"));
    let cache_dir = Path::new(&home_dir).join(".ahriknow/ahrikv");
    let cache_file = cache_dir.join("cache.akv");
    if cache_file.exists() {
        let encoded = fs::read(cache_file).await.unwrap();
        let msgs: Cache = bincode::decode_from_slice(&encoded, config).unwrap().0;
        let mut databases = state.databases.write().await;
        for (db, entries) in msgs.cache.iter() {
            let map = Arc::new(RwLock::new(entries.clone()));
            let jh = interval(map.clone());
            databases.insert(db.clone(), (map, jh));
        }
    }
}

async fn stop(state: State, shutdown_receiver: oneshot::Receiver<()>) {
    let _ = shutdown_receiver.await;

    let home_dir = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE")) // Windows 兼容
        .unwrap_or(OsString::from("./"));
    let cache_dir = Path::new(&home_dir).join(".ahriknow/ahrikv");
    fs::create_dir_all(&cache_dir)
        .await
        .expect("Failed to create cache directory");

    let mut cache = HashMap::new();
    let databases = state.databases.read().await;
    for (db, (map, jh)) in databases.iter() {
        jh.abort();
        cache.insert(db.clone(), map.read().await.clone());
    }

    let cache_file = cache_dir.join("cache.akv");
    let config = standard().with_variable_int_encoding().with_little_endian();
    let encoded = bincode::encode_to_vec(&Cache { cache }, config).unwrap();
    fs::write(cache_file, encoded)
        .await
        .expect("Failed to write cache file");

    println!("Shutting down...");
    std::process::exit(0);
}
