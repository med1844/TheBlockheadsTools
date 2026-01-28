use lmdb_rs::env::{Env, EnvWrite};
use lmdb_rs::arch::DynArch;
use lmdb_rs::codec::types::{Str, Bytes};
use std::io::Cursor;

#[test]
fn test_write_and_read_api() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Write Phase
    let mut buffer = Vec::new();
    let writer = Cursor::new(&mut buffer);

    let mut env_write = EnvWrite::new(writer, DynArch::Arch64);
    let mut txn = env_write.write_txn()?;

    let db_main = txn.create_database::<Str, Bytes>(Some("main"))?;
    db_main.put(&mut txn, "key1", b"value1")?;
    db_main.put(&mut txn, "key2", b"value2")?;

    let db_blocks = txn.create_database::<Str, Bytes>(Some("blocks"))?;
    db_blocks.put(&mut txn, "block1", b"data1")?;

    txn.commit()?;

    // 2. Read Phase (Verify written data)
    let env_read = Env::new(&buffer)?;
    let rtxn = env_read.read_txn()?;

    // Verify Main DB
    let db_main_read = env_read.open_database::<Str, Bytes>(&rtxn, Some("main"))?
        .expect("Main DB should exist");
    
    assert_eq!(db_main_read.get(&rtxn, "key1")?, Some(b"value1".as_slice()));
    assert_eq!(db_main_read.get(&rtxn, "key2")?, Some(b"value2".as_slice()));


    // Verify Blocks DB
    let db_blocks_read = env_read.open_database::<Str, Bytes>(&rtxn, Some("blocks"))?
        .expect("Blocks DB should exist");
    
    assert_eq!(db_blocks_read.get(&rtxn, "block1")?, Some(b"data1".as_slice()));

    Ok(())
}
