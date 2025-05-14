use std::collections::HashMap;
use std::io::{self, Write};

/// A simple in-memory key-value store
struct KvStore {
    store: HashMap<String, String>,
}

impl KvStore {
    /// Create a new, empty KvStore
    fn new() -> Self {
        KvStore {
            store: HashMap::new(),
        }
    }

    /// Set a key to a value
    fn set(&mut self, key: String, value: String) {
        self.store.insert(key, value);
    }

    /// Get the value associated with a key
    fn get(&self, key: String) -> Option<&String> {
        self.store.get(&key)
    }
}

fn main() {
    let mut kv = KvStore::new(); // Create the key-value store

    println!("Minimal KV Store");
    println!("Usage: set <key> <value> | get <key> | exit");

    loop {
        // Print the prompt
        print!("> ");
        io::stdout().flush().unwrap();

        // Read input from stdin
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            println!("Failed to read input.");
            continue;
        }

        // Split the input into parts: command, key, value
        let parts: Vec<&str> = line.trim().splitn(3, ' ').collect();

        match parts.as_slice() {
            // Handle `set key value` command
            ["set", key, value] => {
                kv.set(key.to_string(), value.to_string());
                println!("OK");
            }

            // Handle `get key` command
            ["get", key] => match kv.get(key.to_string()) {
                Some(value) => println!("{}", value),
                None => println!("Key not found"),
            }

            // Handle `exit` command
            ["exit"] => break,

            // Handle unknown commands
            _ => println!("Unknown command"),
        }
    }
}
