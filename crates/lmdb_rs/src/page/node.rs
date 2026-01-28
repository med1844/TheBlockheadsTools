use crate::arch::DynArch;
use crate::constants::{F_BIGDATA, F_DUPDATA, F_SUBDATA, NODE_HEADER_SIZE};
use crate::error::{Error, Result};
use std::convert::TryInto;
use std::fmt::Debug;

/// Zero-copy node reference within a page
#[derive(Clone)]
pub struct Node<'a> {
    data: &'a [u8],
    arch: DynArch,
}

impl<'a> Debug for Node<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("leaf_flags", &self.leaf_flags())
            .field("key_size", &self.key_size())
            .field("leaf_data_size", &self.leaf_data_size())
            .field("branch_child_pgno", &self.branch_child_pgno())
            .finish()
    }
}

impl<'a> Node<'a> {
    /// Create a new Node from a byte slice. 
    /// The slice should start at the beginning of the node.
    pub fn new(data: &'a [u8], arch: DynArch) -> Result<Self> {
        if data.len() < NODE_HEADER_SIZE {
            return Err(Error::UnexpectedEof {
                expected: NODE_HEADER_SIZE,
                available: data.len(),
            });
        }
        Ok(Self { data, arch })
    }

    /// Low 16 bits of data size or page number
    fn lo(&self) -> u16 {
        u16::from_le_bytes(self.data[0..2].try_into().unwrap())
    }

    /// High 16 bits of data size or page number
    fn hi(&self) -> u16 {
        u16::from_le_bytes(self.data[2..4].try_into().unwrap())
    }

    /// Node flags (Valid only for Leaf nodes)
    /// On Branch nodes, this field contains high bits of the child page number.
    pub fn leaf_flags(&self) -> u16 {
        u16::from_le_bytes(self.data[4..6].try_into().unwrap())
    }

    /// Reconstruct child page number (Valid only for Branch nodes)
    /// On Arch64, this includes high bits from the 'flags' field.
    pub fn branch_child_pgno(&self) -> u64 {
        let lo = self.lo() as u64;
        let hi = self.hi() as u64;
        let pgnol = lo | (hi << 16);
        
        match self.arch {
            DynArch::Arch32 => pgnol,
            DynArch::Arch64 => {
                 let high = u16::from_le_bytes(self.data[4..6].try_into().unwrap()) as u64;
                 // PGNO_TOPWORD = 32 on standard 64-bit
                 pgnol | (high << 32)
            }
        }
    }
    
    pub fn key_size(&self) -> usize {
        u16::from_le_bytes(self.data[6..8].try_into().unwrap()) as usize
    }

