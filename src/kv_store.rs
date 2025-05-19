use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

const STORE_FILE: &str = "store.json";

#[derive(Serialize, Deserialize)]
struct InnerStore {
    map: HashMap<String, String>,
}

pub struct KvStore {
    store: Arc<RwLock<InnerStore>>,
}

impl KvStore {
    pub fn new() -> Result<Self, String> {
        let map = if Path::new(STORE_FILE).exists() {
            match fs::read_to_string(STORE_FILE) {
                Ok(data) => match serde_json::from_str(&data) {
                    Ok(map) => map,
                    Err(_) => {
                        eprintln!("Corrupt or invalid store file. Starting with empty store.");
                        InnerStore { map: HashMap::new() }
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read store file: {}", e);
                    InnerStore { map: HashMap::new() }
                }
            }
        } else {
            InnerStore { map: HashMap::new() }
        };

        Ok(KvStore {
            store: Arc::new(RwLock::new(map)),
        })
    }

    pub fn set(&self, key: String, value: String) -> Result<(), String> {
        let mut store = self.store.write().map_err(|e| e.to_string())?;
        store.map.insert(key, value);
        self.persist(&store)
    }

    pub fn get(&self, key: String) -> Option<String> {
        let store = self.store.read().ok()?;
        store.map.get(&key).cloned()
    }

    pub fn remove(&self, key: String) -> Result<(), String> {
        let mut store = self.store.write().map_err(|e| e.to_string())?;
        if store.map.remove(&key).is_none() {
            return Err(format!("Key '{}' not found", key));
        }
        self.persist(&store)
    }

    fn persist(&self, store: &InnerStore) -> Result<(), String> {
        let data = serde_json::to_string_pretty(&store)
            .map_err(|e| format!("Failed to serialize store: {}", e))?;

        let mut file = File::create(STORE_FILE)
            .map_err(|e| format!("Failed to create store file: {}", e))?;
        file.write_all(data.as_bytes())
            .map_err(|e| format!("Failed to write to store file: {}", e))?;
        Ok(())
    }
}