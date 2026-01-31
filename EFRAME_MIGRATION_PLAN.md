# Migration Plan: egui + wgpu → eframe

This document outlines the architectural changes needed to migrate from your current manual egui+wgpu integration to using eframe as the framework. The goal is to simplify input handling and enable proper redraw-on-demand.

---

## Table of Contents

1. [Why eframe?](#why-eframe)
2. [Current Architecture Overview](#current-architecture-overview)
3. [Target Architecture with eframe](#target-architecture-with-eframe)
4. [Key Concepts in eframe](#key-concepts-in-eframe)
5. [Migration Steps](#migration-steps)
6. [The `egui_wgpu::CallbackTrait` Pattern](#the-egui_wgpucallbacktrait-pattern)
7. [Detailed Changes by File](#detailed-changes-by-file)
8. [Expected Benefits](#expected-benefits)
9. [Potential Challenges](#potential-challenges)

---

## Why eframe?

Your current pain points:

```rust
let response = self
    .state
    .as_mut()
    .unwrap()
    .egui_renderer
    .handle_input(window, &event);

if !response.consumed {
    let _ = self.state.as_mut().unwrap().handle_input(window, &event);
}
```

**Problems with this approach:**
1. **Input ownership is ambiguous**: You're manually routing events between egui and your 3D scene, which is error-prone
2. **Always redraws**: `egui_renderer.handle_input()` returns `EventResponse`, but the way you've structured the event loop (`ControlFlow::Poll` + always calling `request_redraw()` in `about_to_wait`), you redraw every frame regardless of whether anything changed
3. **Manual orchestration**: You manage the separate rendering passes (3D → egui) yourself

**eframe solves these by:**
1. Owning the event loop and input handling—egui decides when it consumed an event
2. Native support for `run_and_return` / continuous vs reactive modes
3. First-class support for custom 3D rendering via `egui_wgpu::CallbackTrait`

---

## Current Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                           main.rs                                    │
│   EventLoop::new() → event_loop.run_app(&mut app)                   │
└─────────────────────────┬───────────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────────────────────┐
│                          App (ApplicationHandler)                    │
│  ┌─────────────────┐  ┌───────────────────────────────────────────┐  │
│  │  wgpu::Instance │  │               AppState                    │  │
│  └─────────────────┘  │  ┌──────────────────────────────────────┐ │  │
│                       │  │ Manual wgpu setup:                   │ │  │
│                       │  │ - device, queue, surface             │ │  │
│                       │  │ - surface_config                     │ │  │
│                       │  └──────────────────────────────────────┘ │  │
│                       │  ┌──────────────────────────────────────┐ │  │
│                       │  │ EguiRenderer (egui-winit + egui-wgpu)│ │  │
│                       │  │ - state: egui_winit::State           │ │  │
│                       │  │ - renderer: egui_wgpu::Renderer      │ │  │
│                       │  └──────────────────────────────────────┘ │  │
│                       │  ┌──────────────────────────────────────┐ │  │
│                       │  │ Custom 3D Renderers:                 │ │  │
│                       │  │ - VoxelRenderer                      │ │  │
│                       │  │ - DwIconRenderer                     │ │  │
│                       │  │ - GridRenderer                       │ │  │
│                       │  └──────────────────────────────────────┘ │  │
│                       │  ┌──────────────────────────────────────┐ │  │
│                       │  │ Input (custom mouse tracking)        │ │  │
│                       │  └──────────────────────────────────────┘ │  │
│                       └───────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────┘

Render Flow:
1. App::handle_redraw()
2.   → 3D Pass (VoxelRenderer, DwIconRenderer, GridRenderer)
3.   → egui_renderer.begin_frame()
4.   → build egui UI
5.   → egui_renderer.end_frame_and_draw()
6.   → queue.submit()
```

**Key files:**
- `main.rs` - Event loop setup with winit
- `app.rs` - `App` (ApplicationHandler) + `AppState` 
- `renderer.rs` - `EguiRenderer` wrapper + custom wgpu renderers
- `input.rs` - Manual mouse state tracking

---

## Target Architecture with eframe

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           main.rs                                       │
│   eframe::run_native(options, Box::new(|cc| Ok(Box::new(App::new(cc)))))│
└─────────────────────────┬───────────────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────────────────────┐
│                           App (eframe::App)                          │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────────┐│
│  │ App::new(cc: &CreationContext)                                   ││
│  │   - Access wgpu via cc.wgpu_render_state                         ││
│  │   - Register 3D resources in callback_resources                  ││
│  └──────────────────────────────────────────────────────────────────┘│
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────────┐│
│  │ App::update(&mut self, ctx: &Context, frame: &mut Frame)         ││
│  │   - Build egui UI                                                ││
│  │   - Use ui.painter().add(egui_wgpu::Callback::new_paint_callback)││
│  │     to insert 3D rendering into egui's render pass               ││
│  └──────────────────────────────────────────────────────────────────┘│
│                                                                      │
│  State:                                                              │
│  - camera, world_db, voxel_buf, etc. (same as before)                │
│  - NO manual Input struct needed (use egui's response system)        │
│  - NO EguiRenderer needed (eframe handles this)                      │
└──────────────────────────────────────────────────────────────────────┘

         ▲
         │ registered in cc.wgpu_render_state.renderer.callback_resources
         │
┌─────────────────────────────────────────────────────────────────────┐
│             Custom3dResources (stored in callback_resources)        │
│  - VoxelRenderer's pipeline, bind_group, buffers                    │
│  - DwIconRenderer's pipeline, bind_group, buffers                   │
│  - GridRenderer's pipeline, bind_group, buffers                     │
│  - depth_view (if you need it)                                      │
└─────────────────────────────────────────────────────────────────────┘

         ▲
         │ implements egui_wgpu::CallbackTrait
         │
┌─────────────────────────────────────────────────────────────────────────┐
│                    Custom3dCallback (per-frame data)                    │
│  - camera_buf reference or copy                                         │
│  - which chunks to render                                               │
│  - grid_enabled flag                                                    │
│                                                                         │
│  fn prepare() → update uniform buffers (camera, etc.)                   │
│  fn paint()   → issue draw calls using resources from callback_resources│
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Key Concepts in eframe

### 1. `eframe::App` Trait

Instead of implementing `winit::ApplicationHandler`, you implement `eframe::App`:

```rust
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Build your entire UI here
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            // menu bar
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            // 3D viewport
            self.render_3d_viewport(ui);
        });
    }
}
```

### 2. `CreationContext` and wgpu Access

When creating your app, you receive a `CreationContext` that gives you access to the wgpu state:

```rust
impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let wgpu_state = cc.wgpu_render_state.as_ref().unwrap();
        let device = &wgpu_state.device;
        let queue = &wgpu_state.queue;
        
        // Create your pipelines here
        let voxel_pipeline = create_voxel_pipeline(device, wgpu_state.target_format);
        
        // Store resources in callback_resources for later access in paint()
        wgpu_state.renderer.write().callback_resources.insert(MyResources {
            voxel_pipeline,
            // ...
        });
        
        Self { /* ... */ }
    }
}
```

### 3. `egui_wgpu::CallbackTrait` - The Core Pattern

This is how you inject custom wgpu rendering into egui's render pass:

```rust
struct My3dCallback {
    // Per-frame data needed for rendering
    camera_matrix: [[f32; 4]; 4],
    visible_chunks: Vec<ChunkCoord>,
    grid_enabled: bool,
}

impl egui_wgpu::CallbackTrait for My3dCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        // Called BEFORE paint - update your uniform buffers here
        let my_resources: &MyResources = resources.get().unwrap();
        queue.write_buffer(&my_resources.camera_buf, 0, bytemuck::bytes_of(&self.camera_matrix));
        Vec::new() // Return extra command buffers if needed
    }
    
    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        // Issue draw calls here - this runs INSIDE egui's render pass
        let my_resources: &MyResources = resources.get().unwrap();
        
        render_pass.set_pipeline(&my_resources.voxel_pipeline);
        render_pass.set_bind_group(0, &my_resources.voxel_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
        
        // Grid, icons, etc.
    }
}
```

### 4. Using the Callback in UI Code

```rust
fn render_3d_viewport(&mut self, ui: &mut egui::Ui) {
    // Allocate space for the 3D viewport
    let (rect, response) = ui.allocate_exact_size(
        ui.available_size(),
        egui::Sense::click_and_drag()
    );
    
    // Handle input using egui's response system!
    if response.dragged() {
        let delta = response.drag_delta();
        self.camera.pan(delta.x, delta.y);
    }
    if response.hovered() {
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        self.camera.zoom(scroll);
    }
    if response.clicked() {
        self.select_block_at_cursor();
    }
    
    // Add the 3D render callback
    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        My3dCallback {
            camera_matrix: self.camera.view_proj_matrix(),
            visible_chunks: self.get_visible_chunks(),
            grid_enabled: self.grid_enabled,
        },
    ));
}
```

### 5. Redraw Control

eframe respects `ctx.request_repaint()`. If no UI changed and no repaint was requested, it won't redraw:

```rust
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // Only request repaint if camera is animating
    if self.camera.is_animating() {
        ctx.request_repaint();
    }
    
    // Or use continuous mode for always-on rendering:
    // ctx.request_repaint(); // Uncomment for continuous mode
}
```

---

## Migration Steps

### Phase 1: Dependency Changes

**Cargo.toml changes:**
```toml
# Remove these:
# egui-wgpu = "0.31.1"
# egui-winit = "0.31.1"  
# winit = "0.30.9"
# pollster = "0.4.0"

