use divan::Bencher;
use lmdb_rs::arch::Arch64;
use lmdb_rs::codec::types::{Bytes, Str};
use lmdb_rs::env::{Env, EnvWrite};
use std::env;
use std::fs;
use std::path::PathBuf;
// Arch trait might be needed for as_dyn_arch, but Arch64 struct usually implements it.
// We need to import the trait to call as_dyn_arch if it is a trait method.
use lmdb_rs::arch::Arch;

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    if env::var("LMDB_BENCH_PATH").is_err() {
        eprintln!("WARNING: LMDB_BENCH_PATH not set. Benchmarks will be skipped or fail.");
    }
    divan::main();
}

fn load_data() -> Vec<u8> {
    let path_str = env::var("LMDB_BENCH_PATH").expect("LMDB_BENCH_PATH must be set");
    let path = PathBuf::from(path_str);
    fs::read(&path).expect("Failed to read DB file")
}

#[divan::bench]
fn bench_env_open(bencher: Bencher) {
    let data = load_data();
    bencher.bench_local(|| {
        divan::black_box(Env::new(&data).unwrap());
    });
}

#[divan::bench]
fn bench_read_txn(bencher: Bencher) {
    let data = load_data();
    let env = Env::new(&data).unwrap();

    bencher.bench_local(|| {
        divan::black_box(env.read_txn().unwrap());
    });
}

#[divan::bench]
fn bench_iter_main_db(bencher: Bencher) {
    let data = load_data();
    let env = Env::new(&data).unwrap();

    bencher.bench_local(|| {
        let txn = env.read_txn().unwrap();
        // Main DB is usually nameless (None) or "main"? Example says None implies main db/root?
        // lmdb_copy: env.open_database::<Str, Bytes>(&txn, None)
        let main_db = env
            .open_database::<Str, Bytes>(&txn, None)
            .unwrap()
            .expect("Root DB not found");

        let mut count = 0;
        for item in main_db.iter(&txn).unwrap() {
            let _ = divan::black_box(item);
            count += 1;
        }
        divan::black_box(count);
    });
}

#[divan::bench]
fn bench_read_all_entries(bencher: Bencher) {
    let data = load_data();
    let env = Env::new(&data).unwrap();

    bencher.bench_local(|| {
        let txn = env.read_txn().unwrap();

        // 1. Find DB names
        let main_db = env
            .open_database::<Str, Bytes>(&txn, None)
            .unwrap()
            .expect("Root DB not found");
        let mut db_names = Vec::new();
        for item in main_db.iter(&txn).unwrap() {
            let (key, _) = item.unwrap();
            db_names.push(key.to_string());
        }

        // 2. Iterate all Sub DBs
        for name in &db_names {
            let db = env
                .open_database::<Bytes, Bytes>(&txn, Some(name))
                .unwrap()
                .expect("Sub-DB missing");
            for item in db.iter(&txn).unwrap() {
                let _ = divan::black_box(item.unwrap());
            }
        }
    });
}

#[divan::bench]
fn bench_write_full_copy(bencher: Bencher) {
    let data = load_data();
    let env = Env::new(&data).unwrap();

    // Pre-read logic to have data ready for write benchmark
    // We don't want to measure read time in the write bench if we can avoid it,
    // BUT usually benchmarks run the whole loop.
    // Ideally we prepare the data "outside" the bench loop.

    let mut collected_dbs: Vec<(String, Vec<(Vec<u8>, Vec<u8>)>)> = Vec::new();
    {
        let txn = env.read_txn().unwrap();
        let main_db = env
            .open_database::<Str, Bytes>(&txn, None)
            .unwrap()
            .expect("Root DB not found");
        let mut db_names = Vec::new();
        for item in main_db.iter(&txn).unwrap() {
            let (key, _) = item.unwrap();
            db_names.push(key.to_string());
        }
        for name in &db_names {
            let db = env
                .open_database::<Bytes, Bytes>(&txn, Some(name))
                .unwrap()
                .expect("Sub-DB missing");
            let mut entries = Vec::new();
            for item in db.iter(&txn).unwrap() {
                let (k, v) = item.unwrap();
                entries.push((k.to_vec(), v.to_vec()));
            }
            collected_dbs.push((name.clone(), entries));
        }
    }

    // Now benchmark the WRITE process
    bencher.bench_local(|| {
        let mut writer = Vec::new(); // Memory writer
        // Use Arch64 for benchmark (assuming creating 64-bit DB)
        // EnvWrite::new(writer, arch)
        let mut env_write = EnvWrite::new(&mut writer, Arch64::as_dyn_arch());
        let mut txn = env_write.write_txn().unwrap();

        for (name, entries) in &collected_dbs {
            let db = txn.create_database::<Bytes, Bytes>(Some(name)).unwrap();
            for (k, v) in entries {
                db.put(&mut txn, &k.as_slice(), &v.as_slice()).unwrap();
            }
        }
        txn.commit().unwrap();
        divan::black_box(writer);
    });
}
