use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    time::Instant,
};

#[derive(Debug)]
pub enum Value {
    String(String),
    List(VecDeque<String>),           // 使用 VecDeque 实现双向链表
    Hash(HashMap<String, String>),    // 使用标准 HashMap
    Set(BTreeSet<String>),            // 使用 BTreeSet 实现有序集合
    SortedSet(BTreeMap<String, f64>), // 使用 BTreeMap 实现有序集合(分数作为值)
}

#[derive(Debug)]
pub struct Entry {
    pub value: Value,
    pub expires_at: Option<Instant>,
}

impl Entry {
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|t| t <= Instant::now())
            .unwrap_or(false)
    }
}
