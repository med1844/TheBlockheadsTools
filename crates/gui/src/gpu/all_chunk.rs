use egui_wgpu::wgpu::{self, util::DeviceExt};
use the_blockheads_tools_lib::{BhError, BlockType};

type BlockIdType = u16;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct UVMappedBlockType(BlockIdType);

impl From<BlockIdType> for UVMappedBlockType {
    fn from(value: BlockIdType) -> Self {
        Self(value)
    }
}

impl UVMappedBlockType {
    // [PX, NX, PY, NY, PZ, NZ]
    pub(crate) const UV_AT_FACE: [[u32; 6]; 4] = [
        [0; 6],                        // air,
        [32; 6],                       // stone,
        [160, 160, 161, 64, 160, 160], // grass
        [65; 6],                       // sand,
    ];
}

impl TryFrom<BlockType> for UVMappedBlockType {
    type Error = BhError;
    fn try_from(value: BlockType) -> Result<Self, Self::Error> {
        Ok(Self(match value {
            BlockType::Stone => 1,
            BlockType::Sand => 3,
            BlockType::GrassDirt | BlockType::Dirt => 2,
            _ => 0,
        }))
    }
}

// Contains flattened block type
pub(crate) struct AllChunk {
    pub(crate) block_types: Vec<UVMappedBlockType>,
}

impl AllChunk {
    // Costly function - only call this once or VRAM nuked
    pub(crate) fn create_buffer_init(&self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Voxel Data Buffer"),
            contents: bytemuck::cast_slice(&self.block_types),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }
}
