use crate::arch::DynArch;
use crate::constants::{P_BRANCH};
use crate::error::{Error, Result};
use crate::page::header::PageHeader;
use crate::page::node::Node;
use std::cmp::Ordering;

/// Zero-copy branch page (internal node of B+Tree)
#[derive(Clone)]
pub struct BranchPage<'a> {
    data: &'a [u8],
    header: PageHeader<'a>,
    arch: DynArch,
}

impl<'a> std::fmt::Debug for BranchPage<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BranchPage")
            .field("pgno", &self.header.page_number(self.arch).unwrap_or(0))
            .field("num_keys", &self.num_keys())
            .field("arch", &self.arch)
            .finish()
    }
}

impl<'a> BranchPage<'a> {
    /// Parse a branch page from raw bytes
    pub fn new(data: &'a [u8], arch: DynArch) -> Result<Self> {
        // Minimal check
        if data.len() < 12 {
            return Err(Error::UnexpectedEof { expected: 12, available: data.len() });
        }
        
        let header = PageHeader::new(data);
        // Full check
        let required_header = PageHeader::header_size(arch);
        if data.len() < required_header {
             return Err(Error::UnexpectedEof { expected: required_header, available: data.len() });
        }

        let flags = header.flags(arch);
        
        if (flags & P_BRANCH) == 0 {
             return Err(Error::InvalidPageType { expected: P_BRANCH, found: flags });
        }
        
        Ok(Self {
            data,
            header,
            arch,
        })
    }

