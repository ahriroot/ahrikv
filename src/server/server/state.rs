use std::{collections::HashMap, sync::Arc};

use akv::{
    command::{
        hash::{
            HashDel, HashExists, HashFields, HashGet, HashLen, HashSet, ResultHashDel,
            ResultHashExists, ResultHashFields, ResultHashGet, ResultHashLen, ResultHashSet,
        },
        string::{
            DelString, GetString, ResultDelString, ResultGetString, ResultSetString, SetString,
        },
        Exists, Expire, Keys, ResultExists, ResultExpire, ResultKeys,
    },
    error::Error,
    utils,
    value::{Entry, Value},
};
use tokio::sync::RwLock;

use super::config::Config;

type Taskhandle = tokio::task::JoinHandle<()>;
type Database = Arc<RwLock<HashMap<String, Entry>>>;
type Databases = Arc<RwLock<HashMap<String, (Database, Taskhandle)>>>;

pub struct State {
    pub config: Config,
    pub databases: Databases,
}

impl State {
    pub fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            databases: self.databases.clone(),
        }
    }

    async fn get_db(&mut self, name: String) -> Database {
        {
            let dbs = self.databases.read().await;
            if let Some((db, _jh)) = dbs.get(&name) {
                return db.clone();
            }
        }
        {
            let mut dbs = self.databases.write().await;
            let map = Arc::new(RwLock::new(HashMap::new()));
            let jh = interval(map.clone());
            dbs.insert(name, (map.clone(), jh));
            return map;
        }
    }

    pub async fn keys(&mut self, data: Vec<u8>) -> Result<ResultKeys, Error> {
        let keys: Keys =
            serde_json::from_slice(&data).map_err(|e| Error::BadCommand(e.to_string()))?;

        let db = self.get_db(keys.db).await;
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

    pub async fn exists(&mut self, data: Vec<u8>) -> Result<ResultExists, Error> {
        let string: Exists =
            serde_json::from_slice(&data).map_err(|e| Error::BadCommand(e.to_string()))?;

        let db = self.get_db(string.db).await;

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

            let expire = entry.map(|entry| entry.expires_at).flatten().map(|t| {
                let now = utils::get_unix_timestamp();
                if t > now {
                    t - now
                } else {
                    0
                }
            });

            (value, expire)
        };
        if let Some(t) = expire {
            if t == 0 {
                let mut db = db.write().await;
                db.remove(&string.key);
                return Ok(ResultExists::new(false));
            }
        }

        Ok(ResultExists::new(value))
    }

    pub async fn expire(&mut self, data: Vec<u8>) -> Result<ResultExpire, Error> {
        let string: Expire =
            serde_json::from_slice(&data).map_err(|e| Error::BadCommand(e.to_string()))?;

        let db = self.get_db(string.db).await;

        let mut db = db.write().await;

        let entry = db
            .get_mut(&string.key)
            .ok_or(Error::BadCommand("key not found".to_string()))?;

        if let Some(t) = entry.expires_at {
            let now = utils::get_unix_timestamp();
            if t <= now {
                db.remove(&string.key);
                return Ok(ResultExpire { ok: false });
            }
        }

        if string.expire == 0 {
            entry.expires_at = None;
        } else {
            entry.expires_at = Some(utils::get_unix_timestamp() + string.expire);
        }

        Ok(ResultExpire { ok: true })
    }

    pub async fn set_string(&mut self, data: Vec<u8>) -> Result<ResultSetString, Error> {
        let string: SetString =
            serde_json::from_slice(&data).map_err(|e| Error::BadCommand(e.to_string()))?;

        let db = self.get_db(string.db).await;
        let mut db = db.write().await;

        db.insert(
            string.key.clone(),
            Entry {
                value: Value::String(string.value.clone()),
                expires_at: string.expire.map(|t| utils::get_unix_timestamp() + t),
            },
        );

        Ok(ResultSetString::new(true, "ok".to_string()))
    }

    pub async fn get_string(&mut self, data: Vec<u8>) -> Result<ResultGetString, Error> {
        let string: GetString =
            serde_json::from_slice(&data).map_err(|e| Error::BadCommand(e.to_string()))?;

        let db = self.get_db(string.db).await;

        let (value, expire) = {
            let db = db.read().await;
            let entry = db.get(&string.key);

            match entry {
                Some(e) => match e.value {
                    Value::String(ref s) => (
                        s.clone(),
                        e.expires_at.map(|t| {
                            let now = utils::get_unix_timestamp();
                            if t > now {
                                t - now
                            } else {
                                0
                            }
                        }),
                    ),
                    _ => return Err(Error::BadCommand("not a string".to_string())),
                },
                _ => return Ok(ResultGetString::new(None, None)),
            }
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

        let db = self.get_db(string.db).await;
        let mut db = db.write().await;

        let value = db.remove(&string.key);

        match value {
            Some(entry) => {
                let value = match entry.value {
                    Value::String(ref s) => s.clone(),
                    _ => return Err(Error::Operation("not a string".to_string())),
                };

                let expire = entry.expires_at.map(|t| {
                    let now = utils::get_unix_timestamp();
                    if t > now {
                        t - now
                    } else {
                        0
                    }
                });

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

    pub async fn hash_set(&mut self, data: Vec<u8>) -> Result<ResultHashSet, Error> {
        let hash: HashSet =
            serde_json::from_slice(&data).map_err(|e| Error::BadCommand(e.to_string()))?;

        let db = self.get_db(hash.db).await;
        let mut db = db.write().await;

        if let Some(entry) = db.get_mut(&hash.key) {
            match entry.value {
                Value::Hash(ref mut map) => {
                    map.insert(hash.field, hash.value);
                }
                _ => return Err(Error::BadCommand("not a hash".to_string())),
            }
            if let Some(t) = hash.expire {
                entry.expires_at = Some(utils::get_unix_timestamp() + t);
            }
        } else {
            let mut map = HashMap::new();
            map.insert(hash.field, hash.value);
            db.insert(
                hash.key.clone(),
                Entry {
                    value: Value::Hash(map),
                    expires_at: hash.expire.map(|t| utils::get_unix_timestamp() + t),
                },
            );
        };

        return Ok(ResultHashSet::new(true, "ok".to_string()));
    }

    pub async fn hash_get(&mut self, data: Vec<u8>) -> Result<ResultHashGet, Error> {
        let hash: HashGet =
            serde_json::from_slice(&data).map_err(|e| Error::BadCommand(e.to_string()))?;

        let db = self.get_db(hash.db).await;

        let (value, expire) = {
            let db = db.read().await;
            let entry = db.get(&hash.key);

            match entry {
                Some(e) => match e.value {
                    Value::Hash(ref map) => (
                        map.get(&hash.field).cloned(),
                        e.expires_at.map(|t| {
                            let now = utils::get_unix_timestamp();
                            if t > now {
                                t - now
                            } else {
                                0
                            }
                        }),
                    ),
                    _ => return Err(Error::BadCommand("not a hash".to_string())),
                },
                _ => return Ok(ResultHashGet::new(None, None)),
            }
        };

        if let Some(t) = expire {
            if t == 0 {
                let mut db = db.write().await;
                db.remove(&hash.key);
                return Ok(ResultHashGet::new(None, None));
            }
        }

        Ok(ResultHashGet::new(value, expire))
    }

    pub async fn hash_del(&mut self, data: Vec<u8>) -> Result<ResultHashDel, Error> {
        let hash: HashDel =
            serde_json::from_slice(&data).map_err(|e| Error::BadCommand(e.to_string()))?;

        let db = self.get_db(hash.db).await;
        let mut db = db.write().await;

        let (value, expire) = {
            let entry = db
                .get_mut(&hash.key)
                .ok_or(Error::BadCommand("key not found".to_string()))?;

            match entry.value {
                Value::Hash(ref mut map) => {
                    let value = map.remove(&hash.field);
                    if let Some(t) = entry.expires_at {
                        if t == 0 {
                            return Ok(ResultHashDel::new(None, None));
                        } else {
                            (value, Some(t))
                        }
                    } else {
                        (value, None)
                    }
                }
                _ => return Err(Error::BadCommand("not a hash".to_string())),
            }
        };

        Ok(ResultHashDel::new(value, expire))
    }

    pub async fn hash_exists(&mut self, data: Vec<u8>) -> Result<ResultHashExists, Error> {
        let hash: HashExists =
            serde_json::from_slice(&data).map_err(|e| Error::BadCommand(e.to_string()))?;

        let db = self.get_db(hash.db).await;

        let (exists, expire) = {
            let db = db.read().await;
            let entry = db.get(&hash.key);

            match entry {
                Some(e) => match e.value {
                    Value::Hash(ref map) => (
                        map.contains_key(&hash.field),
                        e.expires_at.map(|t| {
                            let now = utils::get_unix_timestamp();
                            if t > now {
                                t - now
                            } else {
                                0
                            }
                        }),
                    ),
                    _ => return Err(Error::BadCommand("not a hash".to_string())),
                },
                _ => return Ok(ResultHashExists::new(false)),
            }
        };
        if let Some(t) = expire {
            if t == 0 {
                let mut db = db.write().await;
                db.remove(&hash.key);
                return Ok(ResultHashExists::new(false));
            }
        }

        Ok(ResultHashExists::new(exists))
    }

    pub async fn hash_len(&mut self, data: Vec<u8>) -> Result<ResultHashLen, Error> {
        let hash: HashLen =
            serde_json::from_slice(&data).map_err(|e| Error::BadCommand(e.to_string()))?;

        let db = self.get_db(hash.db).await;

        let (len, expire) = {
            let db = db.read().await;
            let entry = db.get(&hash.key);

            match entry {
                Some(e) => match e.value {
                    Value::Hash(ref map) => (
                        map.len() as u32,
                        e.expires_at.map(|t| {
                            let now = utils::get_unix_timestamp();
                            if t > now {
                                t - now
                            } else {
                                0
                            }
                        }),
                    ),
                    _ => return Err(Error::BadCommand("not a hash".to_string())),
                },
                _ => return Ok(ResultHashLen::new(0)),
            }
        };

        if let Some(t) = expire {
            if t == 0 {
                let mut db = db.write().await;
                db.remove(&hash.key);
                return Ok(ResultHashLen::new(0));
            }
        }

        Ok(ResultHashLen::new(len))
    }

    pub async fn hash_fields(&mut self, data: Vec<u8>) -> Result<ResultHashFields, Error> {
        let hash: HashFields =
            serde_json::from_slice(&data).map_err(|e| Error::BadCommand(e.to_string()))?;

        let db = self.get_db(hash.db).await;

        let (keys, expire) = {
            let db = db.read().await;
            let entry = db.get(&hash.key);

            match entry {
                Some(e) => match e.value {
                    Value::Hash(ref map) => (
                        map.keys().cloned().collect::<Vec<String>>(),
                        e.expires_at.map(|t| {
                            let now = utils::get_unix_timestamp();
                            if t > now {
                                t - now
                            } else {
                                0
                            }
                        }),
                    ),
                    _ => return Err(Error::BadCommand("not a hash".to_string())),
                },
                _ => return Ok(ResultHashFields::new(Vec::new(), 0)),
            }
        };

        if let Some(t) = expire {
            if t == 0 {
                let mut db = db.write().await;
                db.remove(&hash.key);
                return Ok(ResultHashFields::new(Vec::new(), 0));
            }
        }

        let len = keys.len() as u32;

        Ok(ResultHashFields::new(keys, len))
    }
}

pub fn interval(map: Database) -> Taskhandle {
    tokio::spawn(async move {
        loop {
            let to_remove = {
                let db = map.read().await;
                let now = utils::get_unix_timestamp();
                let mut to_remove = Vec::new();
                for (key, entry) in db.iter() {
                    if let Some(t) = entry.expires_at {
                        if t <= now {
                            to_remove.push(key.clone());
                        }
                    }
                }
                to_remove
            };
            {
                let mut db = map.write().await;
                for key in to_remove {
                    db.remove(&key);
                }
            }
            let _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    })
}
