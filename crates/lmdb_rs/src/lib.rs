//! # LMDB-RS
//! 
//! A Rust implementation of the LMDB (Lightning Memory-Mapped Database) file format. 
//! This library provides tools for inspecting and parsing LMDB files in a read-only manner,
//! without linking properly to the C library.
//!
//! ## Key Types
//! 
//! - [`env::Env`]: The main entry point, representing a database environment.
//! - [`txn::RoTxn`]: A read-only transaction.
//! - [`database::Database`]: A typed handle to a named sub-database.
//! 
//! ## Example: Reading from a Database (Safe API)
//! 
//! ```no_run
//! use std::fs;
//! use lmdb_rs::env::Env;
//! use lmdb_rs::codec::types::{Str, Bytes};
//! 
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let data = fs::read("data.mdb")?;
//!     let env = Env::new(&data)?;
//! 
//!     // 1. Create Transaction
//!     let txn = env.read_txn()?;
//! 
//!     // 2. Open a Named Database (e.g., "users") expecting String keys and Byte values
//!     if let Some(db) = env.open_database::<Str, Bytes>(&txn, Some("users"))? {
//!         // 3. Iterate
//!         for item in db.iter(&txn)? {
//!             let (key, val) = item?; // key: &str, val: &[u8]
//!             println!("User: {}, Data: {} bytes", key, val.len());
//!         }
//!         
//!         // 4. Get specific value
//!         if let Some(val) = db.get(&txn, "alice")? {
//!              println!("Found Alice: {:?}", val);
//!         }
//!     }
//! 
//!     Ok(())
//! }
//! ```
//! 
//! ## Legacy/Internal Types
//! 
//! If you need low-level access to pages or cursors:
//! - [`page::generic::Page`]
//! - [`cursor::Cursor`]
//!

pub mod arch;
pub mod codec;
pub mod constants;

pub mod cursor;
pub mod db_record;
pub mod database; // Will be the new Typed DB file
pub mod env;
pub mod error;
pub mod page;
pub mod txn;
pub mod write;
pub mod build;

