use std::mem;
use the_blockheads_tools_lib::{
    BhResult,
    game::{chunk::Chunk, db::world_db::WorldDb},
};

fn main() -> BhResult<()> {
    dbg!(mem::size_of::<Chunk>());
    dbg!(mem::size_of::<Vec<u8>>());

    let args: Vec<String> = std::env::args().collect();
    dbg!(&args);
    assert!(args.len() == 2);

    let world_db = WorldDb::from_path(args.last().unwrap()).unwrap();
    for (k, v) in world_db.main.blockhead_inventories.iter() {
        println!("{} {:#}", k.inner(), v);
    }
    Ok(())
}
