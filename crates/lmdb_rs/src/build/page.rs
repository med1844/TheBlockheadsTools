use crate::arch::Arch;
use crate::constants::{NODE_HEADER_SIZE, P_BRANCH, P_LEAF};
use std::marker::PhantomData;

/// Builder for creating leaf pages
pub struct LeafPageBuilder<'a, A: Arch> {
    buffer: Vec<u8>,
    page_size: usize,
    /// (key, value) pairs. Saved to write at build time.
    /// Writing immediately into buffer is tricky because nodes grow downwards
    /// and pointers grow upwards. So we buffer entries.
    /// (key, content, logical_size, flags).
    entries: Vec<(&'a [u8], EntryContent<'a>, usize, u16)>,
    _arch: PhantomData<A>,
}

pub enum EntryContent<'a> {
    Ref(&'a [u8]),
    Owned(usize, [u8; 8]),
}

impl<'a> EntryContent<'a> {
    fn len(&self) -> usize {
        match self {
            EntryContent::Ref(s) => s.len(),
            EntryContent::Owned(l, _) => *l,
        }
    }
}

impl<'a, A: Arch> LeafPageBuilder<'a, A> {
    pub fn new(page_size: usize) -> Self {
        Self {
            buffer: vec![0u8; page_size],
            page_size,
            entries: Vec::new(),
            _arch: PhantomData,
        }
    }

    /// Add entry, returns false if page is full.
    /// Returns true if added.
    pub fn push(&mut self, key: &'a [u8], value: &'a [u8]) -> bool {
        // Normal node: data size = value.len()
        self.push_internal(key, EntryContent::Ref(value), value.len(), 0)
    }

    /// Add sub-database entry with F_SUBDATA flag.
    /// Used for entries in the Main DB whose values are serialized DbRecords.
    pub fn push_subdata(&mut self, key: &'a [u8], value: &'a [u8]) -> bool {
        self.push_internal(
            key,
            EntryContent::Ref(value),
            value.len(),
            crate::constants::F_SUBDATA,
        )
    }

    /// Add overflow entry (F_BIGDATA).
    /// `full_size` is the logical size of the value.
    /// `overflow_pgno` is the page number where data starts.
    pub fn push_overflow(&mut self, key: &'a [u8], full_size: usize, overflow_pgno: u64) -> bool {
        let mut pgno_bytes = [0u8; 8];
        A::write_pgno(overflow_pgno, &mut pgno_bytes);
        // We store the full 8 bytes (or 4 needed) in Owned variant
        // and pass the actual valid length (A::PGNO_SIZE)
        self.push_internal(
            key,
            EntryContent::Owned(A::PGNO_SIZE, pgno_bytes),
            full_size,
            crate::constants::F_BIGDATA,
        )
    }

    fn push_internal(
        &mut self,
        key: &'a [u8],
        content: EntryContent<'a>,
        logical_size: usize,
        flags: u16,
    ) -> bool {
        // Calculate physical size needed (Key + Content (val or pgno))
        let content_len = content.len(); // use method

        let node_phys_size = self.calculate_node_size(key.len(), content_len);
        let current_entries = self.entries.len();

        let header_base = if A::PGNO_SIZE == 8 { 16 } else { 12 };

        let ptr_space = (current_entries + 1) * 2;
        let used_top = header_base + ptr_space;

        // Current used bottom: sum of all node sizes
        let used_bottom: usize = self
            .entries
            .iter()
            .map(|(k, v, _, _)| self.calculate_node_size(k.len(), v.len()))
            .sum::<usize>()
            + node_phys_size;

        if used_top + used_bottom > self.page_size {
            return false;
        }

        self.entries.push((key, content, logical_size, flags));
        true
    }

    fn calculate_node_size(&self, ksize: usize, dsize: usize) -> usize {
        // Node Header: 8 bytes (lo, hi, flags, ksize)
        // + Key + Data
        // + Alignment? LMDB nodes are 2-byte aligned.
        let raw_size = NODE_HEADER_SIZE + ksize + dsize;
        // Align to 2 bytes
        if !raw_size.is_multiple_of(2) {
            raw_size + 1
        } else {
            raw_size
        }
    }

