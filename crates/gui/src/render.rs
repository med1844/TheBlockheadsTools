use super::{
    gpu::{
        CameraUniform, GpuCoordUniform, RenderSettings, Texture, VoxelType,
        dw::{
            DwChunkBuf, DwChunkObjId, DwChunkObjIdUniform, DwIconInstanceRaw, DwIconVertex,
            DwVertex,
        },
    },
    image_type::ImageType,
};
use eframe::{egui, egui_wgpu, wgpu};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use zune_png::PngDecoder;

const ID_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Uint;

pub(crate) enum ResizeOutcome {
    Unchanged,
    Resized,
}

pub(crate) struct GeometryBuffer {
    size: (u32, u32),
    albedo: Texture,
    /// Semi-transparent voxel pixels (alpha < 1.0) accumulated during ray marching.
    translucency: Texture,
    normal_spec: Texture,
    ssao_raw: Texture,
    ssao_blur: Texture,
    // transparent voxels needs depth texture of meshes to be obscured correctly during ray marching
    mesh_depth: Texture,
    voxel_depth: Texture,
    /// A DepthOnly-aspect view of voxel_depth, for binding as texture_2d<f32> (Float sample type).
    voxel_depth_float_view: wgpu::TextureView,
    dyn_obj_id: Texture,
    overlay: Texture,
}

impl GeometryBuffer {
    pub const DEFAULT_WIDTH: u32 = 1920;
    pub const DEFAULT_HEIGHT: u32 = 1080;

