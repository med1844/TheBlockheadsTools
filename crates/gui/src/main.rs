use std::collections::HashMap;

use macroquad::prelude::*;
use the_blockheads_tools_lib::{
    BhResult, BlockCoord, BlockType, BlockView, Chunk, ChunkBlockCoord, WorldDb,
};

// fn dump_to_stdout(bytes: &[u8]) -> std::io::Result<()> {
//     use std::io::Write;
//     std::io::stdout().write_all(bytes)?;
//     std::io::stdout().flush()?; // Make sure all bytes are written immediately
//     Ok(())
// }

// fn main_old() -> BhResult<()> {
//     let world_db = WorldDb::from_path("test_data/saves/3d716d9bbf89c77ef5001e9cd227ec29/world_db")?;
//     if let Some(mut world_db) = world_db {
//         // dump_to_stdout(
//         //     world_db
//         //         .main
//         //         .world_v2
//         //         .circum_navigate_booleans_data
//         //         .as_ref(),
//         // )
//         // .unwrap();
//         // let a = plist::from_bytes::<plist::Value>(world_db.main.world_v2.found_items.as_ref());
//         // dbg!(a);
//         // dbg!(world_db.main.world_v2);
//         // dump_to_stdout(.as_ref());
//         // dbg!(world_db.blocks.keys().collect::<Vec<_>>());
//         // world_db.blocks.at_mut(coord)
//         // let world_v2 = &mut world_db.main.world_v2;
//         let x = world_db.main.world_v2.start_portal_pos_x;
//         let y = world_db.main.world_v2.start_portal_pos_y;
//         dbg!(x, y);
//         let start_portal_pos = BlockCoord::new(x, (y - 1) as u16)?;
//         let block = world_db.blocks.block_at(start_portal_pos).unwrap()?;
//         let block_type = block.fg()?;
//         dbg!(block_type.as_str());
//         let chunk = world_db.blocks.chunk_at(start_portal_pos).unwrap()?;
//         println!("{}", chunk.display_chunk_by_fg());
//         // let chunk = world_db.blocks.chunk_at().unwrap()?;
//         // chunk.block_at(&ChunkBlockCoord::new(x & 31, y & 31));
//     }
//     Ok(())
// }

fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: Conf {
            window_title: "egui with macroquad".to_owned(),
            high_dpi: true,
            ..Default::default()
        },
        update_on: None,
        default_filter_mode: FilterMode::Linear,
        draw_call_vertex_capacity: 1 << 18,
        draw_call_index_capacity: 1 << 20,
    }
}

fn draw_primitives() {
    // draw_cube(
    //     Vec3::default(),
    //     Vec3::ONE,
    //     None,
    //     Color::new(1.0, 0.0, 0.0, 0.5),
    // );
    draw_line_3d(
        Vec3::new(0., 0., 0.),
        Vec3::new(1., 0., 0.),
        Color::from_hex(0xFF0000),
    );
    draw_line_3d(
        Vec3::new(0., 0., 0.),
        Vec3::new(0., 1., 0.),
        Color::from_hex(0x00FF00),
    );
    draw_line_3d(
        Vec3::new(0., 0., 0.),
        Vec3::new(0., 0., 1.),
        Color::from_hex(0x0000FF),
    );
}

struct Pseudo2DCam {
    previous_mouse: Vec2,
    camera: Camera3D,
    distance: f32,
}

impl Pseudo2DCam {
    fn default() -> Self {
        let distance = -10.;
        Self {
            previous_mouse: Vec2::default(),
            camera: Camera3D {
                position: Vec3::new(0., distance, 0.),
                ..Camera3D::default()
            },
            distance,
        }
    }

    fn process_mouse_offset(&mut self, x: f32, y: f32) {
        self.camera.position.x -= x;
        self.camera.position.z += y;

        self.camera.target.x -= x;
        self.camera.target.z += y;
    }

    fn process_mouse_and_keys(&mut self) {
        let mouse_pos: Vec2 = glam::Vec2::from(<[f32; 2]>::from(mouse_position_local()));
        if is_mouse_button_down(MouseButton::Left) {
            self.process_mouse_offset(
                (mouse_pos.x - self.previous_mouse.x) * self.distance.abs(),
                (mouse_pos.y - self.previous_mouse.y) * self.distance.abs(),
            )
        }
        let (_, y) = mouse_wheel();
        self.distance *= 1.0 - y * 0.1;
        self.camera.position.y = self.distance;

        self.previous_mouse = mouse_pos;
    }
}

// We build a mesh for each of the render group other than transparent.
#[derive(Debug, PartialEq, Eq, Hash)]
enum BlockRenderGroup {
    Transparent,
    Semi(BlockType),
    Solid,
}

impl From<BhResult<BlockType>> for BlockRenderGroup {
    fn from(value: BhResult<BlockType>) -> Self {
        match value {
            Ok(value) => match value {
                BlockType::Air => Self::Transparent,
                block_type @ (BlockType::Water
                | BlockType::Ice
                | BlockType::Snow
                | BlockType::BlackGlass
                | BlockType::AmethystBlock
                | BlockType::SapphireBlock
                | BlockType::EmeraldBlock
                | BlockType::RubyBlock
                | BlockType::DiamondBlock) => Self::Semi(block_type),
                _ => Self::Solid,
            },
            Err(_) => Self::Solid,
        }
    }
}

