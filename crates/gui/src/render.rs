use super::gpu::{
    CameraUniform, GpuCoordUniform, RenderSettings, Texture, VoxelType,
    dw::{
        DwChunkBuf, DwChunkObjId, DwChunkObjIdUniform, DwIconVertex, DwItemInstanceRaw, DwVertex,
    },
};
use eframe::{egui, egui_wgpu, wgpu};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use the_blockheads_tools_lib::game::{chunk::Chunk, coord::ChunkCoord};
use wgpu::util::DeviceExt;
use zune_png::PngDecoder;

const ID_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg32Uint;

pub(crate) enum ResizeOutcome {
    Unchanged,
    Resized,
}

pub(crate) struct GeometryBuffer {
    size: (u32, u32),
    uv: Texture,
    // Semi-transparent voxel pixels (alpha < 1.0) accumulated during ray marching.
    translucency: Texture,
    normal: Texture,
    flags: Texture,
    ssao_raw: Texture,
    ssao_blur: Texture,
    // transparent voxels needs depth texture of meshes to be obscured correctly during ray marching
    mesh_depth: Texture,
    voxel_depth: Texture,
    // A DepthOnly-aspect view of voxel_depth, for binding as texture_2d<f32> (Float sample type).
    voxel_depth_float_view: wgpu::TextureView,
    dyn_obj_id: Texture,
    overlay: Texture,
}

impl GeometryBuffer {
    pub const DEFAULT_WIDTH: u32 = 1920;
    pub const DEFAULT_HEIGHT: u32 = 1080;

    pub fn new(size: (u32, u32), device: &wgpu::Device) -> Self {
        let uv_texture = Texture::new(
            size,
            device,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::TextureFormat::Rgba16Float,
        );
        let normal_texture = Texture::new(
            size,
            device,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::TextureFormat::Rgba16Float,
        );
        let flags_texture = Texture::new(
            size,
            device,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::TextureFormat::R8Uint,
        );
        let ssao_raw_texture = Texture::new(
            size,
            device,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::TextureFormat::R8Unorm,
        );
        let ssao_blur_texture = Texture::new(
            size,
            device,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::TextureFormat::R8Unorm,
        );
        let voxel_depth = Texture::new(
            size,
            device,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::TextureFormat::Depth32Float,
        );
        let depth_texture = Texture::new(
            size,
            device,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::TextureFormat::Depth32Float,
        );
        let dyn_obj_id_texture = Texture::new(
            size,
            device,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            ID_TEXTURE_FORMAT,
        );
        let translucency_texture = Texture::new(
            size,
            device,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::TextureFormat::Bgra8Unorm,
        );
        let voxel_depth_float_view =
            voxel_depth
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    aspect: wgpu::TextureAspect::DepthOnly,
                    ..Default::default()
                });
        Self {
            size,
            uv: uv_texture,
            translucency: translucency_texture,
            normal: normal_texture,
            flags: flags_texture,
            ssao_raw: ssao_raw_texture,
            ssao_blur: ssao_blur_texture,
            voxel_depth,
            voxel_depth_float_view,
            mesh_depth: depth_texture,
            dyn_obj_id: dyn_obj_id_texture,
            overlay: Texture::new(
                size,
                device,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                wgpu::TextureFormat::Bgra8Unorm,
            ),
        }
    }

    pub fn default(device: &wgpu::Device) -> Self {
        Self::new((Self::DEFAULT_WIDTH, Self::DEFAULT_HEIGHT), device)
    }

    pub fn resize(&mut self, size: (u32, u32), device: &wgpu::Device) -> ResizeOutcome {
        if self.size != size {
            *self = Self::new(size, device);
            ResizeOutcome::Resized
        } else {
            ResizeOutcome::Unchanged
        }
    }
}

mod composite;
mod grid;
mod item;
pub(crate) mod item_selector;
mod mesh;
mod ssao;
pub(crate) mod voxel;

pub struct RenderResources {
    camera_buf: wgpu::Buffer,
    hover_on_block_buf: wgpu::Buffer,
    selected_block_buf: wgpu::Buffer,
    hover_on_id_buf: wgpu::Buffer,
    selected_id_buf: wgpu::Buffer,

    g_buffer: GeometryBuffer,

    // Stores the object ID of the pixel under cursor from g_buffer.dyn_obj_id
    staging_buffer: wgpu::Buffer,
    hover_on_dyn_obj_id: Arc<Mutex<Option<(DwChunkObjId, ChunkCoord)>>>,
    is_mapping: Arc<AtomicBool>,
    has_new_copy: Arc<AtomicBool>,

