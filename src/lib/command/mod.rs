pub mod string;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::{MAGIC_NUMBER, VERSION};

#[derive(Debug, Clone, PartialEq)]
pub enum CmdType {
    Ping = 0x01,
    Authenticate = 0x02,
    Keys = 0x03,
    SetString = 0x11,
    GetString = 0x12,
    DelString = 0x13,
    ExistsString = 0x14,
    ExpireString = 0x15,
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
