use lmdb_rs::{
    codec::types::{Bytes, Str},
    env::Env,
};
use snafu::Whatever;
use std::fs;

fn main() -> Result<(), Whatever> {
    let args: Vec<String> = std::env::args().collect();
    assert!(args.len() == 2);

    let db_path = args.last().unwrap();
    let db_data = fs::read(std::path::Path::new(db_path).join("data.mdb")).unwrap();
    let env = Env::new(&db_data).unwrap();
    let rtxn = env.read_txn().unwrap();
    // let dw_db = env
    //     .open_database::<Str, Bytes>(&rtxn, Some("dw"))
    //     .unwrap()
    //     .unwrap();
    let main_db = env
        .open_database::<Str, Bytes>(&rtxn, Some("main"))
        .unwrap()
        .unwrap();
    for kv in main_db.iter(&rtxn).unwrap() {
        if let Ok((k, v)) = kv
            && k.starts_with("blockhead")
        {
            println!("{}", k);
            println!("{}", str::from_utf8(v).unwrap());
        }
    }

    // let mut found = [
    //     false, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    //     false, false, true, true, false, false, false, false, false, false, true, false, true,
    //     true, false, false, false, false, true, true, true, true, true, false, true, false, false,
    //     false, false, false, false, false, false, false, false, false, true, false, false, false,
    //     false, false, true, true, true, false, true, true, true,
    // ];
    fs::create_dir_all("test_data").unwrap();

    // for kv in dw_db.iter(&rtxn).unwrap() {
    //     if let Ok((k, v)) = kv
    //         && k.starts_with("430_16/chest_")
    //         && let Some((_, chest_id)) = k.split_once("/")
    //     {
    //         let filename = format!("test_data/{}.xml", chest_id);
    //         fs::write(&filename, v).unwrap();
    //     }
    //     // if let Ok((k, v)) = kv
    //     //     && k.starts_with("trainchest_")
    //     // {
    //     //     let filename = format!("test_data/{}.xml", k);
    //     //     fs::write(&filename, v).unwrap();
    //     // }
    //     // if let Ok((k, v)) = kv
    //     //     && let Some((chunk_coord, type_id_str)) = k.split_once("/")
    //     //     && chunk_coord == "430_16"
    //     //     && let Ok(type_id) = type_id_str.parse::<usize>()
    //     //     && type_id < 64
    //     //     && type_id == 17
    //     //     && let Ok(dict) = plist::from_bytes::<plist::Dictionary>(v)
    //     //     && let Some(dyn_objs) = dict.get("dynamicObjects").and_then(|v| v.as_array())
    //     //     && !dyn_objs.is_empty()
    //     // {
    //     //     found[type_id] = true;
    //     //     let filename = format!("test_data/type_{}.xml", type_id);
    //     //     fs::write(&filename, v).unwrap();
    //     //     println!("Generated {}", filename);
    //     // }
    // }
    Ok(())
}