    pub fn new(size: (u32, u32), device: &wgpu::Device) -> Self {
        let color_texture = Texture::new(
            size,
            device,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::TextureFormat::Bgra8Unorm,
        );
        let normal_spec_texture = Texture::new(
            size,
            device,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::TextureFormat::Rgba16Float,
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
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            wgpu::TextureFormat::R32Uint,
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
            albedo: color_texture,
            translucency: translucency_texture,
            normal_spec: normal_spec_texture,
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
mod icon;
mod sprite;
mod ssao;
mod voxel;

pub struct RenderResources {
    camera_buf: wgpu::Buffer,
    selected_block_buf: wgpu::Buffer,
    hover_on_block_buf: wgpu::Buffer,
    hover_on_chunk_buf: wgpu::Buffer, // for DW highlighting as the DwChunkObjId has no chunk coords
    hover_on_id_buf: wgpu::Buffer,

    g_buffer: GeometryBuffer,

    // Stores the object ID of the pixel under cursor from g_buffer.dyn_obj_id
    staging_buffer: wgpu::Buffer,
    hover_on_dyn_obj_id: Arc<Mutex<Option<DwChunkObjId>>>,
    is_mapping: Arc<AtomicBool>,
    has_new_copy: Arc<AtomicBool>,

    voxel: voxel::VoxelRenderer,
    dw_icon: icon::DwIconRenderer,
    dw_sprite: sprite::DwSpriteRenderer,
    grid: grid::GridRenderer,
    composite: composite::CompositeRenderer,
    ssao: ssao::SsaoRenderer,
    ssao_blur: ssao::SsaoBlurRenderer,

    render_settings_buf: wgpu::Buffer,
}

impl RenderResources {
    const STAGING_BUFFER_SIZE: u64 = std::mem::size_of::<u32>() as u64; // only read single pixel

    pub fn new(
        state: &egui_wgpu::RenderState,
        camera_buf: wgpu::Buffer,
        voxel_buf: wgpu::Buffer,
        selected_block_buf: wgpu::Buffer,
        hover_on_block_buf: wgpu::Buffer,
        hover_on_chunk_buf: wgpu::Buffer,
        hover_on_dyn_obj_id: Arc<Mutex<Option<DwChunkObjId>>>,
    ) -> Self {
        let device = &state.device;
        let queue = &state.queue;
        let target_format = state.target_format;

        let tile_map_texture = {
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
        );
        let hover_on_id_buf = {
            let hover_on_id_uniform: DwChunkObjIdUniform = {
                let guard = hover_on_dyn_obj_id.lock().expect("should lock");
                guard.as_ref().into()
            };
            hover_on_id_uniform.create_buffer(device)
        };

        Self {
            voxel: voxel::VoxelRenderer::new(
                device,
                voxel_buf,
                &camera_buf,
                &selected_block_buf,
                &hover_on_block_buf,
                &tile_map_texture,
                target_format,
                &g_buffer,
                &render_settings_buf,
                &reflect_texture,
                &destruct_texture,
            ),
            dw_icon: icon::DwIconRenderer::new(
                device,
                &camera_buf,
                &items_texture,
                &tile_map_texture,
                &hover_on_chunk_buf,
                &hover_on_id_buf,
            ),
            dw_sprite: sprite::DwSpriteRenderer::new(
                device,
                &camera_buf,
                &tile_map_texture,
                &hover_on_chunk_buf,
                &hover_on_id_buf,
                target_format,
            ),
            grid: grid::GridRenderer::new(device, &camera_buf, target_format),
            ssao: ssao::SsaoRenderer::new(device, queue, &camera_buf, &g_buffer),
            ssao_blur: ssao::SsaoBlurRenderer::new(device, &g_buffer),
            composite,

            camera_buf,
            selected_block_buf,
            hover_on_block_buf,
            hover_on_chunk_buf,
            hover_on_id_buf,

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

    pub fn resize(&mut self, size: (u32, u32), device: &wgpu::Device) {
        if let ResizeOutcome::Resized = self.g_buffer.resize(size, device) {
            self.composite
                .resize(&self.g_buffer, &self.render_settings_buf, device);
            self.ssao.resize(&self.camera_buf, &self.g_buffer, device);
            self.ssao_blur.resize(&self.g_buffer, device);
            self.voxel.resize(&self.g_buffer, device);
        }
    }

    pub fn render_mesh_pass(&self, render_pass: &mut wgpu::RenderPass<'_>, dw_buf: &[DwChunkBuf]) {
        self.dw_sprite.render(render_pass, dw_buf);
    }

    pub fn render_voxel_pass(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        self.voxel.render(render_pass);
    }

    pub fn render_annotation_pass(
        &self,
        render_pass: &mut wgpu::RenderPass<'_>,
        dw_buf: &[DwChunkBuf],
        show_grid: bool,
    ) {
        self.dw_icon.render(render_pass, dw_buf);
        if show_grid {
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
                    let buffer_view = staging_buffer.get_mapped_range(bounds);
                    let raw_id = u32::from_ne_bytes(
                        buffer_view[..]
                            .try_into()
                            .expect("array size should be exactly 4"),
                    );
                    let mut guard = hover_on_dyn_obj_id.lock().expect("should lock mutex");
                    *guard = DwChunkObjId::try_from_u32(raw_id);
                }
                staging_buffer.unmap();
                is_mapping.store(false, Ordering::SeqCst);
            });
    }
}

pub struct Render3dCallback {
    pub camera_uniform: CameraUniform,
    pub dw_chunks: Vec<DwChunkBuf>,
    pub show_grid: bool,
    pub selected_block_coord_uniform: GpuCoordUniform,
    pub hover_on_block_coord_uniform: GpuCoordUniform,
    pub hover_on_chunk_coord_uniform: GpuCoordUniform,
    pub hover_on_id_uniform: DwChunkObjIdUniform,
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
            &r.hover_on_chunk_buf,
            0,
            bytemuck::cast_slice(&[self.hover_on_chunk_coord_uniform]),
        );
        queue.write_buffer(
            &r.hover_on_id_buf,
            0,
            bytemuck::cast_slice(&[self.hover_on_id_uniform]),
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
                        view: &r.g_buffer.albedo.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.normal_spec.view,
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
            r.render_mesh_pass(&mut render_pass, &self.dw_chunks);
        }

        {
            let mut render_pass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("voxel pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.albedo.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.normal_spec.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
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
                    // slot 3: translucency — cleared each voxel pass
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
            r.render_annotation_pass(&mut render_pass, &self.dw_chunks, self.show_grid);
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
