use clap::Parser;
use lmdb_rs::{
    codec::types::{Bytes, Str},
    env::Env,
};
use memmap2::MmapOptions;
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the original (larger) LMDB data file
    #[arg(value_name = "OLD_FILE")]
    old_path: PathBuf,

    /// Path to the new (smaller) LMDB data file
    #[arg(value_name = "NEW_FILE")]
    new_path: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Comparing:");
    println!("  Old: {:?}", args.old_path);
    println!("  New: {:?}", args.new_path);

    let old_file = File::open(&args.old_path)?;
    let old_mmap = unsafe { MmapOptions::new().map(&old_file)? };
    let old_env = Env::new(&old_mmap)?;

    let new_file = File::open(&args.new_path)?;
    let new_mmap = unsafe { MmapOptions::new().map(&new_file)? };
    let new_env = Env::new(&new_mmap)?;

    let old_txn = old_env.read_txn()?;
    let new_txn = new_env.read_txn()?;

    let db_names = vec!["blocks", "dw", "main"];

    for name in db_names {
        println!("\n--- Database: {} ---", name);

        // Open in Old
        let old_db = match old_env.open_database::<Str, Bytes>(&old_txn, Some(name))? {
            Some(db) => db,
            None => {
                println!(
                    "  [WARN] Database '{}' missing in OLD file. Skipping.",
                    name
                );
                continue;
            }
        };

        // Open in New
        let new_db = match new_env.open_database::<Str, Bytes>(&new_txn, Some(name))? {
            Some(db) => db,
            None => {
                println!(
                    "  [CRITICAL] Database '{}' completely MISSING in NEW file!",
                    name
                );
                continue;
            }
        };

        let mut missing_keys_count = 0;
        let mut missing_bytes = 0usize;
        let mut changed_value_count = 0;
        let mut value_size_delta: i64 = 0;
        let mut inspected_keys = 0;

        // Iterate Old items
        let iter = old_db.iter(&old_txn)?;
        for item in iter {
            let (key, old_val) = item?;
            inspected_keys += 1;

            match new_db.get(&new_txn, key) {
                Ok(Some(new_val)) => {
                    // Key exists, check value
                    if old_val.len() != new_val.len() {
                        changed_value_count += 1;
                        let diff = (new_val.len() as i64) - (old_val.len() as i64);
                        value_size_delta += diff;

                        // Verbose detail for significant changes (optional, maybe flag gated?)
                        // For now just aggregate
                    } else if old_val != new_val {
                        // Same length, different content (unlikely to affect size, but good to know)
                        // not tracking separately for size reduction task
                    }
                }
                Ok(None) => {
                    // Key missing in New
                    missing_keys_count += 1;
                    missing_bytes += old_val.len();
                    // println!("  [MISSING] Key: '{}' (Lost {} bytes)", key, old_val.len());
                    // Too spammy if 500kb is lost in small chunks.
                    // Maybe print first few?
                    if missing_keys_count <= 10 {
                        println!("  [MISSING] Key: '{}' (Lost {} bytes)", key, old_val.len());
                    }
                }
                Err(e) => {
                    println!("  [ERROR] checking key '{}': {}", key, e);
                }
            }
        }

        println!("  Summary for '{}':", name);
        println!("    Total Keys in Old: {}", inspected_keys);
        println!("    Missing Keys:      {}", missing_keys_count);
        println!(
            "    Missing Bytes:     {} (from missing keys)",
            missing_bytes
        );
        println!("    Changed Values:    {}", changed_value_count);
        println!(
            "    Size Delta:        {} bytes (from changed values)",
            value_size_delta
        );

        let net_change = (value_size_delta) - (missing_bytes as i64);
        println!("    NET CHANGE:        {} bytes", net_change);
    }

    Ok(())
}