    pub fn header(&self) -> &PageHeader<'a> {
        &self.header
    }

    /// Number of keys (child pointers) in the page
    pub fn num_keys(&self) -> usize {
        self.header.num_keys(self.arch)
    }
    
    fn get_node_offset(&self, index: usize) -> Result<usize> {
        let num_keys = self.num_keys();
        if index >= num_keys {
             return Err(Error::UnexpectedEof { expected: index, available: num_keys });
        }
        
        // Use dynamic header size for pointer offset
        let ptr_offset = PageHeader::header_size(self.arch) + index * 2;
        if self.data.len() < ptr_offset + 2 {
             return Err(Error::UnexpectedEof { expected: ptr_offset + 2, available: self.data.len() });
        }
        
        let node_offset = u16::from_le_bytes(self.data[ptr_offset..ptr_offset+2].try_into().unwrap()) as usize;
        
        if node_offset >= self.data.len() {
             return Err(Error::UnexpectedEof { expected: node_offset, available: self.data.len() });
        }
        
        Ok(node_offset)
    }

    /// Get node at index
    pub fn get_node(&self, index: usize) -> Result<Node<'a>> {
        let offset = self.get_node_offset(index)?;
        Node::new(&self.data[offset..], self.arch)
    }

    /// Find child page for given key.
    /// Returns (child_page_number, index_in_this_page)
    pub fn search(&self, key: &[u8]) -> Result<(u64, usize)> {
        let nkeys = self.num_keys();
        if nkeys == 0 {
            // Should not happen for valid branch page (min 2 children usually, or 1)
            return Err(Error::CorruptedTree { message: "Empty branch page" });
        }
        
        // Binary search
        let mut left = 0;
        let mut right = nkeys;
        // Result: index of first node where node.key >= key
        
        let mut exact_match = false;
        
        while left < right {
            let mid = left + (right - left) / 2;
            let node = self.get_node(mid).map_err(|_| Error::UnexpectedEof { expected: 0, available: 0 })?; 
            
            let node_key = node.key();
            match node_key.cmp(key) {
                Ordering::Equal => {
                    left = mid;
                    exact_match = true;
                    break;
                },
                Ordering::Less => left = mid + 1,
                Ordering::Greater => right = mid,
            }
        }
        
        // 'left' is now the insertion point or exact match.
        // If exact match (node.key == key), we follow this node (index=left).
        // If not exact match (node.key > key), we follow the previous node (index=left-1).
        
        let mut index = left;
        if !exact_match {
            if index > 0 {
                index -= 1;
            } else {
                // Key is smaller than the first node's key.
                // But Node[0] is usually the "leftmost" pointer with implied -infinity key.
                // LMDB structure: Node[0] usually has empty key or is treated as minimal.
                // If Node[0].key > key, checks might fail unless Node[0] key is empty.
                // We default to 0.
                index = 0;
            }
        }
        
        let node = self.get_node(index)?;
        Ok((node.branch_child_pgno(), index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::DynArch;

    // Helper to write a pointer at index
    fn write_ptr(buf: &mut [u8], index: usize, offset: u16) {
        // Assume Arch64 for test unless specialized
        let ptr_off = 16 + index * 2;
        buf[ptr_off..ptr_off+2].copy_from_slice(&offset.to_le_bytes());
    }

    // Helper to write a branch node
    fn write_branch_node(buf: &mut [u8], offset: usize, key: &[u8], pgno: u64) -> usize {
        // Node Header: lo, hi, flags, ksize. NO DATA.
        // For Branch, lo/hi/flags encode Child Pgno.
        let lo = (pgno & 0xFFFF) as u16;
        let hi = ((pgno >> 16) & 0xFFFF) as u16;
        let flags = ((pgno >> 32) & 0xFFFF) as u16; // Flags field holds high bits on 64-bit
        let ksize = key.len() as u16;
        
        buf[offset] = lo as u8;
        buf[offset+1] = (lo >> 8) as u8;
        buf[offset+2] = hi as u8;
        buf[offset+3] = (hi >> 8) as u8;
        
        buf[offset+4] = flags as u8;
        buf[offset+5] = (flags >> 8) as u8;
        
        buf[offset+6] = ksize as u8;
        buf[offset+7] = (ksize >> 8) as u8;
        
        // Key
        buf[offset+8..offset+8+key.len()].copy_from_slice(key);
        
        8 + key.len()
    }

    #[test]
    fn test_branch_search() {
        let arch = DynArch::Arch64;
        let mut buf = vec![0u8; 4096];
        
        // 1. Setup Page Header
        // 3 keys = 3 pointers. 
        // 16 + 3*2 = 22. mp_lower = 22.
        // P_BRANCH = 0x01. Offset 10.
        buf[10] = 0x01;
        buf[12] = 22;
        
        // 2. Write Nodes
        // Node 0: pgno=100, key="" (empty)
        let off0 = 100;
        write_branch_node(&mut buf, off0, b"", 100);
        
        // Node 1: pgno=200, key="m"
        let off1 = 120;
        write_branch_node(&mut buf, off1, b"m", 200);
        
        // Node 2: pgno=300, key="z"
        let off2 = 140;
        write_branch_node(&mut buf, off2, b"z", 300);
        
        // 3. Pointers
        write_ptr(&mut buf, 0, off0 as u16);
        write_ptr(&mut buf, 1, off1 as u16);
        write_ptr(&mut buf, 2, off2 as u16);
        
        let page = BranchPage::new(&buf, arch).unwrap();
        assert_eq!(page.num_keys(), 3);
        
        // Search "a" -> Should go to child 0 (pgno 100) because "a" < "m"
        let (pg, idx) = page.search(b"a").unwrap();
        assert_eq!(pg, 100);
        assert_eq!(idx, 0);
        
        // Search "m" -> Exact match Node 1 ("m"). Value >= "m". Pgno 200.
        let (pg, idx) = page.search(b"m").unwrap();
        assert_eq!(pg, 200);
        assert_eq!(idx, 1);
        
        // Search "n" -> "n" > "m" but < "z". Value >= "m". Pgno 200.
        let (pg, idx) = page.search(b"n").unwrap();
        assert_eq!(pg, 200);
        assert_eq!(idx, 1);
        
        // Search "z" -> Exact match Node 2. Pgno 300.
        let (pg, idx) = page.search(b"z").unwrap();
        assert_eq!(pg, 300);
        assert_eq!(idx, 2);
        
        // Search "{" (ascii 123, z is 122) -> Greater than "z". Pgno 300.
        let (pg, idx) = page.search(b"{").unwrap();
        assert_eq!(pg, 300);
        assert_eq!(idx, 2);
    }
}
