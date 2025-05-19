mod kv_store;

use clap::{Parser, Subcommand};
use kv_store::KvStore;

#[derive(Parser)]
#[command(name = "kvstore")]
#[command(about = "A simple key-value store", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Set a key to a value
    Set { key: String, value: String },
    /// Get the value of a key
    Get { key: String },
    /// Remove a key
    Delete { key: String },
}

fn main() {
    let kv = match KvStore::new() {
            Ok(store) => store,
            Err(e) => {
                eprintln!("Failed to initialize KvStore: {}", e);
                return;
            }
        };
    // Check if only the program name is present (no CLI args)
    if std::env::args().len() == 1 {
        // Interactive shell mode
        use std::io::{self, Write};
        println!("kvstore interactive mode. Type 'help' for commands.");
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }
            let parts: Vec<_> = input.trim().split_whitespace().collect();
            match parts.as_slice() {
                ["set", key, value] => match kv.set(key.to_string(), value.to_string()) {
                    Ok(_) => println!("OK"),
                    Err(e) => println!("Error: {}", e),
                },
                ["get", key] => match kv.get(key.to_string()) {
                    Some(value) => println!("{}", value),
                    None => println!("Key not found"),
                },
                ["delete", key] => match kv.remove(key.to_string()) {
                    Ok(_) => println!("OK"),
                    Err(e) => println!("Error: {}", e),
                },
                ["exit"] | ["quit"] => break,
                ["help"] => println!("Commands: set <key> <value>, get <key>, delete <key>, exit"),
                _ => println!("Unknown command. Type 'help' for commands."),
            }
        }
    } else {
        // CLI mode (parse args with clap)
        let cli = Cli::parse();
        match cli.command {
            Commands::Set { key, value } => match kv.set(key, value) {
                Ok(_) => println!("OK"),
                Err(e) => eprintln!("Error: {}", e),
            },
            Commands::Get { key } => match kv.get(key) {
                Some(value) => println!("{}", value),
                None => println!("Key not found"),
            },
            Commands::Delete { key } => match kv.remove(key) {
                Ok(_) => println!("OK"),
                Err(e) => eprintln!("Error: {}", e),
            },
        }
    }
}
