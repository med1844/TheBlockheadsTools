use crate::arch::DynArch;
use crate::db_record::DbRecord;
use crate::error::Result;
use crate::page::generic::Page;
use crate::page::header::PageHeader;

#[derive(Debug)]
pub struct Cursor<'a> {
    data: &'a [u8],
    arch: DynArch,
    page_size: usize,
    root_page: u64,
    // Stack of (page_number, index) to track position in B-Tree
    // index is the current child/entry index being visited on that page
    stack: Vec<(u64, usize)>,
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8], arch: DynArch, root_page: u64, page_size: usize) -> Self {
        Self {
            data,
            arch,
            page_size,
            root_page,
            stack: vec![(root_page, 0)],
        }
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    pub fn current_page_number(&self) -> Option<u64> {
        self.stack.last().map(|(pgno, _)| *pgno)
    }

    fn get_page(&self, pgno: u64) -> Result<Page<'a>> {
        let page_size = self.page_size;

        // Overflow checks?
        let offset = pgno as usize * page_size;
        if offset + page_size > self.data.len() {
            return Err(crate::error::Error::UnexpectedEof {
                expected: offset + page_size,
                available: self.data.len(),
            });
        }
        Page::new(&self.data[offset..offset + page_size], self.arch)
    }

    pub fn get(&mut self, key: &[u8]) -> Result<Option<&'a [u8]>> {
        // Reset to linear traversal from root?
        // Typically 'get' searches from root.
        // We assume stack[0] is root.
        self.stack.clear();
        self.stack.push((self.root_page, 0));

        loop {
            let (pgno, _) = *self.stack.last().unwrap();
            let page = self.get_page(pgno)?;

            match page {
                Page::Leaf(leaf) => {
                    match leaf.search(key) {
                        Ok(idx) => {
                            self.stack.last_mut().unwrap().1 = idx;
                            let node = leaf.get_node(idx)?;

                            // Handle Value
                            if node.is_bigdata() {
                                let overflow_pgno = node.overflow_page_number().ok_or(
                                    crate::error::Error::CorruptedTree {
                                        message: "Invalid overflow node",
                                    },
                                )?;

                                // In Node::data_size(), it matches lo/hi.
                                // "The size of the data is the size of the overflow value."
                                // Wait, Node::data_size() vs Overflow Header.
                                // Node header stores the size of the value.
                                // Overflow page header also has size? Overflow reader uses size.
                                // Let's check Node::data_size implementation.
                                // "If F_BIGDATA, the node data is the page number. The size field in node header is the total data size." - Verification needed.
                                // mdb.c: "mn_lo/mn_hi is the size of the data."
                                // "If F_BIGDATA, the data is the pgno."
                                // AND "The size of the overflow data is stored in mn_lo/mn_hi."

                                // So yes, `node.data_size()` is the total payload size.

                                // Read overflow page
                                let overflow_page = self.get_page(overflow_pgno)?;
                                if let Page::Overflow(_) = overflow_page {
                                    // Construct reader just to get slice?
                                    // The slice starts at overflow_pgno * page_size + header_size.
                                    // Overflow pages have a header on the first page.
                                    // Constants: PAGE_HEADER_SIZE (16).
                                    let header_sz = PageHeader::header_size(self.arch);
                                    let start_offset =
                                        overflow_pgno as usize * self.page_size + header_sz;
                                    let total_len = node.data_size();

                                    if start_offset + total_len > self.data.len() {
                                        return Err(crate::error::Error::UnexpectedEof {
                                            expected: start_offset + total_len,
                                            available: self.data.len(),
                                        });
                                    }
                                    return Ok(Some(
                                        &self.data[start_offset..start_offset + total_len],
                                    ));
                                } else {
                                    return Err(crate::error::Error::CorruptedTree {
                                        message: "BigData node pointed to non-overflow page",
                                    });
                                }
                            } else {
                                return Ok(node.val_data());
                            }
                        }
                        Err(_) => return Ok(None),
                    }
                }
                Page::Branch(branch) => {
                    let (child_pgno, idx) = branch.search(key)?;
                    self.stack.last_mut().unwrap().1 = idx;
                    self.stack.push((child_pgno, 0));
                }
                _ => {
                    return Err(crate::error::Error::CorruptedTree {
                        message: "Unexpected page type during traversal",
                    });
                }
            }
        }
    }
    /// Navigate to the first (leftmost) entry in the tree
    pub fn to_first(&mut self) -> Result<Option<(&'a [u8], &'a [u8])>> {
        self.stack.clear();
        self.stack.push((self.root_page, 0));

        loop {
            let (pgno, _) = *self.stack.last().unwrap();
            let page = self.get_page(pgno)?;

            match page {
                Page::Leaf(leaf) => {
                    if leaf.num_keys() == 0 {
                        return Ok(None);
                    }
                    self.stack.last_mut().unwrap().1 = 0;
                    return self.get_current();
                }
                Page::Branch(branch) => {
                    if branch.num_keys() == 0 {
                        return Ok(None); // Should rely on empty check
                    }
                    self.stack.last_mut().unwrap().1 = 0;
                    // Follow first pointer
                    let node = branch.get_node(0)?;
                    self.stack.push((node.child_page_number(), 0));
                }
                _ => {
                    return Err(crate::error::Error::CorruptedTree {
                        message: "Unexpected page type during traversal",
                    });
                }
            }
        }
    }

    /// Advance cursor to next item. Returns Item if found, None if end.
    pub fn advance(&mut self) -> Result<Option<(&'a [u8], &'a [u8])>> {
        if self.stack.is_empty() {
            return Ok(None);
        }

        // 1. Try to advance in current leaf
        loop {
            let (pgno, idx) = *self.stack.last().unwrap();
            let page = self.get_page(pgno)?;

            match page {
                Page::Leaf(leaf) => {
                    if idx + 1 < leaf.num_keys() {
                        self.stack.last_mut().unwrap().1 += 1;
                        return self.get_current();
                    } else {
                        // End of leaf, go up
                        self.stack.pop();
                        if self.stack.is_empty() {
                            return Ok(None); // EOF
                        }
                        // Loop continues to process parent (Branch)
                    }
                }
                Page::Branch(branch) => {
                    // We just came UP from a child (at index idx).
                    // We need to advance to idx + 1.
                    if idx + 1 < branch.num_keys() {
                        self.stack.last_mut().unwrap().1 += 1;
                        let new_idx = idx + 1;

                        // Descend down-left from new sibling
                        let node = branch.get_node(new_idx)?;
                        self.stack.push((node.child_page_number(), 0));

                        // Now loop will hit "Leaf" case? No, the top of stack is now a Page (could be Branch or Leaf).
                        // We need to descend all the way to Leaf.
                        // So we should restart inner loop or have a "descend" phase.
                        self.descend_left()?;
                        return self.get_current();
                    } else {
                        // End of branch, go up
                        self.stack.pop();
                        if self.stack.is_empty() {
                            return Ok(None);
                        }
                    }
                }
                _ => {
                    return Err(crate::error::Error::CorruptedTree {
                        message: "Unexpected page type during traversal",
                    });
                }
            }
        }
    }

    // Helper to descend to leftmost leaf from current top of stack
    fn descend_left(&mut self) -> Result<()> {
        loop {
            let (pgno, _) = *self.stack.last().unwrap();
            let page = self.get_page(pgno)?;
            match page {
                Page::Leaf(_) => return Ok(()),
                Page::Branch(branch) => {
                    if branch.num_keys() == 0 {
                        return Ok(());
                    }
                    let node = branch.get_node(0)?;
                    self.stack.push((node.child_page_number(), 0));
                }
                _ => {
                    return Err(crate::error::Error::CorruptedTree {
                        message: "Unexpected page type",
                    });
                }
            }
        }
    }

    pub fn get_current(&mut self) -> Result<Option<(&'a [u8], &'a [u8])>> {
        if self.stack.is_empty() {
            return Ok(None);
        }
        let (pgno, idx) = *self.stack.last().unwrap();
        let page = self.get_page(pgno)?;

        match page {
            Page::Leaf(leaf) => {
                if idx >= leaf.num_keys() {
                    return Ok(None);
                }
                let node = leaf.get_node(idx)?;

                let key = node.key();
                let val_slice = if node.is_bigdata() {
                    // Duplicate logic from get() - reuse?
                    let overflow_pgno =
                        node.overflow_page_number()
                            .ok_or(crate::error::Error::CorruptedTree {
                                message: "Invalid overflow node",
                            })?; // Modified this line
                    let total_len = node.data_size();
                    let header_sz = PageHeader::header_size(self.arch); // Added this line
                    let start_offset = overflow_pgno as usize * self.page_size + header_sz; // Modified this line
                    if start_offset + total_len > self.data.len() {
                        return Err(crate::error::Error::UnexpectedEof {
                            expected: start_offset + total_len,
                            available: self.data.len(),
                        });
                    }
                    &self.data[start_offset..start_offset + total_len]
                } else {
                    node.val_data().ok_or(crate::error::Error::UnexpectedEof {
                        expected: 0,
                        available: 0,
                    })?
                };

                Ok(Some((key, val_slice)))
            }
            _ => Ok(None),
        }
    }

    pub fn seek(&mut self, key: &[u8]) -> Result<Option<(&'a [u8], &'a [u8])>> {
        self.stack.clear();
        self.stack.push((self.root_page, 0));

        loop {
            let (pgno, _) = *self.stack.last().unwrap();
            let page = self.get_page(pgno)?;

            match page {
                Page::Leaf(leaf) => {
                    let idx = match leaf.search(key) {
                        Ok(i) => i,
                        Err(i) => i,
                    };
                    self.stack.last_mut().unwrap().1 = idx;

                    if idx >= leaf.num_keys() {
                        // We are past the last element of this leaf. Move to successsor.
                        return self.advance();
                    }
                    return self.get_current();
                }
                Page::Branch(branch) => {
                    let (child_pgno, idx) = branch.search(key)?;
                    self.stack.last_mut().unwrap().1 = idx;
                    self.stack.push((child_pgno, 0));
                }
                _ => {
                    return Err(crate::error::Error::CorruptedTree {
                        message: "Unexpected page type during traversal",
                    });
                }
            }
        }
    }

    pub fn iter_start(&mut self) -> Result<CursorIter<'_, 'a>> {
        self.to_first()?;
        Ok(CursorIter {
            cursor: self,
            should_advance: false,
        })
    }

    pub fn iter_start_owned(mut self) -> Result<OwnedCursorIter<'a>> {
        self.to_first()?;
        Ok(OwnedCursorIter {
            cursor: self,
            should_advance: false,
        })
    }

    pub fn iter_from(&mut self, key: &[u8]) -> Result<CursorIter<'_, 'a>> {
        self.seek(key)?;
        Ok(CursorIter {
            cursor: self,
            should_advance: false,
        })
    }

    /// Find a named database in the main DB (assuming this cursor is on Main DB).
    pub fn find_db(&mut self, name: &str) -> Result<Option<DbRecord>> {
        if let Some(val_slice) = self.get(name.as_bytes())? {
            // Parse MDB_db struct from value
            let db = DbRecord::from_bytes(val_slice, self.arch)?;
            Ok(Some(db))
        } else {
            Ok(None)
        }
    }

    /// List all named databases in the main DB.
    pub fn list_dbs(&mut self) -> Result<Vec<(String, DbRecord)>> {
        let mut dbs = Vec::new();
        let arch = self.arch; // avoid capture

        // Must iterate all keys in main DB
        let iter = self.iter_start()?;
        for res in iter {
            match res {
                Ok((key, val)) => {
                    // println!("DEBUG: list_dbs key={:?} val_len={}", std::str::from_utf8(key).unwrap_or("?"), val.len());
                    if let Ok(name) = std::str::from_utf8(key) {
                        match DbRecord::from_bytes(val, arch) {
                            Ok(db) => dbs.push((name.to_string(), db)),
                            Err(_) => {
                                // println!("DEBUG: Failed to parse DbRecord for key {}", name);
                            }
                        }
                    } else {
                        // println!("DEBUG: Key not UTF8");
                    }
                }
                Err(_) => {
                    // println!("DEBUG: Iteration error");
                }
            }
        }
        Ok(dbs)
    }

    pub fn iter(&mut self) -> CursorIter<'_, 'a> {
        CursorIter {
            cursor: self,
            should_advance: false,
        }
    }
}