# Add this:
eframe = { version = "0.31", default-features = false, features = ["wgpu"] }
```

> Note: `eframe` bundles `egui`, `egui-wgpu`, `egui-winit`, and `winit` internally.

### Phase 2: Restructure Your Types

**Create a new resources struct:**
```rust
// renderer.rs or new file: resources.rs
pub struct RenderResources {
    // Voxel rendering
    pub voxel_pipeline: wgpu::RenderPipeline,
    pub voxel_bind_group: wgpu::BindGroup,
    pub voxel_buf: wgpu::Buffer,
    pub camera_buf: wgpu::Buffer,
    pub selected_block_buf: wgpu::Buffer,
    pub hover_on_block_buf: wgpu::Buffer,
    
    // DW icon rendering  
    pub dw_icon_pipeline: wgpu::RenderPipeline,
    pub dw_icon_bind_group: wgpu::BindGroup,
    pub dw_icon_vertex_buf: wgpu::Buffer,
    pub dw_icon_index_buf: wgpu::Buffer,
    
    // Grid rendering
    pub grid_pipeline: wgpu::RenderPipeline,
    pub grid_bind_group: wgpu::BindGroup,
    
    // Depth texture (you'll need to manage this carefully, see Challenges)
    pub depth_view: wgpu::TextureView,
}
```

### Phase 3: Rewrite `App`

```rust
pub struct App {
    // Game state (keep these)
    world_db: Option<WorldDb>,
    camera: Camera,
    
