use lmdb_rs::arch::{Arch, Arch32, Arch64};
use lmdb_rs::codec::types::{Bytes, Str};
use lmdb_rs::env::{Env, EnvWrite};

use std::env;
use std::fs::{self, File};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Usage: {} <input_file> <output_file> [arch: 64|32]",
            args[0]
        );
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let arch_arg = args.get(3).map(|s| s.as_str()).unwrap_or("64");

    println!(
        "Copying {} -> {} (Arch{})",
        input_path, output_path, arch_arg
    );

    // 1. Read Input
    let input_data = fs::read(input_path)?;
    let env = Env::new(&input_data)?;
    let txn = env.read_txn()?;

    // Collect all data
    // Map: DB Name -> Entries
    let mut collected_dbs: Vec<(String, Vec<(Vec<u8>, Vec<u8>)>)> = Vec::new();

    // 1.1 Open Main DB to find other databases
    // Note: In LMDB, the root DB (dbi=0) contains records for other Named DBs.
    // We iterate it to find names.
    // BUT we also want to copy keys in Main DB itself if they are NOT db records?
    // Usually Main DB works as a directory of DBs.

    // Let's assume we iterate Main DB to find names.
    let main_db = env
        .open_database::<Str, Bytes>(&txn, None)?
        .expect("Root DB not found?");

    let mut db_names = Vec::new();

    for item in main_db.iter(&txn)? {
        let (key, _val) = item?;
        // In Main DB, keys are DB names.
        // Value is `DbRecord`.
        // We just need names to open them cleanly.
        db_names.push(key.to_string());
    }

    // Sort names for deterministic order check
    db_names.sort();

    println!("Found {} sub-databases: {:?}", db_names.len(), db_names);

    for name in &db_names {
        let db = env
            .open_database::<Bytes, Bytes>(&txn, Some(name))?
            .expect(&format!("Sub-DB {} missing?", name));

        let mut entries = Vec::new();
        for item in db.iter(&txn)? {
            let (k, v) = item?;
            entries.push((k.to_vec(), v.to_vec()));
        }
        collected_dbs.push((name.clone(), entries));
        println!(
            " - {}: {} entries",
            name,
            collected_dbs.last().unwrap().1.len()
        );
    }

    // 2. Build Output
    let file = File::create(output_path)?;
    let arch_arg = args.get(3).map(|s| s.as_str()).unwrap_or("64");
    match arch_arg {
        "32" => write_copy::<Arch32>(file, &collected_dbs)?,
        _ => write_copy::<Arch64>(file, &collected_dbs)?,
    };

    println!("Write complete. Validating...");

    // 3. Validation
    validate_copy(input_path, output_path, &collected_dbs)?;

    println!("SUCCESS: Output matches Input 100%.");
    Ok(())
}

fn write_copy<A: Arch>(
    writer: File,
    dbs: &[(String, Vec<(Vec<u8>, Vec<u8>)>)],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut env = EnvWrite::new(writer, A::as_dyn_arch());
    let mut txn = env.write_txn()?;

    for (name, entries) in dbs {
        let db = txn.create_database::<Bytes, Bytes>(Some(name))?;
        for (k, v) in entries {
            db.put(&mut txn, &k.as_slice(), &v.as_slice())?;
        }
    }

    txn.commit()?;
    Ok(())
}

fn validate_copy(
    _input_path: &str,
    output_path: &str,
    original_dbs: &[(String, Vec<(Vec<u8>, Vec<u8>)>)],
) -> Result<(), Box<dyn std::error::Error>> {
    let output_data = fs::read(output_path)?;
    let env_out = Env::new(&output_data)?;
    let txn_out = env_out.read_txn()?;

    for (name, original_entries) in original_dbs {
        let db_out = env_out
            .open_database::<Bytes, Bytes>(&txn_out, Some(name))?
            .expect(&format!("Output missing DB: {}", name));

        let out_iter = db_out.iter(&txn_out)?;
        let mut idx = 0;

        for item in out_iter {
            let (k_out, v_out) = item?;
            if idx >= original_entries.len() {
                return Err(format!("Output has extra entries for {}", name).into());
            }

            let (k_in, v_in) = &original_entries[idx];

            if k_out != k_in {
                return Err(format!(
                    "Mismatch key at idx {} in {}: {:?} vs {:?}",
                    idx, name, k_out, k_in
                )
                .into());
            }
            if v_out != v_in {
                return Err(format!(
                    "Mismatch val at idx {} in {}: {:?} vs {:?}",
                    idx, name, v_out, v_in
                )
                .into());
            }
            idx += 1;
        }

        if idx < original_entries.len() {
            return Err(format!(
                "Output missing entries for {} (found {}, expected {})",
                name,
                idx,
                original_entries.len()
            )
            .into());
        }
    }

    Ok(())
}