pub struct CursorIter<'c, 'a> {
    cursor: &'c mut Cursor<'a>,
    should_advance: bool,
}

impl<'c, 'a> Iterator for CursorIter<'c, 'a> {
    type Item = Result<(&'a [u8], &'a [u8])>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.should_advance {
            match self.cursor.advance() {
                Ok(Some(item)) => Some(Ok(item)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        } else {
            self.should_advance = true;
            match self.cursor.get_current() {
                Ok(Some(item)) => Some(Ok(item)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        }
    }
}

pub struct OwnedCursorIter<'a> {
    cursor: Cursor<'a>,
    should_advance: bool,
}

impl<'a> Iterator for OwnedCursorIter<'a> {
    type Item = Result<(&'a [u8], &'a [u8])>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.should_advance {
            match self.cursor.advance() {
                Ok(Some(item)) => Some(Ok(item)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        } else {
            self.should_advance = true;
            match self.cursor.get_current() {
                Ok(Some(item)) => Some(Ok(item)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::DynArch;
    use crate::constants::{P_BRANCH, P_LEAF, P_OVERFLOW};

    const TEST_PAGE_HEADER_SIZE_64: usize = 16;

    fn write_ptr(buf: &mut [u8], index: usize, offset: u16) {
        let ptr_off = TEST_PAGE_HEADER_SIZE_64 + index * 2;
        buf[ptr_off..ptr_off + 2].copy_from_slice(&offset.to_le_bytes());
    }

    fn write_leaf_node(buf: &mut [u8], offset: usize, key: &[u8], val: &[u8]) -> usize {
        let dsize = val.len() as u32;
        let ksize = key.len() as u16;
        let lo = (dsize & 0xFFFF) as u16;
        let hi = ((dsize >> 16) & 0xFFFF) as u16;

        buf[offset] = lo as u8;
        buf[offset + 1] = (lo >> 8) as u8;
        buf[offset + 2] = hi as u8;
        buf[offset + 3] = (hi >> 8) as u8;
        buf[offset + 4] = 0; // flags
        buf[offset + 5] = 0;
        buf[offset + 6] = ksize as u8;
        buf[offset + 7] = (ksize >> 8) as u8;

        buf[offset + 8..offset + 8 + key.len()].copy_from_slice(key);
        let val_off = offset + 8 + key.len();
        buf[val_off..val_off + val.len()].copy_from_slice(val);

        8 + key.len() + val.len()
    }

    #[test]
    fn test_get_single_level() {
        let arch = DynArch::Arch64;
        let page_size = 4096;
        let mut buf = vec![0u8; page_size * 2]; // Page 0 unused, Page 1 Root

        let root_pgno = 1;
        let offset = root_pgno * page_size;

        // Setup Page 1 as Leaf
        buf[offset + 10] = P_LEAF as u8; // P_LEAF
        buf[offset + 12] = 22; // mp_lower (16 + 1*2 for 3 pointers? No 1 key = 1 pointer = 18.
        // Wait. 1 key. Pointer at 16. takes 2 bytes. End at 18.
        // So mp_lower = 18.

        // Generic header num_keys calculation: (mp_lower - 16) / 2
        // So 18 - 16 = 2. 2 / 2 = 1.
        buf[offset + 12] = 18;

        // Write Node at offset 100
        let node_off = 100;
        write_leaf_node(&mut buf[offset..], node_off, b"key1", b"val1");
        write_ptr(&mut buf[offset..], 0, node_off as u16);

        // Cursor
        let mut cursor = Cursor::new(&buf, arch, root_pgno as u64, page_size);

        // Test Existing
        let res = cursor.get(b"key1").unwrap();
        assert_eq!(res, Some(&b"val1"[..]));

        // Test Missing
        let res = cursor.get(b"key2").unwrap();
        assert_eq!(res, None);
    }

    fn write_branch_node(buf: &mut [u8], offset: usize, key: &[u8], pgno: u64) -> usize {
        let lo = (pgno & 0xFFFF) as u16;
        let hi = ((pgno >> 16) & 0xFFFF) as u16;
        let flags = ((pgno >> 32) & 0xFFFF) as u16;
        let ksize = key.len() as u16;

        buf[offset] = lo as u8;
        buf[offset + 1] = (lo >> 8) as u8;
        buf[offset + 2] = hi as u8;
        buf[offset + 3] = (hi >> 8) as u8;

        buf[offset + 4] = flags as u8;
        buf[offset + 5] = (flags >> 8) as u8;

        buf[offset + 6] = ksize as u8;
        buf[offset + 7] = (ksize >> 8) as u8;

        buf[offset + 8..offset + 8 + key.len()].copy_from_slice(key);

        8 + key.len()
    }

    #[test]
    fn test_get_multi_level() {
        let arch = DynArch::Arch64;
        let page_size = 4096;
        let mut buf = vec![0u8; page_size * 3]; // Page 0, 1 (Root/Branch), 2 (Leaf)

        let root_pgno = 1;
        let leaf_pgno = 2;

        // --- Page 1: Branch ---
        let off1 = root_pgno * page_size;
        buf[off1 + 10] = P_BRANCH as u8;
        buf[off1 + 12] = 16 + 2; // 1 key

        // Node 0: key="m", child=2
        let node_off = 100;
        write_branch_node(&mut buf[off1..], node_off, b"m", leaf_pgno as u64);
        write_ptr(&mut buf[off1..], 0, node_off as u16);

        // --- Page 2: Leaf ---
        let off2 = leaf_pgno * page_size;
        buf[off2 + 10] = P_LEAF as u8;
        buf[off2 + 12] = 16 + 2;

        // Node 0: key="z", val="found"
        let leaf_node_off = 100;
        write_leaf_node(&mut buf[off2..], leaf_node_off, b"z", b"found");
        write_ptr(&mut buf[off2..], 0, leaf_node_off as u16);

        // Cursor
        let mut cursor = Cursor::new(&buf, arch, root_pgno as u64, page_size);

        // Test: Search "z"
        // Branch has "m". "z" > "m". Follow child 2.
        // Leaf has "z". Exact match. Return "found".
        let res = cursor.get(b"z").unwrap();
        assert_eq!(res, Some(&b"found"[..]));

        // Test: Search "a"
        // Branch has "m". "a" < "m".
        // In LMDB Branch, keys are separators (lowest key in child).
        // Since we only have one pointer "m", anything < "m" should technically go to the "before" pointer if it existed.
        // But here we only have 1 pointer. `search` returns index 0 (closest).
        // Child 2 is searched.
        // Leaf has "z". "a" != "z". Returns None.
        let res = cursor.get(b"a").unwrap();
        assert_eq!(res, None);
    }

    #[test]
    fn test_get_overflow_value() {
        let arch = DynArch::Arch64;
        let page_size = 4096;
        let mut buf = vec![0u8; page_size * 5]; // 0, 1(Leaf), 2(Overflow), 3(Overflow Span), 4(unused)

        let root_pgno = 1;

        // --- Page 1: Leaf ---
        let off1 = root_pgno * page_size;
        buf[off1 + 10] = P_LEAF as u8;
        buf[off1 + 12] = 16 + 2;

        // Node 0: key="big", val is F_BIGDATA pointing to pgno 2
        let node_off = 100;

        let ksize = 3u16; // "big"
        let dsize = 5000u32; // Total data size > page_size

        // Write Node manually
        let lo = (dsize & 0xFFFF) as u16;
        let hi = ((dsize >> 16) & 0xFFFF) as u16;

        buf[off1 + node_off] = lo as u8;
        buf[off1 + node_off + 1] = (lo >> 8) as u8;
        buf[off1 + node_off + 2] = hi as u8;
        buf[off1 + node_off + 3] = (hi >> 8) as u8;

        buf[off1 + node_off + 4] = 0x01; // F_BIGDATA
        buf[off1 + node_off + 5] = 0;

        buf[off1 + node_off + 6] = ksize as u8;
        buf[off1 + node_off + 7] = (ksize >> 8) as u8;

        buf[off1 + node_off + 8..off1 + node_off + 8 + 3].copy_from_slice(b"big");

        // Overflow PGNO at offset 8+3 = 11.
        let overflow_pgno: u64 = 2;
        buf[off1 + node_off + 11..off1 + node_off + 19]
            .copy_from_slice(&overflow_pgno.to_le_bytes());

        write_ptr(&mut buf[off1..], 0, node_off as u16);

        // --- Page 2 & 3: Overflow ---
        let off2 = overflow_pgno as usize * page_size;
        buf[off2 + 10] = P_OVERFLOW as u8;
        // pgno=2, num_pages=2 (covers 5000 bytes. 4096 < 5000 < 8192)
        // Header needed for num_pages? Yes, offset 12 for Arch64.
        // pgno at 0..8
        buf[off2..off2 + 8].copy_from_slice(&overflow_pgno.to_le_bytes());
        // pb_pages at 12 (u32)
        let spanned = 2u32;
        buf[off2 + 12..off2 + 16].copy_from_slice(&spanned.to_le_bytes());

        // Data Payload starts at 16
        // Fill 5000 bytes with pattern
        let data_start = off2 + 16;
        let expected_data: Vec<u8> = (0..5000).map(|i| (i % 255) as u8).collect();
        // Write to buf
        buf[data_start..data_start + 5000].copy_from_slice(&expected_data);

        // Cursor
        let mut cursor = Cursor::new(&buf, arch, root_pgno as u64, page_size);

        let res = cursor.get(b"big").unwrap();
        assert!(res.is_some());
        assert_eq!(res.unwrap(), &expected_data[..]);
    }
    #[test]
    fn test_get_empty_tree() {
        let arch = DynArch::Arch64;
        let page_size = 4096;
        let mut buf = vec![0u8; page_size * 2]; // 0, 1(Root)

        let root_pgno = 1;
        let offset = root_pgno * page_size;

        // Setup Page 1 as Empty Leaf
        buf[offset + 10] = P_LEAF as u8;
        // mp_lower = header size = 16 (no pointers)
        // mp_upper = page_size = 4096 (no data)
        buf[offset + 12] = 16;
        buf[offset + 13] = 0;

        let mut cursor = Cursor::new(&buf, arch, root_pgno as u64, page_size);

        // Search
        let res = cursor.get(b"any").unwrap();
        assert_eq!(res, None);
    }
    #[test]
    fn test_iter_multi() {
        let arch = DynArch::Arch64;
        let page_size = 4096;
        let mut buf = vec![0u8; page_size * 4]; // 0, 1(Root), 2(Leaf A), 3(Leaf B)

        let root_pgno = 1;

        // Root Branch
        let off1 = root_pgno * page_size;
        buf[off1 + 10] = P_BRANCH as u8;
        buf[off1 + 12] = 16 + 2 * 2; // 2 keys

        let node0_off = 100;
        // Key "a" -> child 2
        write_branch_node(&mut buf[off1..], node0_off, b"a", 2);
        write_ptr(&mut buf[off1..], 0, node0_off as u16);

        let node1_off = 200;
        // Key "c" -> child 3
        write_branch_node(&mut buf[off1..], node1_off, b"c", 3);
        write_ptr(&mut buf[off1..], 1, node1_off as u16);

        // Page 2: Leaf (keys "a", "b")
        let off2 = 2 * page_size;
        buf[off2 + 10] = P_LEAF as u8;
        buf[off2 + 12] = 16 + 2 * 2;

        let l2_n0 = 100;
        write_leaf_node(&mut buf[off2..], l2_n0, b"a", b"v1");
        write_ptr(&mut buf[off2..], 0, l2_n0 as u16);

        let l2_n1 = 200;
        write_leaf_node(&mut buf[off2..], l2_n1, b"b", b"v2");
        write_ptr(&mut buf[off2..], 1, l2_n1 as u16);

        // Page 3: Leaf (keys "c", "d")
        let off3 = 3 * page_size;
        buf[off3 + 10] = P_LEAF as u8;
        buf[off3 + 12] = 16 + 2 * 2;

        let l3_n0 = 100;
        write_leaf_node(&mut buf[off3..], l3_n0, b"c", b"v3");
        write_ptr(&mut buf[off3..], 0, l3_n0 as u16);

        let l3_n1 = 200;
        write_leaf_node(&mut buf[off3..], l3_n1, b"d", b"v4");
        write_ptr(&mut buf[off3..], 1, l3_n1 as u16);

        // Cursor
        let mut cursor = Cursor::new(&buf, arch, root_pgno as u64, page_size);

        // Iter Start
        let mut iter = cursor.iter_start().unwrap();
        assert_eq!(iter.next().unwrap().unwrap().0, b"a");
        assert_eq!(iter.next().unwrap().unwrap().0, b"b");
        assert_eq!(iter.next().unwrap().unwrap().0, b"c");
        assert_eq!(iter.next().unwrap().unwrap().0, b"d");
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_iter_range() {
        let arch = DynArch::Arch64;
        let page_size = 4096;
        let mut buf = vec![0u8; page_size * 2];
        let root = 1;
        let off = page_size;

        buf[off + 10] = P_LEAF as u8;
        buf[off + 12] = 16 + 4 * 2; // 4 keys

        let n0 = 100;
        write_leaf_node(&mut buf[off..], n0, b"a", b"1");
        write_ptr(&mut buf[off..], 0, n0 as u16);
        let n1 = 150;
        write_leaf_node(&mut buf[off..], n1, b"b", b"2");
        write_ptr(&mut buf[off..], 1, n1 as u16);
        let n2 = 200;
        write_leaf_node(&mut buf[off..], n2, b"c", b"3");
        write_ptr(&mut buf[off..], 2, n2 as u16);
        let n3 = 250;
        write_leaf_node(&mut buf[off..], n3, b"d", b"4");
        write_ptr(&mut buf[off..], 3, n3 as u16);

        let mut cursor = Cursor::new(&buf, arch, root as u64, page_size);

        // Range: from "b" (inclusive)
        let mut iter = cursor.iter_from(b"b").unwrap();
        assert_eq!(iter.next().unwrap().unwrap().0, b"b");
        assert_eq!(iter.next().unwrap().unwrap().0, b"c");
        assert_eq!(iter.next().unwrap().unwrap().0, b"d");
        assert!(iter.next().is_none());

        // Range: from "c"
        let mut iter = cursor.iter_from(b"c").unwrap();
        assert_eq!(iter.next().unwrap().unwrap().0, b"c");

        // Range: from "b" exclusive (simulate by checking key)
        let mut iter = cursor.iter_from(b"b").unwrap();
        let first = iter.next().unwrap().unwrap();
        if first.0 == b"b" {
            // consume
        }
        assert_eq!(iter.next().unwrap().unwrap().0, b"c");
    }
    #[test]
    fn test_find_db() {
        let arch = DynArch::Arch64;
        let page_size = 4096;
        let mut buf = vec![0u8; page_size * 2];
        let root = 1;
        let off = page_size;

        // Leaf Page
        buf[off + 10] = P_LEAF as u8;
        buf[off + 12] = 16 + 2; // 1 key

        // key="mydb", val=DbRecord(root=123)
        // Construct DbRecord bytes
        let mut db_bytes = vec![0u8; 48];
        let db_root: u64 = 123;
        db_bytes[40..48].copy_from_slice(&db_root.to_le_bytes());

        let n0 = 100;
        write_leaf_node(&mut buf[off..], n0, b"mydb", &db_bytes);
        write_ptr(&mut buf[off..], 0, n0 as u16);

        let mut cursor = Cursor::new(&buf, arch, root as u64, page_size);

        let db = cursor.find_db("mydb").unwrap().unwrap();
        assert_eq!(db.root_page, 123);

        // Missing
        assert!(cursor.find_db("missing").unwrap().is_none());
    }
    #[test]
    fn test_list_dbs() {
        let arch = DynArch::Arch64;
        let page_size = 4096;
        let mut buf = vec![0u8; page_size * 2];
        let root = 1;
        let off = page_size;

        buf[off + 10] = P_LEAF as u8;
        buf[off + 12] = 16 + 2 * 2; // 2 keys

        // Key 1: "db1"
        let mut db1_bytes = vec![0u8; 48];
        db1_bytes[40..48].copy_from_slice(&100u64.to_le_bytes()); // root=100
        let n0 = 100;
        write_leaf_node(&mut buf[off..], n0, b"db1", &db1_bytes);
        write_ptr(&mut buf[off..], 0, n0 as u16);

        // Key 2: "db2"
        let mut db2_bytes = vec![0u8; 48];
        db2_bytes[40..48].copy_from_slice(&200u64.to_le_bytes()); // root=200
        let n1 = 200;
        write_leaf_node(&mut buf[off..], n1, b"db2", &db2_bytes);
        write_ptr(&mut buf[off..], 1, n1 as u16);

        let mut cursor = Cursor::new(&buf, arch, root as u64, page_size);

        let dbs = cursor.list_dbs().unwrap();
        assert!(
            dbs.iter()
                .any(|(name, db)| name == "db1" && db.root_page == 100)
        );
        assert_eq!(dbs.len(), 2);
    }

    #[test]
    fn test_get_overflow_value_32() {
        let arch = DynArch::Arch32;
        let page_size = 4096;
        let mut buf = vec![0u8; page_size * 5];

        let root_pgno = 1;

        // --- Page 1: Leaf ---
        let off1 = root_pgno * page_size;
        // P_LEAF (2) at offset 6 (Arch32)
        buf[off1 + 6] = P_LEAF as u8;
        // mp_lower at 8. 12 byte header + 2 byte ptr = 14.
        buf[off1 + 8] = 14;

        // Node 0: key="big", val is F_BIGDATA pointing to pgno 2
        let node_off = 100;

        let ksize = 3u16; // "big"
        let dsize = 5000u32;

        // Write Node manually (Arch32)
        // lo(2), hi(2), flags(2), ksize(2)
        // For F_BIGDATA: lo/hi is size.
        let lo = (dsize & 0xFFFF) as u16;
        let hi = ((dsize >> 16) & 0xFFFF) as u16;

        buf[off1 + node_off] = lo as u8;
        buf[off1 + node_off + 1] = (lo >> 8) as u8;
        buf[off1 + node_off + 2] = hi as u8;
        buf[off1 + node_off + 3] = (hi >> 8) as u8;

        buf[off1 + node_off + 4] = 0x01; // F_BIGDATA
        buf[off1 + node_off + 5] = 0;

        buf[off1 + node_off + 6] = ksize as u8;
        buf[off1 + node_off + 7] = (ksize >> 8) as u8;

        buf[off1 + node_off + 8..off1 + node_off + 8 + 3].copy_from_slice(b"big");

        // Overflow PGNO at offset 8+3 = 11.
        // Arch32 pgno is 4 bytes.
        let overflow_pgno: u32 = 2;
        buf[off1 + node_off + 11..off1 + node_off + 15]
            .copy_from_slice(&overflow_pgno.to_le_bytes());

        // Write pointer at index 0 (offset 12)
        let ptr_off = off1 + 12;
        let node_off_u16 = node_off as u16;
        buf[ptr_off] = node_off_u16 as u8;
        buf[ptr_off + 1] = (node_off_u16 >> 8) as u8;

        // --- Page 2 & 3: Overflow ---
        let off2 = overflow_pgno as usize * page_size;
        // P_OVERFLOW (4) at offset 6 (Arch32)
        buf[off2 + 6] = P_OVERFLOW as u8;

        // pb_pages at offset 8 (u32) for Arch32.
        // pgno=2
        buf[off2..off2 + 4].copy_from_slice(&overflow_pgno.to_le_bytes());
        // pb_pages=2
        let spanned = 2u32;
        buf[off2 + 8..off2 + 12].copy_from_slice(&spanned.to_le_bytes());

        // Data Payload starts at 12 (Arch32)
        let data_start = off2 + 12;
        let expected_data: Vec<u8> = (0..5000).map(|i| (i % 255) as u8).collect();
        // Write to buf
        buf[data_start..data_start + 5000].copy_from_slice(&expected_data);

        // Cursor
        let mut cursor = Cursor::new(&buf, arch, root_pgno as u64, page_size);

        let res = cursor.get(b"big").unwrap();
        assert!(res.is_some());
        assert_eq!(res.unwrap(), &expected_data[..]);
    }
}
