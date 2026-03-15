use crate::arch::DynArch;
use crate::constants::{P_LEAF, P_LEAF2};
use crate::page::header::PageHeader;
use crate::page::node::Node;
use crate::page::{PageError, PageResult as Result};
use std::cmp::Ordering;

/// Zero-copy leaf page providing key-value iteration
#[derive(Clone)]
pub struct LeafPage<'a> {
    data: &'a [u8],
    header: PageHeader<'a>,
    arch: DynArch,
}

impl<'a> std::fmt::Debug for LeafPage<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LeafPage")
            .field("pgno", &self.header.page_number(self.arch).unwrap_or(0))
            .field("num_keys", &self.num_keys())
            .field("arch", &self.arch)
            .finish()
    }
}

impl<'a> LeafPage<'a> {
    /// Parse a leaf page from raw bytes
    pub fn new(data: &'a [u8], arch: DynArch) -> Result<Self> {
        // Minimal check: at least 12 bytes (32-bit header)
        if data.len() < 12 {
            return Err(PageError::UnexpectedEof {
                expected: 12,
                available: data.len(),
            });
        }

        let header = PageHeader::new(data);
        // Full check
        let required_header = PageHeader::header_size(arch);
        if data.len() < required_header {
            return Err(PageError::UnexpectedEof {
                expected: required_header,
                available: data.len(),
            });
        }

        let flags = header.flags(arch);

        if (flags & P_LEAF) == 0 && (flags & P_LEAF2) == 0 {
            return Err(PageError::InvalidPageType {
                expected: P_LEAF,
                found: flags,
            });
        }

        Ok(Self { data, header, arch })
    }

    pub fn header(&self) -> &PageHeader<'a> {
        &self.header
    }

    /// Number of keys in the page
    pub fn num_keys(&self) -> usize {
        self.header.num_keys(self.arch)
    }

    /// Get the raw offset for the node at index
    fn get_node_offset(&self, index: usize) -> Result<usize> {
        let num_keys = self.num_keys();
        if index >= num_keys {
            return Err(PageError::UnexpectedEof {
                expected: index,
                available: num_keys,
            });
        }

        // Use dynamic header size
        let ptr_offset = PageHeader::header_size(self.arch) + index * 2;
        if self.data.len() < ptr_offset + 2 {
            return Err(PageError::UnexpectedEof {
                expected: ptr_offset + 2,
                available: self.data.len(),
            });
        }

        let node_offset =
            u16::from_le_bytes(self.data[ptr_offset..ptr_offset + 2].try_into().unwrap()) as usize;

        if node_offset >= self.data.len() {
            return Err(PageError::UnexpectedEof {
                expected: node_offset,
                available: self.data.len(),
            });
        }

        Ok(node_offset)
    }

    /// Get node at index (0-based)
    pub fn get_node(&self, index: usize) -> Result<Node<'a>> {
        let offset = self.get_node_offset(index)?;
        Node::new(&self.data[offset..], self.arch)
    }

    /// Convenience: Get key and value at index
    pub fn get_entry(&self, index: usize) -> Result<(&'a [u8], &'a [u8])> {
        let node = self.get_node(index)?;
        Ok((node.key(), node.val_data().unwrap_or(b"")))
    }

    /// Iterator over all nodes
    pub fn iter(&self) -> LeafNodeIter<'a> {
        LeafNodeIter {
            page: self.clone(),
            index: 0,
            count: self.num_keys(),
        }
    }

    /// Binary search for key
    /// Returns Ok(index) if found, Err(index) where it should be inserted
    pub fn search(&self, key: &[u8]) -> std::result::Result<usize, usize> {
        let mut left = 0;
        let mut right = self.num_keys();

        while left < right {
            let mid = left + (right - left) / 2;
            let node = self.get_node(mid).map_err(|_| right)?;

            let node_key = node.key();
            match node_key.cmp(key) {
                Ordering::Equal => return Ok(mid),
                Ordering::Less => left = mid + 1,
                Ordering::Greater => right = mid,
            }
        }

        Err(left)
    }
}

pub struct LeafNodeIter<'a> {
    page: LeafPage<'a>,
    index: usize,
    count: usize,
}

