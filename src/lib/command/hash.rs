use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::{MAGIC_NUMBER, VERSION};

use super::CmdType;

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct HashSet {
    pub db: String,
    pub key: String,
    pub field: String,
    pub value: String,
    pub expire: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct ResultHashSet {
    pub ok: bool,
    pub msg: String,
}

impl ResultHashSet {
    pub fn new(ok: bool, msg: String) -> Self {
        Self { ok, msg }
    }

    pub fn to_result(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(self)?;
        let data_length = data.len() as u32;

        let mut result = vec![];
        result.extend(&MAGIC_NUMBER); // 魔数
        result.extend(&VERSION.to_be_bytes()); // 版本
        result.push(CmdType::HashSet as u8);
        result.extend(&data_length.to_be_bytes()); // 数据长度
        result.extend(data); // 数据

        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct HashGet {
    pub db: String,
    pub key: String,
    pub field: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct ResultHashGet {
    pub value: Option<String>,
    pub expire: Option<u64>,
}

impl ResultHashGet {
    pub fn new(value: Option<String>, expire: Option<u64>) -> Self {
        Self { value, expire }
    }

    pub fn to_result(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(self)?;
        let data_length = data.len() as u32;

        let mut result = vec![];
        result.extend(&MAGIC_NUMBER); // 魔数
        result.extend(&VERSION.to_be_bytes()); // 版本
        result.push(CmdType::HashGet as u8);
        result.extend(&data_length.to_be_bytes()); // 数据长度
        result.extend(data); // 数据

        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct HashDel {
    pub db: String,
    pub key: String,
    pub field: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct ResultHashDel {
    pub value: Option<String>,
    pub expire: Option<u64>,
}

impl ResultHashDel {
    pub fn new(value: Option<String>, expire: Option<u64>) -> Self {
        Self { value, expire }
    }

    pub fn to_result(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(self)?;
        let data_length = data.len() as u32;

        let mut result = vec![];
        result.extend(&MAGIC_NUMBER); // 魔数
        result.extend(&VERSION.to_be_bytes()); // 版本
        result.push(CmdType::HashDel as u8);
        result.extend(&data_length.to_be_bytes()); // 数据长度
        result.extend(data); // 数据

        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct HashExists {
    pub db: String,
    pub key: String,
    pub field: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct ResultHashExists {
    pub exists: bool,
}

impl ResultHashExists {
    pub fn new(exists: bool) -> Self {
        Self { exists }
    }

    pub fn to_result(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(self)?;
        let data_length = data.len() as u32;

        let mut result = vec![];
        result.extend(&MAGIC_NUMBER); // 魔数
        result.extend(&VERSION.to_be_bytes()); // 版本
        result.push(CmdType::HashExists as u8);
        result.extend(&data_length.to_be_bytes()); // 数据长度
        result.extend(data); // 数据

        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct HashLen {
    pub db: String,
    pub key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct ResultHashLen {
    pub len: u32,
}

impl ResultHashLen {
    pub fn new(len: u32) -> Self {
        Self { len }
    }

    pub fn to_result(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(self)?;
        let data_length = data.len() as u32;

        let mut result = vec![];
        result.extend(&MAGIC_NUMBER); // 魔数
        result.extend(&VERSION.to_be_bytes()); // 版本
        result.push(CmdType::HashLen as u8);
        result.extend(&data_length.to_be_bytes()); // 数据长度
        result.extend(data); // 数据

        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct HashFields {
    pub db: String,
    pub key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct ResultHashFields {
    pub fields: Vec<String>,
    pub total: u32,
}

impl ResultHashFields {
    pub fn new(fields: Vec<String>, total: u32) -> Self {
        Self { fields, total }
    }

    pub fn to_result(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(self)?;
        let data_length = data.len() as u32;

        let mut result = vec![];
        result.extend(&MAGIC_NUMBER); // 魔数
        result.extend(&VERSION.to_be_bytes()); // 版本
        result.push(CmdType::HashFields as u8);
        result.extend(&data_length.to_be_bytes()); // 数据长度
        result.extend(data); // 数据

        Ok(result)
    }
}
