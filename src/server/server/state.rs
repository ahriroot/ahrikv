use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use akv::{
    command::{
        string::{
            DelString, ExistsString, ExpireString, GetString, ResultDelString, ResultExistsString,
            ResultExpireString, ResultGetString, SetString,
        },
        Keys, ResultKeys,
    },
    error::Error,
    value::{Entry, Value},
};
use tokio::sync::RwLock;

use super::config::Config;

pub struct State {
    pub config: Config,
    pub databases: HashMap<String, Arc<RwLock<HashMap<String, Entry>>>>,
}

impl State {
    pub fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            databases: self.databases.clone(),
        }
    }

    pub async fn keys(&mut self, data: Vec<u8>) -> Result<ResultKeys, Error> {
        let keys: Keys =
            serde_json::from_slice(&data).map_err(|e| Error::BadCommand(e.to_string()))?;
        let db = self
            .databases
            .entry(keys.db.clone())
            .or_insert_with(|| Arc::new(RwLock::new(HashMap::new())));

        let db = db.read().await;

        let total = db.len() as u32;

        let keys: Vec<String> = db
            .keys()
            .skip((keys.page - 1) * keys.size)
            .take(keys.size)
            .cloned()
            .collect();

        Ok(ResultKeys::new(keys, total))
    }

    pub async fn set_string(&mut self, data: Vec<u8>) -> Result<(), Error> {
        let string: SetString =
            serde_json::from_slice(&data).map_err(|e| Error::BadCommand(e.to_string()))?;

        let db = self
            .databases
            .entry(string.db.clone())
            .or_insert_with(|| Arc::new(RwLock::new(HashMap::new())));

        let mut db = db.write().await;

        db.insert(
            string.key.clone(),
            Entry {
                value: Value::String(string.value.clone()),
                expires_at: string
                    .expire
                    .map(|t| Instant::now() + Duration::from_secs(t)),
            },
        );

        Ok(())
    }

    pub async fn get_string(&self, data: Vec<u8>) -> Result<ResultGetString, Error> {
        let string: GetString =
            serde_json::from_slice(&data).map_err(|e| Error::BadCommand(e.to_string()))?;

        let db = self
            .databases
            .get(&string.db)
            .ok_or(Error::BadCommand("db not found".to_string()))?;

        let (value, expire) = {
            let db = db.read().await;

            let entry = db
                .get(&string.key)
                .ok_or(Error::BadCommand("key not found".to_string()))?;

            let value = match entry.value {
                Value::String(ref s) => s.clone(),
                _ => return Err(Error::BadCommand("not a string".to_string())),
            };

            let expire = entry
                .expires_at
                .map(|t| t.saturating_duration_since(Instant::now()).as_secs());

            (value, expire)
        };

        if let Some(t) = expire {
            if t == 0 {
                let mut db = db.write().await;
                db.remove(&string.key);
                return Ok(ResultGetString::new(None, None));
            }
        }

        Ok(ResultGetString::new(Some(value), expire))
    }

    pub async fn del_string(&mut self, data: Vec<u8>) -> Result<ResultDelString, Error> {
        let string: DelString =
            serde_json::from_slice(&data).map_err(|e| Error::BadCommand(e.to_string()))?;

        let db = self
            .databases
            .get(&string.db)
            .ok_or(Error::BadCommand("db not found".to_string()))?;

        let mut db = db.write().await;

        let value = db.remove(&string.key);

        match value {
            Some(entry) => {
                let value = match entry.value {
                    Value::String(ref s) => s.clone(),
                    _ => return Err(Error::Operation("not a string".to_string())),
                };

                let expire = entry
                    .expires_at
                    .map(|t| t.saturating_duration_since(Instant::now()).as_secs());

                if let Some(t) = expire {
                    if t == 0 {
                        return Ok(ResultDelString::new(None, None));
                    }
                }

                Ok(ResultDelString::new(Some(value), expire))
            }
            None => return Ok(ResultDelString::new(None, None)),
        }
    }

    pub async fn exists_string(&self, data: Vec<u8>) -> Result<ResultExistsString, Error> {
        let string: ExistsString =
            serde_json::from_slice(&data).map_err(|e| Error::BadCommand(e.to_string()))?;

        let db = self
            .databases
            .get(&string.db)
            .ok_or(Error::BadCommand("db not found".to_string()))?;

        let (value, expire) = {
            let db = db.read().await;

            let entry = db.get(&string.key);

            let value = match entry {
                Some(entry) => match entry.value {
                    Value::String(_) => true,
                    _ => false,
                },
                None => false,
            };

            let expire = entry
                .map(|entry| entry.expires_at)
                .flatten()
                .map(|t| t.saturating_duration_since(Instant::now()).as_secs());

            (value, expire)
        };
        if let Some(t) = expire {
            if t == 0 {
                let mut db = db.write().await;
                db.remove(&string.key);
                return Ok(ResultExistsString::new(false));
            }
        }

        Ok(ResultExistsString::new(value))
    }

    pub async fn expire_string(&mut self, data: Vec<u8>) -> Result<ResultExpireString, Error> {
        let string: ExpireString =
            serde_json::from_slice(&data).map_err(|e| Error::BadCommand(e.to_string()))?;

        let db = self
            .databases
            .get(&string.db)
            .ok_or(Error::BadCommand("db not found".to_string()))?;

        let mut db = db.write().await;

        let entry = db
            .get_mut(&string.key)
            .ok_or(Error::BadCommand("key not found".to_string()))?;

        if let Some(t) = entry.expires_at {
            let t = t.saturating_duration_since(Instant::now()).as_secs();
            if t == 0 {
                db.remove(&string.key);
                return Ok(ResultExpireString { ok: false });
            }
        }

        if string.expire == 0 {
            entry.expires_at = None;
        } else {
            entry.expires_at = Some(Instant::now() + Duration::from_secs(string.expire));
        }

        Ok(ResultExpireString { ok: true })
    }
}
