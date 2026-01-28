use crate::arch::{Arch, Arch32, Arch64, DynArch};
use crate::constants::MDB_MAGIC;
use crate::error::{Error, Result};
use crate::page::header::PageHeader;
use std::convert::TryInto;

use crate::db_record::DbRecord;

/// Zero-copy reference to a meta page
#[derive(Debug, Clone)]
pub struct MetaPage<'a> {
    data: &'a [u8],
    header: PageHeader<'a>,
    arch: DynArch,
}

impl<'a> MetaPage<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self> {
        let (page, _) = Self::parse(data)?;
        Ok(page)
    }

    pub fn header(&self) -> &PageHeader<'a> {
        &self.header
    }

    /// Parse meta page, auto-detecting architecture
    pub fn parse(data: &'a [u8]) -> Result<(Self, DynArch)> {
        if data.len() < 64 {
            return Err(Error::UnexpectedEof {
                expected: 64,
                available: data.len(),
            });
        }

        // Strategy:
        // 1. Check for magic number at offset 12 (implies 32-bit header) - Check this FIRST
        // 2. Check for magic number at offset 16 (implies 64-bit header or 16-byte aligned 32-bit)

        let magic16 = u32::from_le_bytes(data[16..20].try_into().unwrap());
        let magic12 = u32::from_le_bytes(data[12..16].try_into().unwrap());

        let (offset, arch) = if magic12 == MDB_MAGIC {
            let _flags_at_6 = u16::from_le_bytes(data[6..8].try_into().unwrap());
            (12, DynArch::Arch32)
        } else if magic16 == MDB_MAGIC {
            // Likely 64-bit or 16-byte aligned.
            // Check flags to confirm arch?
            // On 64-bit, flags at 10. pattern at 10 should be P_META (0x08).
            let flags_at_10 = u16::from_le_bytes(data[10..12].try_into().unwrap());

            if (flags_at_10 & crate::constants::P_META) != 0 {
                (16, DynArch::Arch64)
            } else {
                let flags_at_6 = u16::from_le_bytes(data[6..8].try_into().unwrap());

                if (flags_at_6 & crate::constants::P_META) != 0 {
                    (16, DynArch::Arch32)
                } else {
                    // Default to 64? Or error?
                    // If flags are garbage, we can't be sure.
                    (16, DynArch::Arch64)
                }
            }
        } else {
            return Err(Error::InvalidMagic {
                expected: MDB_MAGIC,
                found: magic16,
            });
        };

        // Verify version
        let version = u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
        if version != 1 {
            return Err(Error::UnsupportedVersion { version });
        }

        let header = PageHeader::new(data);

        Ok((
            MetaPage {
                data: &data[offset..],
                header,
                arch,
            },
            arch,
        ))
    }

    pub fn magic(&self) -> u32 {
        u32::from_le_bytes(self.data[0..4].try_into().unwrap())
    }

    pub fn version(&self) -> u32 {
        u32::from_le_bytes(self.data[4..8].try_into().unwrap())
    }

    pub fn page_size(&self) -> u32 {
        self.free_db().pad
    }

    pub fn last_page(&self) -> u64 {
        match self.arch {
            DynArch::Arch32 => {
                // 32-bit: mm_dbs[1] end = 16 + 28*2 = 72.
                // mm_last_pg at 72.
                Arch32::read_pgno(&self.data[72..]).unwrap()
            }
            DynArch::Arch64 => {
                // 64-bit: mm_dbs[1] end = 24 + 48*2 = 120.
                // mm_last_pg at 120.
                Arch64::read_pgno(&self.data[120..]).unwrap()
            }
        }
    }

    pub fn txn_id(&self) -> u64 {
        match self.arch {
            DynArch::Arch32 => {
                // mm_txnid at 76 (72 + 4)
                Arch32::read_size(&self.data[76..]).unwrap()
            }
            DynArch::Arch64 => {
                // mm_txnid at 128 (120 + 8)
                Arch64::read_size(&self.data[128..]).unwrap()
            }
        }
    }

    pub fn free_db(&self) -> DbRecord {
        match self.arch {
            DynArch::Arch32 => DbRecord::from_bytes(&self.data[16..], DynArch::Arch32).unwrap(),
            DynArch::Arch64 => DbRecord::from_bytes(&self.data[24..], DynArch::Arch64).unwrap(),
        }
    }

    pub fn main_db(&self) -> DbRecord {
        match self.arch {
            DynArch::Arch32 => {
                // mm_dbs[1] starts at 16 + 28 = 44
                DbRecord::from_bytes(&self.data[44..], DynArch::Arch32).unwrap()
            }
            DynArch::Arch64 => {
                // mm_dbs[1] starts at 24 + 48 = 72
                DbRecord::from_bytes(&self.data[72..], DynArch::Arch64).unwrap()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_meta_32(buf: &mut [u8], magic: u32, version: u32, psize: u32) {
        buf[6] = 0x08;
        let offset = 16;
        buf[offset + 0..offset + 4].copy_from_slice(&magic.to_le_bytes());
        buf[offset + 4..offset + 8].copy_from_slice(&version.to_le_bytes());
        buf[offset + 16..offset + 20].copy_from_slice(&psize.to_le_bytes());
    }

    fn write_meta_64(buf: &mut [u8], magic: u32, version: u32, psize: u32) {
        buf[10] = 0x08;
        let offset = 16;
        buf[offset + 0..offset + 4].copy_from_slice(&magic.to_le_bytes());
        buf[offset + 4..offset + 8].copy_from_slice(&version.to_le_bytes());
        buf[offset + 24..offset + 28].copy_from_slice(&psize.to_le_bytes());
    }

    #[test]
    fn test_parse_meta_32() {
        let mut data = [0u8; 512];
        write_meta_32(&mut data, MDB_MAGIC, 1, 4096);
        let offset = 16;
        data[offset + 12..offset + 16].copy_from_slice(&(10 * 1024 * 1024 as u32).to_le_bytes());

        let (meta, arch) = MetaPage::parse(&data).unwrap();
        assert_eq!(arch, DynArch::Arch32);
        assert_eq!(meta.magic(), MDB_MAGIC);
    }

    #[test]
    fn test_parse_meta_64() {
        let mut data = [0u8; 512];
        write_meta_64(&mut data, MDB_MAGIC, 1, 4096);
        let (meta, arch) = MetaPage::parse(&data).unwrap();
        assert_eq!(arch, DynArch::Arch64);
        assert_eq!(meta.magic(), MDB_MAGIC);
    }

    #[test]
    fn test_invalid_magic() {
        let mut data = [0u8; 512];
        write_meta_32(&mut data, 0xBADF00D, 1, 4096);
        assert!(matches!(
            MetaPage::parse(&data),
            Err(Error::InvalidMagic { .. })
        ));
    }

    #[test]
    fn test_meta_offsets_64() {
        // Reproduce issue where offsets were wrong (72 vs 48 byte struct stride)
        let mut data = [0u8; 512];

        // Header
        data[10] = 0x08; // P_META

        // Meta struct at 16
        // Magic
        let offset = 16;
        data[offset..offset + 4].copy_from_slice(&MDB_MAGIC.to_le_bytes());
        data[offset + 4..offset + 8].copy_from_slice(&1u32.to_le_bytes());

        // Free DB at 24. Size 48 bytes.
        // Main DB at 24 + 48 = 72. Size 48 bytes.
        // Last Page at 24 + 48 + 48 = 120.
        // Txn ID at 120 + 8 = 128.

        // Write expected values at CORRECT offsets

        // Main DB: Root Page at offset 72 + 40 (offset of root_page in DbRecord) = 112
        // DbRecord layout: pad(4)+flags(2)+depth(2)+branch(8)+leaf(8)+overflow(8)+entries(8)+root(8)
        // root is at offset 40 within struct.
        let main_db_start = 24 + 48; // 72
        let root_offset = main_db_start + 40; // 112
        // Write root page 999
        data[offset + root_offset..offset + root_offset + 8].copy_from_slice(&999u64.to_le_bytes());

        // Last Page at 120
        data[offset + 120..offset + 120 + 8].copy_from_slice(&888u64.to_le_bytes());

        // Txn ID at 128
        data[offset + 128..offset + 128 + 8].copy_from_slice(&777u64.to_le_bytes());

        let (meta, _) = MetaPage::parse(&data).unwrap();

        // These assertions should FAIL if implementation uses wrong offsets
        assert_eq!(meta.last_page(), 888, "Last page offset mismatch");
        assert_eq!(meta.txn_id(), 777, "Txn ID offset mismatch");
        assert_eq!(
            meta.main_db().root_page,
            999,
            "Main DB Root Page offset mismatch"
        );
    }
}
