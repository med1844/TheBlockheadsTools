use lmdb_rs::codec::types::{Bytes, Str};
use lmdb_rs::env::Env;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_safe_api_read_dw() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("resources/3d7_modified/world_db/data.mdb");

    if !path.exists() {
        // Skip if resource not present (ci friendliness)
        return;
    }

    let data = fs::read(&path).expect("Failed to read DB");
    println!("Read {} bytes from {:?}", data.len(), path);

    // 1. Open Environment
    let env = Env::new(&data).expect("Failed to open Env");
    println!("Env opened. Page size: {}", env.page_size());

    // 2. Create Transaction
    let txn = env.read_txn().expect("Failed to create txn");

    // 3. Open Database "dw"
    let db = env
        .open_database::<Str, Bytes>(&txn, Some("dw"))
        .expect("Failed to open DB")
        .expect("Database 'dw' not found");

    println!("Opened 'dw' database. Validating entries...");

    // 4. Iterate and Capture a Key
    let mut count = 0;
    let mut first_key: Option<String> = None;

    for item in db.iter(&txn).expect("Failed to iterate") {
        let (key, val) = item.expect("Failed to read item");
        if count == 0 {
            first_key = Some(key.to_string());
        }
        count += 1;
        if count <= 5 {
            println!("Key: {:?}, Val Len: {}", key, val.len());
        }
    }

    assert_eq!(count, 3025, "Entry count mismatch for dw");

    // 5. Test Typed Get
    if let Some(key) = first_key {
        println!("Testing get with key: {}", key);
        let val = db
            .get(&txn, &key)
            .expect("Failed to get")
            .expect("Key not found");
        println!("Found value of size: {}", val.len());
    } else {
        panic!("No keys found in dw?");
    }

    // 6. Test Typed Get (Missing)
    let missing = db.get(&txn, "non_existent_key_12345").expect("db error");
    assert!(missing.is_none());
}