    // UI state (keep these)
    show_info: bool,
    load_err: Option<BhError>,
    save_err: Option<BhError>,
    grid_enabled: bool,
    
    // Input state - SIMPLIFIED (egui handles most of this now)
    selected_block: Option<BlockCoord>,
    hover_block: Option<BlockCoord>,
    
    // FPS
    fps_counter: FpsCounter,
    
    // Chunk data for dynamic objects
    dw_chunks: HashMap<ChunkCoord, DwChunkData>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> Option<Self> {
        let wgpu_state = cc.wgpu_render_state.as_ref()?;
        let device = &wgpu_state.device;
        let queue = &wgpu_state.queue;
        let target_format = wgpu_state.target_format;
        
        // Create all your pipelines and resources
        let resources = RenderResources::new(device, queue, target_format);
        
        // Store in callback_resources
        wgpu_state.renderer.write().callback_resources.insert(resources);
        
        Some(Self {
            world_db: None,
            camera: Camera::default(),
            show_info: false,
            load_err: None,
            save_err: None,
            grid_enabled: false,
            selected_block: None,
            hover_block: None,
            fps_counter: FpsCounter::new(2.0),
            dw_chunks: HashMap::new(),
        })
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.fps_counter.update();
        
        // Top menu bar
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            self.render_menu_bar(ui);
        });
        
        // Info side panel
        egui::SidePanel::left("info")
            .resizable(false)
            .show_animated(ctx, self.show_info, |ui| {
                self.render_info_panel(ui);
            });
        
        // Central 3D viewport
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                self.render_3d_viewport(ctx, ui, frame);
            });
        
        // Error dialogs
        self.render_error_dialogs(ctx);
    }
}
```

### Phase 4: Implement the 3D Viewport

```rust
impl App {
    fn render_3d_viewport(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let available = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available, egui::Sense::click_and_drag());
        
