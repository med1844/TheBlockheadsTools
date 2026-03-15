use crate::arch::Arch;
use crate::arch::DynArch;
use crate::constants::P_OVERFLOW;
use crate::page::header::PageHeader;
use crate::page::{PageError, PageResult as Result};
use std::borrow::Cow;

/// Handler for multi-page overflow data
pub struct OverflowReader<'a> {
    env_data: &'a [u8],
    page_size: usize,
    start_page: u64,
    total_size: usize,
    arch: DynArch,
}

impl<'a> OverflowReader<'a> {
    pub fn new(
        env_data: &'a [u8],
        page_size: usize,
        start_page: u64,
        total_size: usize,
        arch: DynArch,
    ) -> Result<Self> {
        // Basic bounds check for start page
        let start_offset = start_page as usize * page_size;
        if start_offset >= env_data.len() {
            return Err(PageError::UnexpectedEof {
                expected: start_offset + 16,
                available: env_data.len(),
            });
        }

        // Verify header of first page
        let header_slice = &env_data[start_offset..];
        if header_slice.len() < 16 {
            // Min header size
            return Err(PageError::UnexpectedEof {
                expected: 16,
                available: header_slice.len(),
            });
        }

        let header = PageHeader::new(header_slice);
        let flags = header.flags(arch);

        if (flags & P_OVERFLOW) == 0 {
            return Err(PageError::InvalidPageType {
                expected: P_OVERFLOW,
                found: flags,
            });
        }

        Ok(Self {
            env_data,
            page_size,
            start_page,
            total_size,
            arch,
        })
    }

    /// Header size helper
    fn header_size(&self) -> usize {
        match self.arch {
            DynArch::Arch32 => 12,
            DynArch::Arch64 => 16,
        }
    }

    /// Get contiguous slice if possible (no allocation)
    pub fn try_as_slice(&self) -> Option<&'a [u8]> {
        let start_offset = self.start_page as usize * self.page_size;
        let header_sz = self.header_size();
        let data_start = start_offset + header_sz;
        let data_end = data_start + self.total_size;

        if data_end > self.env_data.len() {
            // Should be error, but this method returns Option
            return None;
        }

        Some(&self.env_data[data_start..data_end])
    }

    /// Read all overflow data (may span multiple pages)
    /// Returns slice if contiguous (always true for LMDB mmap?), otherwise allocates (not needed here?)
    pub fn read(&self) -> Result<Cow<'a, [u8]>> {
        let slice = self.try_as_slice().ok_or(PageError::UnexpectedEof {
            expected: self.total_size,
            available: 0,
        })?;
        Ok(Cow::Borrowed(slice))
    }

    /// Read the number of pages spanned by this overflow record
    pub fn num_pages(&self) -> u32 {
        let start_offset = self.start_page as usize * self.page_size;
        match self.arch {
            DynArch::Arch32 => {
                // pb_pages at offset 8 (u32)
                crate::arch::Arch32::read_size(&self.env_data[start_offset + 8..]).unwrap() as u32
            }
            DynArch::Arch64 => {
                // pb_pages at offset 12 (u32)
                // Note: We use Arch32::read_size because pb_pages is always u32.
                crate::arch::Arch32::read_size(&self.env_data[start_offset + 12..]).unwrap() as u32
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overflow_read() {
        let arch = DynArch::Arch64;
        let page_size = 4096;
        let mut buf = vec![0u8; page_size * 3]; // 3 pages (Page 0 unused, Page 1 start, Page 2 cont)

        let start_page = 1;
        let offset = start_page * page_size;

        // Header
        // Flags P_OVERFLOW (0x04) at offset 10
        buf[offset + 10] = 0x04;

        let header_sz = 16;
        let data_len = 5000;

        let data_start = offset + header_sz;
        for i in 0..data_len {
            buf[data_start + i] = (i % 255) as u8;
        }

        // This test doesn't call num_pages, so offsets matter less unless new() checks them?
        // new() checks flags ONLY.

        let reader =
            OverflowReader::new(&buf, page_size, start_page as u64, data_len, arch).unwrap();
        let result = reader.read().unwrap();

        assert_eq!(result.len(), data_len);
    }

    #[test]
    fn test_overflow_span_reading() {
        use crate::constants::P_OVERFLOW;

        let arch = DynArch::Arch64;
        let page_size = 4096;
        let mut buf = vec![0u8; page_size * 4];

        let start_page = 1;
        let offset = start_page * page_size;

        // Setup Header for 64-bit
        // pgno at 0..8 (8 bytes)
        buf[offset..offset + 8].copy_from_slice(&1u64.to_le_bytes());
        // flags at 10..12
        buf[offset + 10..offset + 12].copy_from_slice(&P_OVERFLOW.to_le_bytes());

        // pb_pages at 12 (u32)
        let num_pages = 3u32;
        buf[offset + 12..offset + 16].copy_from_slice(&num_pages.to_le_bytes());

        let reader =
            OverflowReader::new(&buf, page_size, start_page as u64, 4096 * 3, arch).unwrap();

        assert_eq!(reader.num_pages(), 3);
    }
}
