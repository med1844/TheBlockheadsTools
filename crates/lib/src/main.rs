use std::collections::HashMap;
use the_blockheads_tools_lib::{
    BhResult, BlockCoord, BlockType, BlockView, BlockViewMut, Chunk, ChunkBlockCoord, WorldDb,
};

fn dump_to_stdout(bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    std::io::stdout().write_all(bytes)?;
    std::io::stdout().flush()?; // Make sure all bytes are written immediately
    Ok(())
}

fn main() -> BhResult<()> {
    let mut world_db = WorldDb::from_path(
        "/home/med/GNUstep/Library/ApplicationSupport/TheBlockheads/saves/3d7_modified/world_db",
    )?;
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

    let world_v2 = &mut world_db.main.world_v2;
    let x = world_v2.start_portal_pos_x;
    let y = world_v2.start_portal_pos_y;
    world_v2.world_name = "MOD_2".to_string();
    world_v2.save_id = "3d7_modified_2".to_string();

    let start_portal_pos = BlockCoord::new(x as u32, (y - 1) as u16)?;
    let chunk = world_db.blocks.chunk_at_mut(start_portal_pos).unwrap()?;

    for x in 0..32 {
        for y in 0..32 {
            let mut block = chunk.block_at_mut(ChunkBlockCoord::new(x, y).unwrap());
            if block.fg()? == BlockType::Dirt {
                block.set_fg(BlockType::TimeCrystal);
            }
        }
    }

    std::fs::create_dir_all(
        "/home/med/GNUstep/Library/ApplicationSupport/TheBlockheads/saves/3d7_modified_2/world_db",
    )
    .unwrap();

    world_db.to_path(
        "/home/med/GNUstep/Library/ApplicationSupport/TheBlockheads/saves/3d7_modified_2/world_db",
    )

    // let mut keys = world_db.dw.keys().collect::<Vec<_>>();
    // keys.sort_unstable();
    // dbg!(keys);
    // dbg!(dump_to_stdout(
    //     world_db.dw.get("335_16/14").unwrap().as_slice()
    // ));
    // Ok(())
}