        // ========== INPUT HANDLING (egui style!) ==========
        
        // Dragging = pan camera
        if response.dragged_by(egui::PointerButton::Primary) && self.world_db.is_some() {
            let delta = response.drag_delta();
            // Convert screen delta to world delta based on zoom level
            self.camera.pan(-delta.x, delta.y);
            ctx.request_repaint();
        }
        
        // Scroll = zoom
        if response.hovered() {
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll.abs() > 0.0 {
                self.camera.zoom(scroll);
                ctx.request_repaint();
            }
        }
        
        // Click = select block
        if response.clicked() {
            if let Some(world_pos) = self.screen_to_world(response.interact_pointer_pos()) {
                self.selected_block = BlockCoord::new(world_pos.x as u32, world_pos.y as u16).ok();
            }
        }
        
        // Hover = highlight block
        if response.hovered() {
            if let Some(hover_pos) = response.hover_pos() {
                if let Some(world_pos) = self.screen_to_world(Some(hover_pos)) {
                    self.hover_block = BlockCoord::new(world_pos.x as u32, world_pos.y as u16).ok();
                }
            }
        }
        
        // ========== RENDER CALLBACK ==========
        
        // Get render state to update buffers
        if let Some(wgpu_state) = frame.wgpu_render_state() {
            // Update camera uniform (we do this here, not in prepare(), because we need &self)
            let queue = &wgpu_state.queue;
            let resources = wgpu_state.renderer.read();
            if let Some(r) = resources.callback_resources.get::<RenderResources>() {
                // Update camera buffer
                self.camera.write_to_buffer(queue, &r.camera_buf, rect.width(), rect.height());
                
                // Update selected/hover block buffers
                // ... 
            }
        }
        
        // Schedule the 3D rendering
        let visible_chunks = self.calculate_visible_chunks(rect);
        
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            Render3dCallback {
                visible_chunks,
                grid_enabled: self.grid_enabled,
                dw_chunks: self.get_dw_chunk_indices_to_render(),
            },
        ));
    }
}
```

### Phase 5: Implement the Callback

```rust
struct Render3dCallback {
    visible_chunks: Vec<ChunkCoord>,
    grid_enabled: bool,
    dw_chunks: Vec<ChunkCoord>,
}

