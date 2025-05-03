pub mod string;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::{MAGIC_NUMBER, VERSION};

#[derive(Debug, Clone, PartialEq)]
pub enum CmdType {
    Ping = 0x01,
    Authenticate = 0x02,
    Keys = 0x03,
    Exists = 0x04,
    Expire = 0x05,
    SetString = 0x11,
    GetString = 0x12,
    DelString = 0x13,
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct Ping {
    pub timestamp: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct Pong {
    pub timestamp: u64,
}

impl Pong {
    pub fn new(timestamp: u64) -> Self {
        Self { timestamp }
    }

    pub fn default() -> Self {
        Self { timestamp: 0 }
    }

    pub fn to_result(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(self)?;
        let data_length = data.len() as u32;

        let mut result = vec![];
        result.extend(&MAGIC_NUMBER); // 魔数
        result.push(VERSION); // 版本
        result.push(CmdType::Ping as u8); // Ping 命令结果
        result.extend(&data_length.to_be_bytes()); // 长度
        result.extend(data); // Pong 数据
        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct Authenticate {
    pub secret: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct ResultAuthenticate {
    pub ok: bool,
    pub msg: String,
}

impl ResultAuthenticate {
    pub fn new(ok: bool, msg: String) -> Self {
        Self { ok, msg }
    }

    pub fn to_result(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(self)?;
        let data_length = data.len() as u32;

        let mut result = vec![];
        result.extend(&MAGIC_NUMBER); // 魔数
        result.push(VERSION); // 版本
        result.push(CmdType::Authenticate as u8); // Authenticate 命令结果
        result.extend(&data_length.to_be_bytes()); // 长度
        result.extend(data); // Authenticate 数据
        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct Keys {
    pub db: String,
    pub page: usize,
    pub size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct ResultKeys {
    pub keys: Vec<String>,
    pub total: u32,
}

impl ResultKeys {
    pub fn new(keys: Vec<String>, total: u32) -> Self {
        Self { keys, total }
    }

    pub fn to_result(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(self)?;
        let data_length = data.len() as u32;

        let mut result = vec![];
        result.extend(&MAGIC_NUMBER); // 魔数
        result.push(VERSION); // 版本
        result.push(CmdType::Keys as u8); // Keys 命令结果
        result.extend(&data_length.to_be_bytes()); // 长度
        result.extend(data); // Keys 数据
        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct Exists {
    pub db: String,
    pub key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct ResultExists {
    pub exists: bool,
}

impl ResultExists {
    pub fn new(exists: bool) -> Self {
        Self { exists }
    }

    pub fn to_result(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(self)?;
        let data_length = data.len() as u32;

        let mut result = vec![];
        result.extend(&MAGIC_NUMBER); // 魔数
        result.extend(&VERSION.to_be_bytes()); // 版本
        result.push(CmdType::Exists as u8);
        result.extend(&data_length.to_be_bytes()); // 数据长度
        result.extend(data); // 数据

        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct Expire {
    pub db: String,
    pub key: String,
    pub expire: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct ResultExpire {
    pub ok: bool,
}

impl ResultExpire {
    pub fn new(ok: bool) -> Self {
        Self { ok }
    }

    pub fn to_result(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(self)?;
        let data_length = data.len() as u32;

        let mut result = vec![];
        result.extend(&MAGIC_NUMBER); // 魔数
        result.extend(&VERSION.to_be_bytes()); // 版本
        result.push(CmdType::Expire as u8);
        result.extend(&data_length.to_be_bytes()); // 数据长度
        result.extend(data); // 数据

        Ok(result)
    }
}
