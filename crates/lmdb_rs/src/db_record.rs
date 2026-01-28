use crate::arch::{DynArch, Arch, Arch32, Arch64};
use crate::error::{Result, Error};
use std::convert::TryInto;

/// Represents a named database record stored inside the Main DB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DbRecord {
    pub root_page: u64,
    pub flags: u16,
    pub depth: u16,
    pub branch_pages: u64,
    pub leaf_pages: u64,
    pub overflow_pages: u64,
    pub entries: u64,
    pub size: usize,
    pub pad: u32,
}

impl DbRecord {

    pub fn from_bytes(data: &[u8], arch: DynArch) -> Result<Self> {
        match arch {
            DynArch::Arch32 => Self::parse::<Arch32>(data),
            DynArch::Arch64 => Self::parse::<Arch64>(data),
        }
    }

    fn parse<A: Arch>(data: &[u8]) -> Result<Self> {
        // Validation: 4 (pad) + 2 (flags) + 2 (depth) + 4 * PGNO + 1 * SIZE
        let required = 8 + A::PGNO_SIZE * 4 + A::SIZE_T_SIZE;
        if data.len() < required {
            return Err(Error::UnexpectedEof { expected: required, available: data.len() });
        }

        let pad = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let flags = u16::from_le_bytes(data[4..6].try_into().unwrap());
        let depth = u16::from_le_bytes(data[6..8].try_into().unwrap());
        
        let mut offset = 8;
        
        let branch_pages = A::read_pgno(&data[offset..])?;
        offset += A::PGNO_SIZE;
        
        let leaf_pages = A::read_pgno(&data[offset..])?;
        offset += A::PGNO_SIZE;
        
        let overflow_pages = A::read_pgno(&data[offset..])?;
        offset += A::PGNO_SIZE;
        
        let entries = A::read_size(&data[offset..])?;
        offset += A::SIZE_T_SIZE;
        
        let root_page = A::read_pgno(&data[offset..])?;
        // offset += A::PGNO_SIZE; 
        
        Ok(DbRecord {
            flags, depth, branch_pages, leaf_pages, overflow_pages, entries, root_page,
            size: data.len(),
            pad,
        })
    }
}