    voxel: voxel::VoxelRenderer,
    dw_item: item::DwItemRenderer,
    dw_mesh: mesh::DwMeshRenderer,
    grid: grid::GridRenderer,
    composite: composite::CompositeRenderer,
    ssao: ssao::SsaoRenderer,
    ssao_blur: ssao::SsaoBlurRenderer,
    item_selector: item_selector::ItemSelectorRenderer,

    render_settings_buf: wgpu::Buffer,
}

impl RenderResources {
    const STAGING_BUFFER_SIZE: u64 = std::mem::size_of::<(u32, u32)>() as u64; // read raw_id and packed_chunk

    pub fn new(
        state: &egui_wgpu::RenderState,
        camera_buf: wgpu::Buffer,
        selected_block_buf: wgpu::Buffer,
        hover_on_block_buf: wgpu::Buffer,
        hover_on_dyn_obj_id: Arc<Mutex<Option<(DwChunkObjId, ChunkCoord)>>>,
    ) -> Self {
        let device = &state.device;
        let queue = &state.queue;
        let target_format = state.target_format;

        let albedo_texture = {
            let bytes = include_bytes!("../resources/TileMap.png");
            let mut decoder = PngDecoder::new(bytes);
            let img = decoder.decode().unwrap().u8().unwrap();
            Texture::from_img(img.as_slice(), (512, 512), device, queue)
        };
        let items_texture = {
            let bytes = include_bytes!("../resources/Items.png");
            let mut decoder = PngDecoder::new(bytes);
            let img = decoder.decode().unwrap().u8().unwrap();
            Texture::from_img(img.as_slice(), (512, 256), device, queue)
        };
        let reflect_texture = {
            let bytes = include_bytes!("../resources/TileReflect.png");
            let mut decoder = PngDecoder::new(bytes);
            let img = decoder.decode().unwrap().u8().unwrap();
            Texture::from_img(img.as_slice(), (512, 512), device, queue)
        };
        let destruct_texture = {
            let bytes = include_bytes!("../resources/TileDestruct.png");
            let mut decoder = PngDecoder::new(bytes);
            let img = decoder.decode().unwrap().u8().unwrap();
            Texture::from_img(img.as_slice(), (512, 512), device, queue)
        };
        let g_buffer = GeometryBuffer::default(device);
        let staging_buffer = device.create_buffer(&wgpu::wgt::BufferDescriptor {
            label: Some("staging buffer"),
            size: Self::STAGING_BUFFER_SIZE,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let render_settings_buf = RenderSettings::default().buffer(device);
        let composite = composite::CompositeRenderer::new(
            device,
            &g_buffer,
            target_format,
            &render_settings_buf,
            &camera_buf,
            &albedo_texture,
            &reflect_texture,
            &destruct_texture,
        );
        let (hover_on_id_buf, selected_id_buf) = {
            let id_uniform = DwChunkObjIdUniform::default();
            (
                id_uniform.create_buffer(device),
                id_uniform.create_buffer(device),
            )
        };
        let default_world_dim_x =
            voxel::VoxelRenderer::DEFAULT_WORLD_WIDTH_CHUNK * Chunk::NUM_BLOCK_PER_ROW as u32;
        let world_dim_x_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("World Dim X Buffer"),
            contents: bytemuck::cast_slice(&[default_world_dim_x]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            voxel: voxel::VoxelRenderer::new(
                device,
                &camera_buf,
                world_dim_x_buf.clone(),
                &selected_block_buf,
                &hover_on_block_buf,
                &hover_on_id_buf,
                &selected_id_buf,
                &g_buffer,
                &render_settings_buf,
                &albedo_texture,
                &reflect_texture,
                &destruct_texture,
            ),
            dw_item: item::DwItemRenderer::new(
                device,
                &camera_buf,
                &items_texture,
                &albedo_texture,
                &hover_on_id_buf,
                &selected_id_buf,
                &render_settings_buf,
                &world_dim_x_buf,
            ),
            dw_mesh: mesh::DwMeshRenderer::new(
                device,
                &camera_buf,
                &albedo_texture,
                &render_settings_buf,
                &world_dim_x_buf,
            ),
            grid: grid::GridRenderer::new(
                device,
                &camera_buf,
                &world_dim_x_buf,
                &render_settings_buf,
                target_format,
            ),
            ssao: ssao::SsaoRenderer::new(device, queue, &camera_buf, &g_buffer),
            ssao_blur: ssao::SsaoBlurRenderer::new(device, &g_buffer),
            composite,
            item_selector: item_selector::ItemSelectorRenderer::new(
                device,
                &items_texture,
                &albedo_texture,
                target_format,
            ),

            camera_buf,
            selected_block_buf,
            hover_on_block_buf,
            hover_on_id_buf,
            selected_id_buf,

            g_buffer,

            staging_buffer,
            hover_on_dyn_obj_id,
            is_mapping: Arc::new(AtomicBool::new(false)),
            has_new_copy: Arc::new(AtomicBool::new(false)),
            render_settings_buf,
        }
    }

    pub fn voxel_buf(&self) -> &wgpu::Buffer {
        &self.voxel.voxel_buf
    }

    pub fn replace_voxel_buf(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        new_voxel_buf: wgpu::Buffer,
        world_dim_x: u32,
    ) {
        self.voxel
            .replace_voxel_buf(device, queue, new_voxel_buf, world_dim_x);
    }

    pub fn resize(&mut self, size: (u32, u32), device: &wgpu::Device) {
        if let ResizeOutcome::Resized = self.g_buffer.resize(size, device) {
            self.composite.resize(&self.g_buffer, device);
            self.ssao.resize(&self.camera_buf, &self.g_buffer, device);
            self.ssao_blur.resize(&self.g_buffer, device);
            self.voxel.resize(&self.g_buffer, device);
        }
    }

    pub fn render_mesh_pass(&self, render_pass: &mut wgpu::RenderPass<'_>, dw_buf: &[DwChunkBuf]) {
        self.dw_mesh.render(render_pass, dw_buf);
    }

    pub fn render_voxel_pass(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        self.voxel.render(render_pass);
    }

    pub fn render_annotation_pass(
        &self,
        render_pass: &mut wgpu::RenderPass<'_>,
        dw_buf: &[DwChunkBuf],
        render_settings: &RenderSettings,
    ) {
        if render_settings.render_dw_item {
            self.dw_item.render(render_pass, dw_buf);
        }
        if render_settings.show_grid {
            self.grid.render(render_pass);
        }
    }

    pub fn composite(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        self.composite.render(render_pass);
    }

    pub fn copy_hover_id(&self, pos: (u32, u32), encoder: &mut wgpu::CommandEncoder) {
        // Don't copy if the buffer is being mapped; or if the previous copy has not been consumed yet
        if self.is_mapping.load(Ordering::SeqCst) || self.has_new_copy.load(Ordering::SeqCst) {
            return;
        }

        let (x, y) = pos;
        let (sx, sy) = self.g_buffer.size;
        if x < sx && y < sy {
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfoBase {
                    texture: &self.g_buffer.dyn_obj_id.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x, y, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfoBase {
                    buffer: &self.staging_buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: None,
                        rows_per_image: None,
                    },
                },
                wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
            self.has_new_copy.store(true, Ordering::SeqCst);
        }
    }

    pub fn try_read_id(&self) {
        // Don't map when there's no new copy; or if there's an ongoing mapping
        if !self.has_new_copy.load(Ordering::SeqCst) || self.is_mapping.load(Ordering::SeqCst) {
            return;
        }

        self.is_mapping.store(true, Ordering::SeqCst);
        self.has_new_copy.store(false, Ordering::SeqCst);
        let is_mapping = self.is_mapping.clone();
        let hover_on_dyn_obj_id = self.hover_on_dyn_obj_id.clone();
        let staging_buffer = self.staging_buffer.clone();

        let bounds = 0..RenderResources::STAGING_BUFFER_SIZE;
        self.staging_buffer
            .map_async(wgpu::MapMode::Read, bounds.clone(), move |result| {
                if result.is_ok() {
                    let u32_size = std::mem::size_of::<u32>();
                    let buffer_view = staging_buffer.get_mapped_range(bounds);
                    let raw_id = u32::from_ne_bytes(
                        buffer_view[0..u32_size]
                            .try_into()
                            .expect("array size should be exactly 4"),
                    );
                    let packed_chunk = u32::from_ne_bytes(
                        buffer_view[u32_size..u32_size + u32_size]
                            .try_into()
                            .expect("array size should be exactly 4"),
                    );
                    let chunk_y = packed_chunk & 0b11111;
                    let chunk_x = packed_chunk >> 5;
                    let mut guard = hover_on_dyn_obj_id.lock().expect("should lock mutex");
                    *guard = DwChunkObjId::try_from_u32(raw_id)
                        .map(|id| (id, ChunkCoord::new(chunk_x, chunk_y as u8).unwrap()));
                }
                staging_buffer.unmap();
                is_mapping.store(false, Ordering::SeqCst);
            });
    }
}

pub struct Render3dCallback {
    pub camera_uniform: CameraUniform,
    pub dw_chunks: Vec<DwChunkBuf>,
    pub selected_block_coord_uniform: GpuCoordUniform,
    pub hover_on_block_coord_uniform: GpuCoordUniform,
    pub hover_on_id_uniform: DwChunkObjIdUniform,
    pub selected_id_uniform: DwChunkObjIdUniform,
    pub mouse_physical_pos: Option<(f32, f32)>,
    pub world_viewport_rect: egui::Rect,
    pub render_settings: RenderSettings,
}

impl egui_wgpu::CallbackTrait for Render3dCallback {
    fn prepare(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        egui_encoder: &mut eframe::wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<eframe::wgpu::CommandBuffer> {
        let r: &mut RenderResources = callback_resources.get_mut().unwrap();
        let (vp_w, vp_h) = (self.world_viewport_rect.size() * screen_descriptor.pixels_per_point)
            .round()
            .into();
        r.resize((vp_w as u32, vp_h as u32), device);
        queue.write_buffer(
            &r.camera_buf,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        queue.write_buffer(
            &r.selected_block_buf,
            0,
            bytemuck::cast_slice(&[self.selected_block_coord_uniform]),
        );
        queue.write_buffer(
            &r.hover_on_block_buf,
            0,
            bytemuck::cast_slice(&[self.hover_on_block_coord_uniform]),
        );
        queue.write_buffer(
            &r.hover_on_id_buf,
            0,
            bytemuck::cast_slice(&[self.hover_on_id_uniform]),
        );
        queue.write_buffer(
            &r.selected_id_buf,
            0,
            bytemuck::cast_slice(&[self.selected_id_uniform]),
        );
        queue.write_buffer(
            &r.render_settings_buf,
            0,
            bytemuck::cast_slice(&[self.render_settings.uniform()]),
        );
        {
            let mut render_pass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mesh pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.uv.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.normal.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.dyn_obj_id.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.translucency.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &r.g_buffer.mesh_depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if self.render_settings.render_dw_mesh {
                r.render_mesh_pass(&mut render_pass, &self.dw_chunks);
            }
        }

        {
            let mut render_pass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("voxel pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.uv.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.normal.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.translucency.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.flags.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &r.g_buffer.voxel_depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            r.render_voxel_pass(&mut render_pass);
        }

        if self.render_settings.enable_ssao {
            {
                let mut render_pass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("ssao pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.ssao_raw.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                r.ssao.render(&mut render_pass);
            }

            {
                let mut render_pass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("ssao blur pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.ssao_blur.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                r.ssao_blur.render(&mut render_pass);
            }
        }

        {
            let mut render_pass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("annotation pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.overlay.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.dyn_obj_id.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            r.render_annotation_pass(&mut render_pass, &self.dw_chunks, &self.render_settings);
        }

        if let Some((x, y)) = self.mouse_physical_pos {
            let x = x.round() as u32;
            let y = y.round() as u32;
            r.copy_hover_id((x, y), egui_encoder);
        }

        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'static>,
        callback_resources: &egui_wgpu::CallbackResources,
    ) {
        let r: &RenderResources = callback_resources.get().unwrap();
        r.composite(render_pass);
    }
}

pub struct ItemSelectorCallback {
    pub hovered_index: Option<u32>,
    pub selected_index: u32,
    /// Viewport in grid-pixel space: origin = scroll offset, size = visible area.
    pub viewport: egui::Rect,
    pub pixels_per_point: f32,
}

impl egui_wgpu::CallbackTrait for ItemSelectorCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let r: &mut RenderResources = callback_resources.get_mut().unwrap();
        let viewport = self.viewport * self.pixels_per_point;
        r.item_selector.prepare(
            device,
            queue,
            self.hovered_index,
            self.selected_index,
            viewport.min.into(),
            viewport.size().into(),
            self.pixels_per_point,
        );
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &egui_wgpu::CallbackResources,
    ) {
        let r: &RenderResources = callback_resources.get().unwrap();
        r.item_selector.render(render_pass);
    }
}