    pub fn key(&self) -> &'a [u8] {
        let ksize = self.key_size();
        &self.data[NODE_HEADER_SIZE..NODE_HEADER_SIZE + ksize]
    }

    /// Access value bytes (Only for Leaf pages!)
    /// For F_BIGDATA, this returns the bytes containing the pgno (not the overflow data itself).
    pub fn val_data(&self) -> Option<&'a [u8]> {
        // Warning: meaningless for Branch nodes
        let ksize = self.key_size();
        let dsize = self.leaf_data_size();
        let offset = NODE_HEADER_SIZE + ksize;
        if self.data.len() < offset + dsize {
            return None; // Or handle error?
        }
        Some(&self.data[offset..offset + dsize])
    }

    /// Data size (Only for Leaf pages!)
    pub fn leaf_data_size(&self) -> usize {
        // On Leaf nodes, lo/hi always encode data size.
        (self.lo() as usize) | ((self.hi() as usize) << 16)
    }

    /// Alias for backwards compatibility or clarity
    pub fn data_size(&self) -> usize {
        self.leaf_data_size()
    }
    pub fn child_page_number(&self) -> u64 {
        self.branch_child_pgno()
    }
    
    /// Get the overflow page number (if F_BIGDATA is set)
    pub fn overflow_page_number(&self) -> Option<u64> {
        if !self.is_bigdata() {
            return None;
        }
        
        let offset = NODE_HEADER_SIZE + self.key_size(); // Calculate data_offset explicitly
        // The pgno is at the beginning of the data section
        if self.data.len() < offset + 4 { // Min pgno size
             return None;
        }
        
        match self.arch {
            DynArch::Arch32 => {
                 use crate::arch::Arch32;
                 use crate::arch::Arch;
                 Arch32::read_pgno(&self.data[offset..]).ok()
            },
            DynArch::Arch64 => {
                 use crate::arch::Arch64;
                 use crate::arch::Arch;
                 // Ensure we have 8 bytes
                 if self.data.len() < offset + 8 {
                     return None;
                 }
                 Arch64::read_pgno(&self.data[offset..]).ok()
            }
        }
    }

    pub fn is_bigdata(&self) -> bool {
        (self.leaf_flags() & F_BIGDATA) != 0
    }

    pub fn is_subdata(&self) -> bool {
        (self.leaf_flags() & F_SUBDATA) != 0
    }

    pub fn is_dupdata(&self) -> bool {
        (self.leaf_flags() & F_DUPDATA) != 0
    }

    /// Total size of this node in bytes (header + key + data)
    pub fn total_size(&self) -> usize {
        0 // Placeholder
    }
    
    /// Calculate size assuming it is a Leaf node
    pub fn leaf_size(&self) -> usize {
        let ksize = self.key_size();
        let dsize = self.data_size();
        // Round up to even address? LMDB nodes are 2-byte aligned usually?
        // "nodes are 2-byte aligned" - mdb.c
        // The size returned here is the used bytes. 
        // Iterate pointers usually handles alignment.
        NODE_HEADER_SIZE + ksize + dsize
    }
    
    /// Calculate size assuming it is a Branch node
    pub fn branch_size(&self) -> usize {
        let ksize = self.key_size();
        NODE_HEADER_SIZE + ksize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::DynArch;
    
    #[test]
    fn test_leaf_node_parse() {
        // Construct a leaf node with "key" and "val"
        // Header: lo, hi (size=3), flags=0, ksize=3
        let arch = DynArch::Arch64;
        let mut buf = vec![0u8; 32];
        
        // Lo/Hi = data_size = 3
        buf[0] = 3; 
        buf[1] = 0;
        buf[2] = 0;
        buf[3] = 0;
        
        // Flags = 0
        buf[4] = 0;
        buf[5] = 0;
        
        // Ksize = 3
        buf[6] = 3;
        buf[7] = 0;
        
        // Key "key"
        buf[8] = b'k'; buf[9] = b'e'; buf[10] = b'y';
        
        // Val "val"
        buf[11] = b'v'; buf[12] = b'a'; buf[13] = b'l';
        
        let node = Node::new(&buf, arch).unwrap();
        assert_eq!(node.key(), b"key");
        assert_eq!(node.val_data().unwrap(), b"val");
        assert_eq!(node.data_size(), 3);
        assert_eq!(node.key_size(), 3);
        assert!(!node.is_bigdata());
    }

    #[test]
    fn test_branch_node_parse_64() {
        // Construct a branch node pointing to page 0x100000005 (bit 32 set)
        // MDB_node only supports 48 bits of pgno on 64-bit (lo+hi+flags)
        // Header: lo/hi = low 32 bits, flags = high 16 bits (bits 32..48)
        let arch = DynArch::Arch64;
        let mut buf = vec![0u8; 32];
        
        let large_pgno: u64 = 0x1_0000_0005; 
        // low 32: 5
        // high 16 (bits 32..48): 1
        
        let lo = (large_pgno & 0xFFFF) as u16;
        let hi = ((large_pgno >> 16) & 0xFFFF) as u16;
        let high_part = ((large_pgno >> 32) & 0xFFFF) as u16;
        
        buf[0..2].copy_from_slice(&lo.to_le_bytes());
        buf[2..4].copy_from_slice(&hi.to_le_bytes());
        buf[4..6].copy_from_slice(&high_part.to_le_bytes());
        
        // Ksize = 3
        buf[6] = 3;
        buf[7] = 0;
        
        // Key "key"
        buf[8] = b'k'; buf[9] = b'e'; buf[10] = b'y';
        
        let node = Node::new(&buf, arch).unwrap();
        assert_eq!(node.branch_child_pgno(), large_pgno);
        assert_eq!(node.key(), b"key");
    }
    
    #[test]
    fn test_overflow_node_64() {
        // F_BIGDATA node. Flags has 0x01. 
        // Data size = 8 (sizeof pgno). 
        // Data content = pgno.
        let arch = DynArch::Arch64;
        let mut buf = vec![0u8; 32];
        let pgno: u64 = 999;
        
        // Data Size = 8
        buf[0] = 8;
        
        // Flags = F_BIGDATA (0x01)
        buf[4] = 0x01;
        
        // Ksize = 3
        buf[6] = 3;
        
        // Key "key"
        buf[8] = b'k'; buf[9] = b'e'; buf[10] = b'y';
        
        // Overflow PGNO at offset 8+3 = 11.
        buf[11..19].copy_from_slice(&pgno.to_le_bytes());
        
        let node = Node::new(&buf, arch).unwrap();
        assert!(node.is_bigdata());
        assert_eq!(node.key(), b"key");
        assert_eq!(node.overflow_page_number().unwrap(), pgno);
    }
    #[test]
    fn test_branch_node_32_with_garbage_flags() {
        // 32-bit Branch Node.
        // On 32-bit, flags/high-bits are ignored for pgno.
        // But if they contain garbage (e.g. 0x1000), Arch64 would interpret them as high bits.
        
        let arch = DynArch::Arch32;
        let mut buf = vec![0u8; 32];
        
        // lo = 5, hi = 0
        buf[0] = 5;
        buf[1] = 0;
        buf[2] = 0;
        buf[3] = 0;
        
        // Flags = 0x1000 (garbage) at offset 4
        // 0x1000 -> 0x00 0x10 (LE) ? No. 0x1000 = 4096. 
        // 0x10 << 8. 
        // byte 4 = 0x00. byte 5 = 0x10.
        buf[4] = 0x00;
        buf[5] = 0x10;
        
        // Ksize = 3
        buf[6] = 3;
        
        // Key
        buf[8] = b'k'; buf[9] = b'e'; buf[10] = b'y';
        
        // 1. Verify Arch32 handles it correctly (ignores flags)
        let node = Node::new(&buf, arch).unwrap();
        assert_eq!(node.branch_child_pgno(), 5);
        
        // 2. Verify Arch64 fails (interprets flags as high bits)
        // This confirms that if we use Arch64 on this data, we get the huge error.
        let node_64 = Node::new(&buf, DynArch::Arch64).unwrap();
        // 0x1000 << 32 = 0x100000000000 = 17592186044416
        // plus 5 = 17592186044421
        let expected_bad = (0x1000u64 << 32) | 5;
        assert_eq!(node_64.branch_child_pgno(), expected_bad);
    }
}
