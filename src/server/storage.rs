use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::resp;

#[derive(Debug)]
pub struct Item {
    pub value: String,
    pub created: Instant,
    pub expires: Option<Duration>,
}
pub struct Storage {
    pub storage: HashMap<String, Item>,
}
impl Storage {
    pub fn new() -> Self {
        Storage {
            storage: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String, expires: Option<Duration>) -> resp::Value {
        let item = Item {
            value,
            created: Instant::now(),
            expires,
        };
        self.storage.insert(key, item);
        resp::Value::SimpleString("OK".to_string())
    }

    pub fn get(&mut self, key: String) -> resp::Value {
        let item = self.storage.get(&key);
        match item {
            Some(item) => match item.expires {
                Some(expires) if item.created.elapsed() > expires => {
                    self.storage.remove(&key);
                    resp::Value::Null
                }
                _ => resp::Value::BulkString(item.value.clone()),
            },
            None => resp::Value::Null,
        }
    }
}
impl Default for Storage {
    fn default() -> Self {
        Storage::new()
    }
}
