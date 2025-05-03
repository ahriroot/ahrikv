use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use bincode::{Decode, Encode};

use crate::utils;

#[derive(Debug, Clone, Encode, Decode)]
pub enum Value {
    String(String),
    List(VecDeque<String>),           // 使用 VecDeque 实现双向链表
    Hash(HashMap<String, String>),    // 使用标准 HashMap
    Set(BTreeSet<String>),            // 使用 BTreeSet 实现有序集合
    SortedSet(BTreeMap<String, f64>), // 使用 BTreeMap 实现有序集合(分数作为值)
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct Entry {
    pub value: Value,
    pub expires_at: Option<u64>,
}

impl Entry {
    pub fn is_expired(&self) -> bool {
        let now = utils::get_unix_timestamp();
        self.expires_at.map(|t| t <= now).unwrap_or(false)
    }
}
