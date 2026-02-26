use lmdb_rs::env::Env;
use std::fs;
use the_blockheads_tools_lib::BhResult;

fn main() -> BhResult<()> {
    let args: Vec<String> = std::env::args().collect();
    assert!(args.len() == 2);

    let db_path = args.last().unwrap();
    let db_data = fs::read(std::path::Path::new(db_path).join("data.mdb")).unwrap();
    let env = Env::new(&db_data).unwrap();
    let rtxn = env.read_txn().unwrap();
    let dw_db = env
        .open_database::<lmdb_rs::codec::types::Str, lmdb_rs::codec::types::Bytes>(
            &rtxn,
            Some("dw"),
        )
        .unwrap()
        .unwrap();

    // let mut found = [false; 64];
    let mut found = [
        false, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
        false, false, true, true, false, false, false, false, false, false, true, false, true,
        true, false, false, false, false, true, true, true, true, true, false, true, false, false,
        false, false, false, false, false, false, false, false, false, true, false, false, false,
        false, false, true, true, true, false, true, true, true,
    ];
    fs::create_dir_all("test_data").unwrap();

    for kv in dw_db.iter(&rtxn).unwrap() {
        if let Ok((k, v)) = kv
            && let Some((chunk_coord, type_id_str)) = k.split_once("/")
            && chunk_coord == "349_20"
            && let Ok(type_id) = type_id_str.parse::<usize>()
            && type_id < 64
            && !found[type_id]
            && let Ok(dict) = plist::from_bytes::<plist::Dictionary>(v)
            && let Some(dyn_objs) = dict.get("dynamicObjects").and_then(|v| v.as_array())
            && !dyn_objs.is_empty()
        {
            found[type_id] = true;
            let filename = format!("test_data/type_{}.xml", type_id);
            fs::write(&filename, v).unwrap();
            println!("Generated {}", filename);
        }
    }
    Ok(())
}
