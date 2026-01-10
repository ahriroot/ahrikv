use std::collections::HashMap;

use tokio::{sync::mpsc, time::interval};

use crate::{
    persistence::{
        config::PersistenceConfig, snapshot::SnapshotManager, wal::WalEntry, WalManager,
    },
    value::Entry,
};

#[derive(Clone)]
pub struct PersistenceEngine {
    #[allow(dead_code)]
    config: PersistenceConfig,
    wal_manager: WalManager,
    snapshot_manager: SnapshotManager,
    #[allow(dead_code)]
    snapshot_trigger: mpsc::Sender<()>,
}

impl PersistenceEngine {
    pub async fn new(config: PersistenceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let wal_manager = WalManager::new(config.clone()).await?;
        let snapshot_manager = SnapshotManager::new(config.clone()).await?;

        let (snapshot_trigger, mut snapshot_receiver) = mpsc::channel::<()>(1);

        let engine = Self {
            config: config.clone(),
            wal_manager,
            snapshot_manager,
            snapshot_trigger: snapshot_trigger.clone(),
        };

        let _snapshot_manager = engine.snapshot_manager.clone();
        let wal_manager = engine.wal_manager.clone();
        let config = config.clone();

        tokio::spawn(async move {
            let mut snapshot_timer = interval(config.snapshot_interval);
            snapshot_timer.tick().await;

            loop {
                tokio::select! {
                    _ = snapshot_timer.tick() => {
                        let count = wal_manager.get_entry_count().await;
                        if count >= config.snapshot_wal_threshold {
                            let _ = snapshot_trigger.send(()).await;
                        }
                    }
                    Some(_) = snapshot_receiver.recv() => {
                    }
                }
            }
        });

        Ok(engine)
    }

    pub async fn set(
        &self,
        db: String,
        key: String,
        entry: Entry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.wal_manager
            .append(WalEntry::Set { db, key, entry })
            .await
    }

    pub async fn delete(&self, db: String, key: String) -> Result<(), Box<dyn std::error::Error>> {
        self.wal_manager.append(WalEntry::Delete { db, key }).await
    }

    pub async fn expire(
        &self,
        db: String,
        key: String,
        expires_at: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.wal_manager
            .append(WalEntry::Expire { db, key, expires_at })
            .await
    }

    pub async fn create_snapshot(
        &self,
        databases: &HashMap<String, HashMap<String, Entry>>,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let timestamp = self
            .snapshot_manager
            .create_snapshot(databases)
            .await?;

        self.wal_manager.cleanup_old_wals(timestamp).await?;

        Ok(timestamp)
    }

    pub async fn recover(
        &self,
    ) -> Result<HashMap<String, HashMap<String, Entry>>, Box<dyn std::error::Error>> {
        let mut databases = HashMap::new();

        if let Some(snapshot) = self.snapshot_manager.load_latest_snapshot().await? {
            databases = snapshot.databases;
        }

        let wal_entries = self.wal_manager.recover().await?;

        for entry in wal_entries {
            match entry {
                WalEntry::Set { db, key, entry } => {
                    let db_map = databases.entry(db).or_insert_with(HashMap::new);
                    db_map.insert(key, entry);
                }
                WalEntry::Delete { db, key } => {
                    if let Some(db_map) = databases.get_mut(&db) {
                        db_map.remove(&key);
                    }
                }
                WalEntry::Expire {
                    db,
                    key,
                    expires_at,
                } => {
                    if let Some(db_map) = databases.get_mut(&db) {
                        if let Some(entry) = db_map.get_mut(&key) {
                            entry.expires_at = Some(expires_at);
                        }
                    }
                }
            }
        }

        Ok(databases)
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