impl<'a> Iterator for LeafNodeIter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }
        let node = self.page.get_node(self.index).ok()?;
        self.index += 1;
        Some(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::DynArch;

    // Helper to write a pointer at index
    fn write_ptr(buf: &mut [u8], index: usize, offset: u16) {
        // Assume Arch64 for tests unless specified
        let ptr_off = 16 + index * 2;
        buf[ptr_off..ptr_off + 2].copy_from_slice(&offset.to_le_bytes());
    }

    // Helper to write a simple Leaf Node at offset
    fn write_leaf_node(buf: &mut [u8], offset: usize, key: &[u8], val: &[u8]) -> usize {
        // Node Header: lo, hi, flags, ksize
        // Data Size = val.len()
        let dsize = val.len() as u32;
        let ksize = key.len() as u16;

        let lo = (dsize & 0xFFFF) as u16;
        let hi = ((dsize >> 16) & 0xFFFF) as u16;

        // At offset
        buf[offset] = lo as u8;
        buf[offset + 1] = (lo >> 8) as u8;
        buf[offset + 2] = hi as u8;
        buf[offset + 3] = (hi >> 8) as u8;

        // Flags = 0
        buf[offset + 4] = 0;
        buf[offset + 5] = 0;

        // Ksize
        buf[offset + 6] = ksize as u8;
        buf[offset + 7] = (ksize >> 8) as u8;

        // Key
        buf[offset + 8..offset + 8 + key.len()].copy_from_slice(key);

        // Val
        let val_off = offset + 8 + key.len();
        buf[val_off..val_off + val.len()].copy_from_slice(val);

        8 + key.len() + val.len()
    }

    #[test]
    fn test_leaf_page() {
        let arch = DynArch::Arch64;
        let mut buf = vec![0u8; 4096];

        // 1. Setup Page Header for 3 keys
        // mp_lower needs to point after pointers.
        // 3 keys = 3 pointers. 16 + 3*2 = 22.
        // So mp_lower = 22.

        // flags = P_LEAF (0x02) at offset 10 (Arch64)
        buf[10] = 0x02;

        // mp_lower at offset 12 (u16)
        buf[12] = 22;
        buf[13] = 0;

        // 2. Write Nodes (growing down from end of page is standard, but for test we can put them anywhere free)
        // Let's put them at 100, 200, 300.

        let off1 = 100;
        write_leaf_node(&mut buf, off1, b"a", b"valA");

        let off2 = 200;
        write_leaf_node(&mut buf, off2, b"b", b"valB");

        let off3 = 300;
        write_leaf_node(&mut buf, off3, b"c", b"valC");

        // 3. Write Pointers
        write_ptr(&mut buf, 0, off1 as u16);
        write_ptr(&mut buf, 1, off2 as u16);
        write_ptr(&mut buf, 2, off3 as u16);

        // Parse
        let page = LeafPage::new(&buf, arch).unwrap();
        assert_eq!(page.num_keys(), 3);

        // Check Iteration
        let keys: Vec<Vec<u8>> = page.iter().map(|n| n.key().to_vec()).collect();
        assert_eq!(keys, vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()]);

        // Search
        assert_eq!(page.search(b"a"), Ok(0));
        assert_eq!(page.search(b"b"), Ok(1));
        assert_eq!(page.search(b"c"), Ok(2));

        // Search missing
        assert_eq!(page.search(b"0"), Err(0)); // Before 'a'
        assert_eq!(page.search(b"bb"), Err(2)); // Between 'b' and 'c'
        assert_eq!(page.search(b"z"), Err(3)); // After 'c'
    }

    #[test]
    fn test_empty_page() {
        let arch = DynArch::Arch64;
        let mut buf = vec![0u8; 4096];

        // flags = P_LEAF at 10
        buf[10] = 0x02;
        // mp_lower = PAGE_HEADER_SIZE (16)
        buf[12] = 16;

        let page = LeafPage::new(&buf, arch).unwrap();
        assert_eq!(page.num_keys(), 0);
        assert_eq!(page.iter().count(), 0);
        assert_eq!(page.search(b"anything"), Err(0));
    }
    #[test]
    fn test_leaf_page_32() {
        let arch = DynArch::Arch32;
        let mut buf = vec![0u8; 4096];

        // 12-byte header.
        // P_LEAF (2) at offset 6.
        buf[6] = 2; // flags

        // mp_lower at offset 8 (u16 on 32-bit: flags at 6, lower at 8, upper at 10, ptrs at 12)
        // Let's set up 1 key.
        // mp_lower should be 12 + 1*2 = 14.
        buf[8] = 14;

        // Write pointer at index 0 (which is at offset 12).
        let ptr_off = 12;
        let node_off = 100u16;
        buf[ptr_off] = node_off as u8;
        buf[ptr_off + 1] = (node_off >> 8) as u8;

        // Write Node at 100
        let off = 100;
        write_leaf_node(&mut buf, off, b"x", b"val32");

        let page = LeafPage::new(&buf, arch).unwrap();
        assert_eq!(page.num_keys(), 1);

        // Search
        assert_eq!(page.search(b"x"), Ok(0));
        let node = page.get_node(0).unwrap();
        assert_eq!(node.key(), b"x");
        assert_eq!(node.val_data().unwrap(), b"val32");
    }
}
