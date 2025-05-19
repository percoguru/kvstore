use std::sync::Arc;
use std::thread;
use kv_store::KvStore;

#[test]
fn concurrent_set_and_get() {
    // Create a shared KvStore instance wrapped in Arc for thread safety
    let store = Arc::new(KvStore::new().unwrap());

    let mut handles = vec![];

    // Spawn 10 threads, each setting and getting a unique key/value pair
    for i in 0..10 {
        let store = Arc::clone(&store);
        handles.push(thread::spawn(move || {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            // Set the key to the value
            store.set(key.clone(), value.clone()).unwrap();
            // Immediately get the value and check correctness
            assert_eq!(store.get(key.clone()), Some(value));
        }));
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // After all threads, check all keys are present and correct
    for i in 0..10 {
        let key = format!("key{}", i);
        let value = format!("value{}", i);
        assert_eq!(store.get(key), Some(value));
    }
}

#[test]
fn concurrent_writes_to_same_key() {
    // Create a shared KvStore instance
    let store = Arc::new(KvStore::new().unwrap());
    let key = "shared_key".to_string();
    // Prepare 10 different values to write to the same key
    let values: Vec<String> = (0..10).map(|i| format!("value{}", i)).collect();

    let mut handles = vec![];

    // Spawn 10 threads, each writing a different value to the same key
    for value in values.clone() {
        let store = Arc::clone(&store);
        let key = key.clone();
        handles.push(thread::spawn(move || {
            store.set(key, value).unwrap();
        }));
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // The final value of the key should be one of the values written by the threads
    let final_value = store.get("shared_key".to_string()).unwrap();
    println!("Final value for 'shared_key': {}", final_value);
    assert!(values.contains(&final_value));
}