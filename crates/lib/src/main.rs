use std::collections::HashMap;
use the_blockheads_tools_lib::{
    BhResult, BlockCoord, BlockType, BlockView, Chunk, ChunkBlockCoord, WorldDb,
};

fn dump_to_stdout(bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    std::io::stdout().write_all(bytes)?;
    std::io::stdout().flush()?; // Make sure all bytes are written immediately
    Ok(())
}

fn main() -> BhResult<()> {
    let world_db = WorldDb::from_path(
        "/home/med/GNUstep/Library/ApplicationSupport/TheBlockheads/saves/3d7_modified/world_db",
    )?;
    if let Some(mut world_db) = world_db {
        // dump_to_stdout(
        //     world_db
        //         .main
        //         .world_v2
        //         .circum_navigate_booleans_data
        //         .as_ref(),
        // )
        // .unwrap();
        // let a = plist::from_bytes::<plist::Value>(world_db.main.world_v2.found_items.as_ref());
        // dbg!(a);
        // dbg!(world_db.main.world_v2);
        // dump_to_stdout(.as_ref());
        // dbg!(world_db.blocks.keys().collect::<Vec<_>>());
        // world_db.blocks.at_mut(coord)
        // let world_v2 = &mut world_db.main.world_v2;
        // let x = world_db.main.world_v2.start_portal_pos_x;
        // let y = world_db.main.world_v2.start_portal_pos_y;
        // dbg!(x, y);
        // let start_portal_pos = BlockCoord::new(x, (y - 1) as u16)?;
        // let block = world_db.blocks.block_at(start_portal_pos).unwrap()?;
        // let block_type = block.fg()?;
        // dbg!(block_type.as_str());
        // let chunk = world_db.blocks.chunk_at(start_portal_pos).unwrap()?;
        // println!("{}", chunk.display_chunk_by_fg());
        // let chunk = world_db.blocks.chunk_at().unwrap()?;
        // chunk.block_at(&ChunkBlockCoord::new(x & 31, y & 31));
        // dump_to_stdout();
        let keys = world_db.dw.keys().collect::<Vec<_>>();
        dbg!(keys);
    }
    Ok(())
}