    /// Finalize page, returns page bytes
    pub fn build(mut self, page_number: u64) -> Vec<u8> {
        // 1. Write Page Header
        // Layout depends on A::write_pgno etc.
        // Flags: P_LEAF
        let header_base = if A::PGNO_SIZE == 8 { 16 } else { 12 };

        // Write Pgno
        A::write_pgno(page_number, &mut self.buffer[0..]); // writes 4 or 8 bytes

        // Write Pad/Flags/PB
        // MDB_page layout:
        // Arch64: pgno(8) + pad(2) + flags(2) + lower(2) + upper(2)
        // Arch32: pgno(4) + pad(2) + flags(2) + lower(2) + upper(2)

        let ptr_end = (header_base + self.entries.len() * 2) as u32; // mp_lower

        // Calculate data offset
        let total_data_size: usize = self
            .entries
            .iter()
            .map(|(k, v, _, _)| self.calculate_node_size(k.len(), v.len()))
            .sum();
        let data_start = (self.page_size - total_data_size) as u32; // mp_upper

        let flags = P_LEAF;

        if A::PGNO_SIZE == 8 {
            // 64-bit: Flags at offset 10. Lower/Upper at 12, 14.

            let flags_off = 10;
            self.buffer[flags_off..flags_off + 2].copy_from_slice(&flags.to_le_bytes());

            let lower = ptr_end as u16;
            let upper = data_start as u16;
            self.buffer[12..14].copy_from_slice(&lower.to_le_bytes());
            self.buffer[14..16].copy_from_slice(&upper.to_le_bytes());
        } else {
            // 32-bit
            // +4: pad(2), flags(2)
            // +8: lower(2), upper(2)
            // Total 12 bytes.

            let flags_off = 6;
            self.buffer[flags_off..flags_off + 2].copy_from_slice(&flags.to_le_bytes());

            let lower = ptr_end as u16;
            let upper = data_start as u16;
            self.buffer[8..10].copy_from_slice(&lower.to_le_bytes());
            self.buffer[10..12].copy_from_slice(&upper.to_le_bytes());
        }

        // 2. Write Entries
        // Pointers grow up from header_base.
        // Nodes grow down from page_size.

        let mut current_data_offset = self.page_size;

        for (i, (key, value, logical_size, entry_flags)) in self.entries.iter().enumerate() {
            let ksize = key.len();
            let phys_dsize = value.len();
            let nsize = self.calculate_node_size(ksize, phys_dsize);

            current_data_offset -= nsize;
            let node_offset = current_data_offset;

            // Write Pointer
            let ptr_offset = header_base + i * 2;
            let offset_u16 = node_offset as u16;
            self.buffer[ptr_offset..ptr_offset + 2].copy_from_slice(&offset_u16.to_le_bytes());

            // Write Node at node_offset
            // Header: 8 bytes
            // lo(2), hi(2) -> logical data size (u32)
            // flags(2)
            // ksize(2)

            let dsize_u32 = *logical_size as u32;
            let lo = (dsize_u32 & 0xFFFF) as u16;
            let hi = ((dsize_u32 >> 16) & 0xFFFF) as u16;

            // Flags
            let n_flags: u16 = *entry_flags;
            let ksize_u16 = ksize as u16;

            // Only write header fields
            let h = &mut self.buffer[node_offset..node_offset + 8];
            h[0..2].copy_from_slice(&lo.to_le_bytes());
            h[2..4].copy_from_slice(&hi.to_le_bytes());
            h[4..6].copy_from_slice(&n_flags.to_le_bytes());
            h[6..8].copy_from_slice(&ksize_u16.to_le_bytes());

            // Write Key
            let k_start = node_offset + NODE_HEADER_SIZE;
            self.buffer[k_start..k_start + ksize].copy_from_slice(key);

            // Write Data
            let d_start = k_start + ksize;
            match value {
                EntryContent::Ref(s) => {
                    self.buffer[d_start..d_start + phys_dsize].copy_from_slice(s)
                }
                EntryContent::Owned(len, buf) => {
                    self.buffer[d_start..d_start + phys_dsize].copy_from_slice(&buf[0..*len])
                }
            }

            // Alignment padding is already handled by `current_data_offset` decrementing `nsize`
            // which was aligned. The bytes are written at start of that block.
        }

        self.buffer
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Builder for branch pages
pub struct BranchPageBuilder<'a, A: Arch> {
    buffer: Vec<u8>,
    page_size: usize,
    /// (key, child_pgno).
    /// Key is the separator (min key of child, usually).
    entries: Vec<(&'a [u8], u64)>,
    _arch: PhantomData<A>,
}

impl<'a, A: Arch> BranchPageBuilder<'a, A> {
    pub fn new(page_size: usize) -> Self {
        Self {
            buffer: vec![0u8; page_size],
            page_size,
            entries: Vec::new(),
            _arch: PhantomData,
        }
    }

    pub fn push(&mut self, key: &'a [u8], child_pgno: u64) -> bool {
        let node_size = self.calculate_node_size(key.len());

        let header_base = if A::PGNO_SIZE == 8 { 16 } else { 12 };
        let ptr_space = (self.entries.len() + 1) * 2;
        let used_top = header_base + ptr_space;

        let used_bottom: usize = self
            .entries
            .iter()
            .map(|(k, _)| self.calculate_node_size(k.len()))
            .sum::<usize>()
            + node_size;

        if used_top + used_bottom > self.page_size {
            return false;
        }

        self.entries.push((key, child_pgno));
        true
    }

    fn calculate_node_size(&self, ksize: usize) -> usize {
        // Branch Node: Header (8) + Key. No data.
        let raw_size = NODE_HEADER_SIZE + ksize;
        if !raw_size.is_multiple_of(2) {
            raw_size + 1
        } else {
            raw_size
        }
    }

    pub fn build(mut self, page_number: u64) -> Vec<u8> {
        let header_base = if A::PGNO_SIZE == 8 { 16 } else { 12 };
        A::write_pgno(page_number, &mut self.buffer[0..]);

        let ptr_end = (header_base + self.entries.len() * 2) as u32;
        let total_data_size: usize = self
            .entries
            .iter()
            .map(|(k, _)| self.calculate_node_size(k.len()))
            .sum();
        let data_start = (self.page_size - total_data_size) as u32;
        let flags = P_BRANCH;

        if A::PGNO_SIZE == 8 {
            // 64-bit
            let flags_off = 10;
            self.buffer[flags_off..flags_off + 2].copy_from_slice(&flags.to_le_bytes());
            let lower = ptr_end as u16;
            let upper = data_start as u16;
            self.buffer[12..14].copy_from_slice(&lower.to_le_bytes());
            self.buffer[14..16].copy_from_slice(&upper.to_le_bytes());
        } else {
            // 32-bit
            let flags_off = 6;
            self.buffer[flags_off..flags_off + 2].copy_from_slice(&flags.to_le_bytes());
            let lower = ptr_end as u16;
            let upper = data_start as u16;
            self.buffer[8..10].copy_from_slice(&lower.to_le_bytes());
            self.buffer[10..12].copy_from_slice(&upper.to_le_bytes());
        }

        let mut current_data_offset = self.page_size;

        for (i, (key, child_pgno)) in self.entries.iter().enumerate() {
            let ksize = key.len();
            let nsize = self.calculate_node_size(ksize);

            current_data_offset -= nsize;
            let node_offset = current_data_offset;

            // Pointer
            let ptr_offset = header_base + i * 2;
            let offset_u16 = node_offset as u16;
            self.buffer[ptr_offset..ptr_offset + 2].copy_from_slice(&offset_u16.to_le_bytes());

            // Node
            // lo, hi = low 32 bits of child pgno
            // flags = high 16 bits of child pgno (on 64-bit) or ignore (32-bit)
            // ksize = key len

            let lo = (child_pgno & 0xFFFF) as u16;
            let hi = ((child_pgno >> 16) & 0xFFFF) as u16;
            let high_bits = if A::PGNO_SIZE == 8 {
                ((child_pgno >> 32) & 0xFFFF) as u16
            } else {
                0
            };

            let ksize_u16 = ksize as u16;

            let h = &mut self.buffer[node_offset..node_offset + 8];
            h[0..2].copy_from_slice(&lo.to_le_bytes());
            h[2..4].copy_from_slice(&hi.to_le_bytes());
            h[4..6].copy_from_slice(&high_bits.to_le_bytes());
            h[6..8].copy_from_slice(&ksize_u16.to_le_bytes());

            // Write Key
            let k_start = node_offset + NODE_HEADER_SIZE;
            self.buffer[k_start..k_start + ksize].copy_from_slice(key);
        }

        self.buffer
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::{Arch64, DynArch};
    use crate::page::generic::Page;

    #[test]
    fn test_build_leaf_page() {
        let mut builder = LeafPageBuilder::<Arch64>::new(4096);
        assert!(builder.push(b"key1", b"val1"));
        assert!(builder.push(b"key2", b"val2"));

        // Finalize
        let buf = builder.build(100);

        // Parse back
        let page = Page::new(&buf, DynArch::Arch64).expect("Should parse");
        match page {
            Page::Leaf(l) => {
                assert_eq!(l.num_keys(), 2);
                let (k1, v1) = l.get_entry(0).unwrap();
                assert_eq!(k1, b"key1");
                assert_eq!(v1, b"val1");

                let (k2, v2) = l.get_entry(1).unwrap();
                assert_eq!(k2, b"key2");
                assert_eq!(v2, b"val2");
            }
            _ => panic!("Wrong page type"),
        }
    }

    #[test]
    fn test_build_branch_page() {
        let mut builder = BranchPageBuilder::<Arch64>::new(4096);
        assert!(builder.push(b"min", 200));
        assert!(builder.push(b"max", 300));

        let buf = builder.build(101);

        let page = Page::new(&buf, DynArch::Arch64).expect("Should parse");
        match page {
            Page::Branch(b) => {
                assert_eq!(b.num_keys(), 2);
                let node0 = b.get_node(0).unwrap();
                assert_eq!(node0.key(), b"min");
                assert_eq!(node0.branch_child_pgno(), 200);

                let node1 = b.get_node(1).unwrap();
                assert_eq!(node1.key(), b"max");
                assert_eq!(node1.branch_child_pgno(), 300);
            }
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_page_overflow_panic() {
        let mut builder = LeafPageBuilder::<Arch64>::new(4096);
        // This causes used_bottom > page_size, triggering panic on subtraction
        let result = builder.push(b"key", vec![0u8; 5000].as_slice());
        assert!(!result, "Should return false for oversized item");
    }
}
