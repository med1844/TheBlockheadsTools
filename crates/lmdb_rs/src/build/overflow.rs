use crate::arch::Arch;
use crate::constants::P_OVERFLOW;
use std::marker::PhantomData;

/// Builder for overflow pages (large values)
pub struct OverflowBuilder<A: Arch> {
    page_size: usize,
    _arch: PhantomData<A>,
}

impl<A: Arch> OverflowBuilder<A> {
    pub fn new(page_size: usize) -> Self {
        Self {
            page_size,
            _arch: PhantomData,
        }
    }

    /// Calculate number of pages needed for data
    /// Overflow header:
    ///  pgno(4/8) + pad/flags/pb(8/8) + ptrs(0) + data
    /// Note: Only the first page contains a header. Subsequent pages are raw data.
    ///
    /// Header Size:
    /// Arch32: 12 bytes. Arch64: 16 bytes.
    /// Capacity of Page 0: page_size - header_size.
    /// Capacity of Page N: page_size.
    pub fn pages_needed(&self, data_size: usize) -> usize {
        let header_size = if A::PGNO_SIZE == 8 { 16 } else { 12 };
        if data_size <= (self.page_size - header_size) {
            return 1;
        }

        let remaining = data_size - (self.page_size - header_size);
        // ceil(remaining / page_size) + 1
        1 + remaining.div_ceil(self.page_size)
    }

    /// Build overflow pages, returns Vec of page buffers
    /// P_OVERFLOW pages span multiple pages.
    /// `start_page`: page number of the first page.
    pub fn build(&self, data: &[u8], start_page: u64) -> Vec<Vec<u8>> {
        let num_pages = self.pages_needed(data.len());
        let mut pages = Vec::with_capacity(num_pages);

        let header_size = if A::PGNO_SIZE == 8 { 16 } else { 12 };

        // 1. Build First Page
        let mut first_page = vec![0u8; self.page_size];

        // Write Header
        A::write_pgno(start_page, &mut first_page[0..]);

        // Flags
        let flags = P_OVERFLOW;
        if A::PGNO_SIZE == 8 {
            // 64-bit
            let flags_off = 10;
            first_page[flags_off..flags_off + 2].copy_from_slice(&flags.to_le_bytes());

            // pb_pages (u32) is stored at offset 12 (after pad+flags) for 64-bit.
            let pb_pages = num_pages as u32;
            first_page[12..16].copy_from_slice(&pb_pages.to_le_bytes());
        } else {
            // 32-bit
            // +6: flags
            let flags_off = 6;
            first_page[flags_off..flags_off + 2].copy_from_slice(&flags.to_le_bytes());

            // pb_pages at +8
            let pb_pages = num_pages as u32;
            first_page[8..12].copy_from_slice(&pb_pages.to_le_bytes());
        }

        // Write Data Chunk 1
        let capacity0 = self.page_size - header_size;
        let chunk0_len = std::cmp::min(data.len(), capacity0);
        first_page[header_size..header_size + chunk0_len].copy_from_slice(&data[0..chunk0_len]);

        pages.push(first_page);

        let mut current_data_offset = chunk0_len;

        // 2. Build Subsequent Pages (Raw Data)
        for _ in 1..num_pages {
            let mut page = vec![0u8; self.page_size];
            let remaining = data.len() - current_data_offset;
            let chunk_len = std::cmp::min(remaining, self.page_size);

            page[0..chunk_len]
                .copy_from_slice(&data[current_data_offset..current_data_offset + chunk_len]);
            current_data_offset += chunk_len;

            pages.push(page);
        }

        pages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::{Arch64, DynArch};
    use crate::page::generic::Page;

    #[test]
    fn test_overflow_build_arch64() {
        let page_size = 4096;
        let builder = OverflowBuilder::<Arch64>::new(page_size);

        // Header (16)
        // Cap0 = 4096 - 16 = 4080
        // Data = 5000
        // Page 0: 4080 bytes. Rem = 920.
        // Page 1: 920 bytes.
        // Total pages = 2.

        let data = vec![0xAAu8; 5000];
        let pages = builder.build(&data, 100);

        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].len(), 4096);
        assert_eq!(pages[1].len(), 4096);

        // Verify Header P0
        let p0 = Page::new(&pages[0], DynArch::Arch64).unwrap();
        match p0 {
            Page::Overflow(ref h) => {
                assert_eq!(h.flags(DynArch::Arch64) & P_OVERFLOW, P_OVERFLOW);
                let num = p0.overflow_pages(DynArch::Arch64).unwrap();
                assert_eq!(num, 2);
            }
            _ => panic!("Not overflow"),
        }

        // Verify Data Continuity
        // P0 Data starts at 16
        assert_eq!(&pages[0][16..], &data[0..4080]);
        // P1 Data starts at 0
        assert_eq!(&pages[1][0..920], &data[4080..]);
        // Remaining p1 should be 0
        assert_eq!(pages[1][920], 0);
    }
}