struct MeshBuilder {
    map: HashMap<BlockRenderGroup, Vec<Mesh>>,
}

impl MeshBuilder {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    // Y   Z
    // |  /
    // | /
    // |/
    // ------X
    fn add_block(&mut self, x: f32, y: f32, z: f32, color: Color, group: BlockRenderGroup) {
        if group != BlockRenderGroup::Transparent {
            let x0 = x;
            let y0 = y;
            let z0 = z;
            let x1 = x + 1.0;
            let y1 = y + 1.0;
            let z1 = z + 1.0;

            // The 8 vertices of the cube
            let vertices = vec![
                // Front face (Z-)
                Vertex::new(x0, y0, z0, 0.0, 0.0, color), // 0
                Vertex::new(x1, y0, z0, 1.0, 0.0, color), // 1
                Vertex::new(x1, y1, z0, 1.0, 1.0, color), // 2
                Vertex::new(x0, y1, z0, 0.0, 1.0, color), // 3
                // Back face (Z+)
                Vertex::new(x0, y0, z1, 0.0, 0.0, color), // 4
                Vertex::new(x1, y0, z1, 1.0, 0.0, color), // 5
                Vertex::new(x1, y1, z1, 1.0, 1.0, color), // 6
                Vertex::new(x0, y1, z1, 0.0, 1.0, color), // 7
            ];

            // 12 triangles (2 per face) for a total of 36 indices.
            let indices = vec![
                // Front face (Z-)
                0, 1, 2, 0, 2, 3, // Right face (X+)
                1, 5, 6, 1, 6, 2, // Back face (Z+)
                5, 4, 7, 5, 7, 6, // Left face (X-)
                4, 0, 3, 4, 3, 7, // Bottom face (Y-)
                4, 5, 1, 4, 1, 0, // Top face (Y+)
                3, 2, 6, 3, 6, 7,
            ];

            let block_mesh = Mesh {
                vertices,
                indices,
                texture: None,
            };

            self.map
                .entry(group)
                .or_insert_with(|| vec![])
                .push(block_mesh);
        }
    }

    fn aggregate_mesh(meshes: Vec<Mesh>) -> Mesh {
        let mut vertices =
            Vec::with_capacity(meshes.iter().map(|mesh| mesh.vertices.len()).sum::<usize>());
        let mut indices =
            Vec::with_capacity(meshes.iter().map(|mesh| mesh.indices.len()).sum::<usize>());
        let mut accu_vertices = 0;
        for mut mesh in meshes {
            indices.extend(mesh.indices.iter().copied().map(|i| i + accu_vertices));
            accu_vertices += mesh.vertices.len() as u16;
            vertices.append(&mut mesh.vertices);
        }
        Mesh {
            vertices,
            indices,
            texture: None,
        }
    }

    fn build(self) -> Mesh {
        let mut group_meshes = Vec::with_capacity(self.map.keys().len());
        for (_, value) in self.map {
            group_meshes.push(Self::aggregate_mesh(value));
        }
        Self::aggregate_mesh(group_meshes)
    }
}

fn chunk_to_mesh(chunk: &Chunk) -> Mesh {
    let mut builder = MeshBuilder::new();
    for y in 0..32 {
        for x in 0..32 {
            let block = chunk.block_at(ChunkBlockCoord::new(x, y).expect("Must be valid"));
            builder.add_block(
                x as f32,
                0.,
                y as f32,
                Color::from_rgba(255, 0, 0, 128),
                block.fg().into(),
            );
            builder.add_block(
                x as f32,
                1.,
                y as f32,
                Color::from_rgba(0, 255, 0, 128),
                block.fg().into(),
            );
            builder.add_block(
                x as f32,
                2.,
                y as f32,
                Color::from_rgba(0, 0, 255, 128),
                block.bg().into(),
            );
        }
    }
    builder.build()
}

#[macroquad::main(window_conf)]
async fn main() -> BhResult<()> {
    let mut world_db =
        WorldDb::from_path("test_data/saves/3d716d9bbf89c77ef5001e9cd227ec29/world_db")?.unwrap();
    let x = world_db.main.world_v2.start_portal_pos_x;
    let y = world_db.main.world_v2.start_portal_pos_y;
    let start_portal_pos = BlockCoord::new(x, (y - 1) as u16)?;
    let chunk = world_db.blocks.chunk_at(start_portal_pos).unwrap()?;
    // println!("{}", chunk.display_chunk_by_fg());
    let mesh = chunk_to_mesh(chunk);

    let mut pixels_per_point: Option<f32> = None;
    let mut cam = Pseudo2DCam::default();

    loop {
        clear_background(WHITE);

        egui_macroquad::ui(|egui_ctx| {
            if pixels_per_point.is_none() {
                pixels_per_point = Some(egui_ctx.pixels_per_point());
            }

            egui::Window::new("egui ❤  macroquad").show(egui_ctx, |ui| {
                let response = ui.add(
                    egui::Slider::new(pixels_per_point.as_mut().unwrap(), 0.75..=3.0)
                        .logarithmic(true),
                );

                // Don't change scale while dragging the slider
                if response.drag_stopped() {
                    egui_ctx.set_pixels_per_point(pixels_per_point.unwrap());
                }
            });
        });

        cam.process_mouse_and_keys();
        set_camera(&cam.camera);

        // draw_primitives();
        draw_mesh(&mesh);
        egui_macroquad::draw();

        next_frame().await
    }
}