impl egui_wgpu::CallbackTrait for Render3dCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        _resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        // We already updated buffers in render_3d_viewport, so nothing to do here
        // OR you could do buffer updates here if you restructure
        Vec::new()
    }
    
    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        let r: &RenderResources = resources.get().unwrap();
        
        // 1. Draw voxels (fullscreen quad with raymarching)
        render_pass.set_pipeline(&r.voxel_pipeline);
        render_pass.set_bind_group(0, &r.voxel_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
        
        // 2. Draw dynamic object icons
        for chunk_coord in &self.dw_chunks {
            if let Some(chunk_buf) = r.dw_chunks.get(chunk_coord) {
                render_pass.set_pipeline(&r.dw_icon_pipeline);
                render_pass.set_bind_group(0, &r.dw_icon_bind_group, &[]);
                render_pass.set_vertex_buffer(0, r.dw_icon_vertex_buf.slice(..));
                render_pass.set_vertex_buffer(1, chunk_buf.instance_buf.slice(..));
                render_pass.set_index_buffer(r.dw_icon_index_buf.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..DwIconVertex::INDICES.len() as u32, 0, 0..chunk_buf.num_instances);
            }
        }
        
        // 3. Draw grid if enabled
        if self.grid_enabled {
            render_pass.set_pipeline(&r.grid_pipeline);
            render_pass.set_bind_group(0, &r.grid_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
    }
}
```

### Phase 6: Delete/Simplify Files

| File | Action |
|------|--------|
| `input.rs` | **DELETE** - egui handles input via `Response` |
| `renderer.rs::EguiRenderer` | **DELETE** - eframe handles this internally |
| `renderer.rs::VoxelRenderer` | **KEEP** but convert to just pipeline creation, store in `callback_resources` |
| `renderer.rs::DwIconRenderer` | **KEEP** but convert similarly |
| `renderer.rs::GridRenderer` | **KEEP** but convert similarly |
| `main.rs` | **SIMPLIFY** - just call `eframe::run_native()` |

---

## The `egui_wgpu::CallbackTrait` Pattern

### Three Stages of Callback Execution

```
┌─────────────────────────────────────────────────────────────────────┐
│  1. PREPARE (per callback)                                          │
│     Called before any paint() calls in this frame.                  │
│     Use this to update uniform buffers, upload data, etc.           │
│     Has access to: device, queue, encoder, callback_resources       │
│     Returns: Vec<CommandBuffer> (additional commands)               │
└─────────────────────────────────────────────────────────────────────┘
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│  2. FINISH_PREPARE (once per frame, optional)                       │
│     Only if you register a FinishPrepareCallback.                   │
│     Use for work that needs to happen after ALL callbacks prepared. │
└─────────────────────────────────────────────────────────────────────┘
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│  3. PAINT (per callback)                                            │
│     Called during egui's main render pass.                          │
│     Issue your draw commands here.                                  │
│     Has access to: render_pass, callback_resources                  │
│                                                                     │
│     ⚠️ IMPORTANT: You're INSIDE egui's render pass!                  │
│     - You share the same color attachment as egui                   │
│     - Depth buffer behavior depends on how egui was configured      │
└─────────────────────────────────────────────────────────────────────┘
```

### Callback Resources

`callback_resources` is a type-map (like `anymap`). You insert your struct once during app creation, then retrieve it in callbacks:

```rust
// Insert (in App::new)
wgpu_state.renderer.write().callback_resources.insert(MyResources { ... });

// Retrieve (in CallbackTrait::paint)
let resources: &MyResources = resources.get().unwrap();

// You can insert multiple types:
wgpu_state.renderer.write().callback_resources.insert(VoxelResources { ... });
wgpu_state.renderer.write().callback_resources.insert(GridResources { ... });
```

---

## Detailed Changes by File

### `main.rs`

**Before:**
```rust
fn main() {
    pollster::block_on(run());
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = app::App::new();
    event_loop.run_app(&mut app).expect("Failed to run app");
}
```

**After:**
```rust
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0])
            .with_title("The Blockheads Tools"),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: egui_wgpu::WgpuConfiguration {
            // Configure if needed
            present_mode: wgpu::PresentMode::AutoVsync,
            ..Default::default()
        },
        ..Default::default()
    };
    
    eframe::run_native(
        "The Blockheads Tools",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc).expect("Failed to create app")))),
    )
}
```

### `app.rs`

- **Remove**: `ApplicationHandler` impl, `wgpu::Instance`, `surface`, `surface_config`, `resize_surface()`, `handle_redraw()`, `set_window()`, etc.
- **Remove**: `EguiRenderer` field and all its usage
- **Remove**: `Input` field (replace with egui response-based input)
- **Keep**: Game state (`world_db`, `camera_buf`, `voxel_buf`, etc.)
- **Add**: `eframe::App` impl with `update()` method

### `renderer.rs`

- **Remove**: `EguiRenderer` struct entirely
- **Refactor**: `VoxelRenderer`, `DwIconRenderer`, `GridRenderer` to just create pipelines/bind groups and return them (no struct wrapping needed if stored in callback_resources)
- **Add**: `RenderResources` struct to hold all GPU resources
- **Add**: `Render3dCallback` implementing `CallbackTrait`

### `input.rs`

- **DELETE** this file entirely
- Input is now handled via `egui::Response`:
  - `response.clicked()` - single click
  - `response.dragged()` - mouse drag
  - `response.drag_delta()` - how much it dragged
  - `response.hovered()` - mouse is over this area
  - `ui.input(|i| i.raw_scroll_delta)` - scroll wheel

---

## Expected Benefits

| Aspect | Before | After |
|--------|--------|-------|
| **Input handling** | Manual two-stage routing, error-prone | egui's `Response` API, clear ownership |
| **Redraw control** | Always redraws (`ControlFlow::Poll`) | Only redraws when needed (or when you call `ctx.request_repaint()`) |
| **Event loop** | Manual `winit` management | eframe handles it |
| **GPU init** | Manual adapter/device/surface setup | eframe handles it, you just use it |
| **Code complexity** | ~672 lines in app.rs | Likely ~400-500 lines |
| **Depth buffer** | Manual creation and management | Need to handle carefully (see Challenges) |

---

## Potential Challenges

### 1. Depth Buffer

**Problem**: egui's render pass doesn't have a depth buffer by default. Your voxel renderer uses depth testing (`depth_compare: wgpu::CompareFunction::Less`).

**Solutions**:
- **Option A**: Use `Frame::canvas_with_options()` and configure depth format — but this only works for the egui canvas, not for the callback
- **Option B**: Create your own depth texture, pass it via callback_resources, and manually set up depth in your pipeline (but you won't be able to attach it to egui's render pass)
- **Option C**: Since your voxel renderer is a single fullscreen raymarching pass, you might not actually need depth testing. The raymarcher handles occlusion internally. Only the DW icons and grid might need depth, and you could handle their ordering manually.

**Recommendation**: Start by removing depth stencil from your pipelines and see if it works. If you need proper depth for icons/grid, you may need to render to an intermediate texture and composite.

### 2. Render Pass Sharing

**Problem**: In the callback, you're inside egui's render pass. You can't start a new render pass or change the render target.

**Solution**: All your 3D rendering must work within that single pass. This means:
- No multi-pass effects
- No render-to-texture (within the callback)
- Depth buffer limitations (see above)

For your use case (voxel raymarcher + icons + grid), this should be fine.

### 3. Resource Mutability

**Problem**: In `CallbackTrait::paint()`, you only get `&egui_wgpu::CallbackResources`, not `&mut`. You can't mutate your resources during paint.

**Solution**: Do all mutation in `prepare()` or outside the callback (in `App::update()`). For buffer updates, use `queue.write_buffer()` which takes `&self`.

### 4. Per-Frame Data in Callbacks

**Problem**: The callback must be `'static` and `Send + Sync`, so you can't capture references to `App`.

