use the_blockheads_tools_lib::{BhResult, game::db::world_db::WorldDb};

fn dump_to_stdout(bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    std::io::stdout().write_all(bytes)?;
    std::io::stdout().flush()?; // Make sure all bytes are written immediately
    Ok(())
}

fn main() -> BhResult<()> {
    let world_db = WorldDb::from_path(
        "/home/med/GNUstep/Library/ApplicationSupport/TheBlockheads/saves/3d7_mod2/world_db",
    )?;

    let chunk_coord = "335_16";
    let mut keys = world_db
        .dw
        .keys()
        .filter(|s| s.starts_with(chunk_coord))
        .collect::<Vec<_>>();
    keys.sort_unstable();
    dbg!(keys);
    let data = world_db.dw.get(format!("{}/18", chunk_coord).as_str());
    if let Some(data) = data {
        dump_to_stdout(data.as_slice()).unwrap();
    }
    Ok(())
}
