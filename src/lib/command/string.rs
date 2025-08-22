use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::{MAGIC_NUMBER, VERSION};

use super::CmdType;

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct SetString {
    pub db: String,
    pub key: String,
    pub value: String,
    pub expire: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct ResultSetString {
    pub ok: bool,
    pub msg: String,
}

impl ResultSetString {
    pub fn new(ok: bool, msg: String) -> Self {
        Self { ok, msg }
    }

    pub fn to_result(&self, seq: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(self)?;
        let data_length = data.len() as u32;

        let mut result = vec![];
        result.extend(&MAGIC_NUMBER); // 魔数
        result.extend(&VERSION.to_be_bytes()); // 版本
        result.push(CmdType::SetString as u8);
        result.extend(&seq.to_be_bytes()); // 序列号
        result.extend(&data_length.to_be_bytes()); // 数据长度
        result.extend(data); // 数据

        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct GetString {
    pub db: String,
    pub key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct ResultGetString {
    pub value: Option<String>,
    pub expire: Option<u64>,
}

impl ResultGetString {
    pub fn new(value: Option<String>, expire: Option<u64>) -> Self {
        Self { value, expire }
    }

    pub fn to_result(&self, seq: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(self)?;
        let data_length = data.len() as u32;

        let mut result = vec![];
        result.extend(&MAGIC_NUMBER); // 魔数
        result.extend(&VERSION.to_be_bytes()); // 版本
        result.push(CmdType::GetString as u8);
        result.extend(&seq.to_be_bytes()); // 序列号
        result.extend(&data_length.to_be_bytes()); // 数据长度
        result.extend(data); // 数据

        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct DelString {
    pub db: String,
    pub key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Encode, Decode)]
pub struct ResultDelString {
    pub value: Option<String>,
    pub expire: Option<u64>,
}

impl ResultDelString {
    pub fn new(value: Option<String>, expire: Option<u64>) -> Self {
        Self { value, expire }
    }

    pub fn to_result(&self, seq: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(self)?;
        let data_length = data.len() as u32;

        let mut result = vec![];
        result.extend(&MAGIC_NUMBER); // 魔数
        result.extend(&VERSION.to_be_bytes()); // 版本
        result.push(CmdType::DelString as u8);
        result.extend(&seq.to_be_bytes()); // 序列号
        result.extend(&data_length.to_be_bytes()); // 数据长度
        result.extend(data); // 数据

        Ok(result)
    }
}