**Solution**: Copy any per-frame data into the callback struct:
```rust
Render3dCallback {
    camera_matrix: self.camera.view_proj(),  // Copy the matrix
    visible_chunks: self.visible_chunks.clone(),  // Clone the list
}
```

### 5. Viewport Size Changes

**Problem**: Your depth texture needs to be resized when the viewport changes.

**Solution**: Check the size in `prepare()` and recreate if needed:
```rust
fn prepare(&self, device: &Device, ..., resources: &mut CallbackResources) -> ... {
    let r: &mut RenderResources = resources.get_mut().unwrap();
    let current_size = (screen_descriptor.size_in_pixels[0], screen_descriptor.size_in_pixels[1]);
    if r.depth_size != current_size {
        r.recreate_depth_view(device, current_size);
    }
}
```

### 6. Accessing `device` and `queue` Outside Callbacks

**Problem**: Your `open_world_db()` function needs `device` and `queue` to:
- Clear/resize `voxel_buf` 
- Create new `GridRenderer`
- Upload chunk data via `from_chunks(&queue, ...)`
- Create DW chunk buffers with `set_chunk(&device, ...)`

But in eframe, **you don't own these** - eframe does. You only get temporary access through:
1. `&eframe::CreationContext` in `App::new()`
2. `&eframe::Frame` in `App::update()`

**The Core Issue**: In your current design:
```rust
fn render_menu_bar(&mut self, ui: &mut egui::Ui) {
    // ... menu code ...
    if opened_path.is_some() {
        self.open_world_db(opened_path.unwrap());  // ❌ No access to device/queue here!
    }
}
```

You can't call `open_world_db()` from here because you don't have `frame` available.

**Solutions**:

#### Option A: Pass `frame` Down to All Functions That Need It (Simplest)

Change your function signatures to accept `frame`:

```rust
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            self.render_menu_bar(ui, frame);  // Pass frame
        });
        // ...
    }
}

impl App {
    fn render_menu_bar(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let mut opened_path = None;
        
        // ... menu UI code ...
        
        if let Some(path) = opened_path {
            if let Some(wgpu_state) = frame.wgpu_render_state() {
                if let Err(e) = self.open_world_db(path, &wgpu_state.device, &wgpu_state.queue) {
                    self.load_err = Some(e);
                }
            }
        }
    }
    
    fn open_world_db(&mut self, path: impl AsRef<Path>, device: &wgpu::Device, queue: &wgpu::Queue) -> BhResult<()> {
        // Now you have device and queue!
        self.voxel_buf.clear(queue);
        self.voxel_buf = VoxelBuf::new(device, new_world_width_macro);
        // ...
    }
}
```

