use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

const STORE_FILE: &str = "store.json";
const WAL_FILE: &str = "store.log";

#[derive(Serialize, Deserialize)]
struct InnerStore {
    map: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
enum LogCommand {
    Set { key: String, value: String },
    Remove { key: String },
}

pub struct KvStore {
    store: Arc<RwLock<InnerStore>>,
}

impl KvStore {
    pub fn new() -> Result<Self, String> {
        let mut map = if Path::new(STORE_FILE).exists() {
            match fs::read_to_string(STORE_FILE) {
                Ok(data) => match serde_json::from_str(&data) {
                    Ok(map) => map,
                    Err(_) => {
                        eprintln!("Corrupt or invalid store file. Starting with empty store.");
                        InnerStore {
                            map: HashMap::new(),
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read store file: {}", e);
                    InnerStore {
                        map: HashMap::new(),
                    }
                }
            }
        } else {
            InnerStore {
                map: HashMap::new(),
            }
        };

        // Replay WAL
        if Path::new(WAL_FILE).exists() {
            let file = std::fs::File::open(WAL_FILE).map_err(|e| e.to_string())?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.map_err(|e| e.to_string())?;
                let cmd: LogCommand = serde_json::from_str(&line).map_err(|e| e.to_string())?;
                match cmd {
                    LogCommand::Set { key, value } => {
                        map.map.insert(key, value);
                    }
                    LogCommand::Remove { key } => {
                        map.map.remove(&key);
                    }
                }
            }
        }

        Ok(KvStore {
            store: Arc::new(RwLock::new(map)),
        })
    }

    pub fn set(&self, key: String, value: String) -> Result<(), String> {
        // Write to WAL
        let cmd = LogCommand::Set {
            key: key.clone(),
            value: value.clone(),
        };
        Self::append_to_wal(&cmd)?;

        // Apply to in-memory store
        let mut store = self.store.write().map_err(|e| e.to_string())?;
        store.map.insert(key, value);
        Ok(())
    }

    pub fn get(&self, key: String) -> Option<String> {
        let store = self.store.read().ok()?;
        store.map.get(&key).cloned()
    }

    pub fn remove(&self, key: String) -> Result<(), String> {
        // Write to WAL
        let cmd = LogCommand::Remove { key: key.clone() };
        Self::append_to_wal(&cmd)?;

        // Apply to in-memory store
        let mut store = self.store.write().map_err(|e| e.to_string())?;
        if store.map.remove(&key).is_none() {
            return Err(format!("Key '{}' not found", key));
        }
        Ok(())
    }

    // fn persist(&self, store: &InnerStore) -> Result<(), String> {
    //     let data = serde_json::to_string_pretty(&store)
    //         .map_err(|e| format!("Failed to serialize store: {}", e))?;

    //     let mut file =
    //         File::create(STORE_FILE).map_err(|e| format!("Failed to create store file: {}", e))?;
    //     file.write_all(data.as_bytes())
    //         .map_err(|e| format!("Failed to write to store file: {}", e))?;
    //     Ok(())
    // }

    fn append_to_wal(cmd: &LogCommand) -> Result<(), String> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(WAL_FILE)
            .map_err(|e| format!("Failed to open WAL file: {}", e))?;
        let line =
            serde_json::to_string(cmd).map_err(|e| format!("Failed to serialize WAL: {}", e))?;
        writeln!(file, "{}", line).map_err(|e| format!("Failed to write WAL: {}", e))?;
        Ok(())
    }

    /// Write a snapshot and clear the WAL
    pub fn compact(&self) -> Result<(), String> {
        // Take a write lock to block all other writes during compaction
        let store = self.store.write().map_err(|e| e.to_string())?;

        // Write snapshot
        let data = serde_json::to_string_pretty(&*store)
            .map_err(|e| format!("Failed to serialize store: {}", e))?;
        let mut file = File::create(STORE_FILE)
            .map_err(|e| format!("Failed to create store file: {}", e))?;
        file.write_all(data.as_bytes())
            .map_err(|e| format!("Failed to write to store file: {}", e))?;

        // Truncate WAL (safe, since no writes can happen during this lock)
        File::create(WAL_FILE).map_err(|e| format!("Failed to truncate WAL: {}", e))?;
        Ok(())
    }
}