#### Option B: Defer Actions to update() (More Decoupled)

Store a "pending action" in your App, then execute it in `update()` where you have `frame`:

```rust
pub struct App {
    // ... other fields ...
    
    /// Pending file to open (set by UI, processed in update)
    pending_open: Option<PathBuf>,
    pending_save: Option<PathBuf>,
}

impl App {
    fn render_menu_bar(&mut self, ui: &mut egui::Ui) {
        // UI code only - no device/queue access needed
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    self.pending_open = rfd::FileDialog::new().pick_folder();
                }
                if ui.button("Save as").clicked() {
                    self.pending_save = rfd::FileDialog::new().pick_folder();
                }
            });
        });
        // Don't process here - just store the path
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Process pending actions first, while we have access to frame
        if let Some(path) = self.pending_open.take() {
            if let Some(wgpu_state) = frame.wgpu_render_state() {
                if let Err(e) = self.open_world_db(path, &wgpu_state.device, &wgpu_state.queue) {
                    self.load_err = Some(e);
                }
            }
        }
        
        if let Some(path) = self.pending_save.take() {
            // Handle save...
        }
        
        // Now render UI
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            self.render_menu_bar(ui);  // No frame needed here anymore
        });
        
        // ... rest of update
    }
}
```

#### Option C: Access via callback_resources (For Resources That Live There)

If your `VoxelBuf` and `GridRenderer` live in `callback_resources`, you can access them through the renderer's write lock:

```rust
fn open_world_db(&mut self, path: impl AsRef<Path>, frame: &eframe::Frame) -> BhResult<()> {
    let wgpu_state = frame.wgpu_render_state().unwrap();
    let device = &wgpu_state.device;
    let queue = &wgpu_state.queue;
    
    // Access mutable callback_resources
    let mut renderer = wgpu_state.renderer.write();
    let resources = renderer.callback_resources.get_mut::<RenderResources>().unwrap();
    
    // Now modify GPU resources
    resources.voxel_buf.clear(queue);
    resources.voxel_buf = VoxelBuf::new(device, new_world_width_macro);
    resources.grid_stuff = GridRenderer::new(device, ...);
    
    Ok(())
}
```

**Recommendation**: Use **Option B** (deferred actions). It's the cleanest because:
1. Your UI functions stay simple (no `frame` parameter pollution)
2. All GPU operations happen in one place (`update()`)
3. It's easier to reason about when things happen

#### Architecture Diagram for Option B

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           App::update(ctx, frame)                           │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ 1. PROCESS PENDING ACTIONS (have frame access here)                   │  │
│  │    if let Some(path) = self.pending_open.take() {                     │  │
│  │        let wgpu_state = frame.wgpu_render_state();                    │  │
│  │        self.open_world_db(path, device, queue)?;                      │  │
│  │    }                                                                  │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ 2. RENDER UI (set pending actions, don't execute)                     │  │
│  │    TopBottomPanel::top(...).show(ctx, |ui| {                          │  │
│  │        self.render_menu_bar(ui);  // May set self.pending_open        │  │
│  │    });                                                                │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ 3. UPDATE GPU BUFFERS FOR RENDERING                                   │  │
│  │    if let Some(wgpu_state) = frame.wgpu_render_state() {              │  │
│  │        // Update camera, selection buffers, etc.                      │  │
│  │    }                                                                  │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ 4. SCHEDULE RENDER CALLBACK                                           │  │
│  │    ui.painter().add(egui_wgpu::Callback::new_paint_callback(...));    │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

This pattern ensures you always have `frame` access when you need to do GPU operations.

---

## Summary

The migration fundamentally changes how your app is structured:

1. **eframe owns the event loop** - You just implement the `eframe::App` trait
2. **egui handles input** - Use `Response` objects instead of manual event tracking
3. **Rendering is callback-based** - Insert your 3D rendering into egui's render pass via `CallbackTrait`
4. **Resources live in callback_resources** - A type-map that persists across frames

The main work is:
1. Refactoring `App` from `ApplicationHandler` to `eframe::App`
2. Converting your renderers to a `CallbackTrait` implementation
3. Moving GPU resource creation to `App::new(cc)` and storing in `callback_resources`
4. Replacing manual input handling with egui's response system
5. Handling the depth buffer situation (likely by just removing it)

Good luck with the migration! 🚀
