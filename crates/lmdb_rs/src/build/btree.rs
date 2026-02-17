use crate::arch::Arch;
use crate::build::overflow::OverflowBuilder;
use crate::build::page::{BranchPageBuilder, LeafPageBuilder};
use crate::db_record::DbRecord;
use std::marker::PhantomData;

/// Bulk B+Tree builder
pub struct BTreeBuilder<A: Arch> {
    page_size: usize,
    /// Generated pages buffer
    pages: Vec<Vec<u8>>,
    /// Next available page number
    next_page: u64,
    /// If true, leaf entries get F_SUBDATA flag (used for Main DB entries)
    subdata: bool,
    _arch: PhantomData<A>,

    // Stats for DbRecord
    stat_leaf_pages: u64,
    stat_branch_pages: u64,
    stat_overflow_pages: u64,
    stat_entries: u64,
    stat_depth: u32,
}

pub struct TreeBuildResult {
    pub pages: Vec<Vec<u8>>,
    pub root_page: u64,
    pub db_record: DbRecord,
    /// Next available page number after this tree
    pub next_page: u64,
}

impl<A: Arch> BTreeBuilder<A> {
    pub fn new(page_size: usize, start_page: u64) -> Self {
        Self {
            page_size,
            pages: Vec::new(),
            next_page: start_page,
            subdata: false,
            _arch: PhantomData,
            stat_leaf_pages: 0,
            stat_branch_pages: 0,
            stat_overflow_pages: 0,
            stat_entries: 0,
            stat_depth: 0,
        }
    }

    /// Set subdata mode: leaf entries will have F_SUBDATA flag.
    /// Must be called before `build()`.
    pub fn with_subdata(mut self) -> Self {
        self.subdata = true;
        self
    }

    /// Add sorted entries and build tree
    pub fn build<'a, I>(mut self, entries: I) -> Result<TreeBuildResult, crate::error::Error>
    where
        I: IntoIterator<Item = (&'a [u8], &'a [u8])>,
    {
        // 1. Build Leaf Pages
        // Info needed for parent: (separator_key, pgno)
        let mut parent_entries: Vec<(&'a [u8], u64)> = Vec::new(); // Changed to ref

        let mut current_leaf = LeafPageBuilder::<A>::new(self.page_size);
        let mut current_leaf_min_key: Option<&'a [u8]> = None; // Changed to ref

        for (key, val) in entries {
            self.stat_entries += 1;

            // Check overflow support
            let max_payload = self.page_size / 2; // Conservative threshold
            let is_overflow = val.len() > max_payload;

            let mut pushed = false;

            if is_overflow {
                // 1. Build Overflow Pages
                let ov_builder = OverflowBuilder::<A>::new(self.page_size);
                let ov_start_pg = self.next_page;
                let ov_pages = ov_builder.build(val, ov_start_pg);
                let num_ov = ov_pages.len() as u64;

                if current_leaf_min_key.is_none() {
                    current_leaf_min_key = Some(key);
                }

                // Try push overflow pointer
                if current_leaf.push_overflow(key, val.len(), ov_start_pg) {
                    pushed = true;
                    // If push succeeded, commit overflow pages
                    self.pages.extend(ov_pages);
                    self.next_page += num_ov;
                    self.stat_overflow_pages += num_ov;
                }
            } else {
                if current_leaf_min_key.is_none() {
                    current_leaf_min_key = Some(key);
                }
                let ok = if self.subdata {
                    current_leaf.push_subdata(key, val)
                } else {
                    current_leaf.push(key, val)
                };
                if ok {
                    pushed = true;
                }
            }

            if !pushed {
                // Current page is full.

                if current_leaf.is_empty() {
                    // Empty page couldn't take 1 entry?
                    // If it's overflow, maybe the overflow POINTER itself didn't fit (unlikely unless page is tiny).
                    // If it's normal, then key+val > page capacity.
                    return Err(crate::error::Error::PageFull);
                }

                // 2. Finalize current page
                let pgno = self.next_page;
                self.next_page += 1;
                self.stat_leaf_pages += 1;
                let buf = current_leaf.build(pgno);

                // 3. Record for parent
                if let Some(min_key) = current_leaf_min_key.take() {
                    parent_entries.push((min_key, pgno));
                }
                self.pages.push(buf);

                // 4. Start new page and retry push
                current_leaf = LeafPageBuilder::new(self.page_size);
                current_leaf_min_key = Some(key);

                let retry_success = if is_overflow {
                    let ov_builder = OverflowBuilder::<A>::new(self.page_size);
                    let ov_start_pg = self.next_page;
                    let ov_pages = ov_builder.build(val, ov_start_pg);
                    let num_ov = ov_pages.len() as u64;

                    if current_leaf.push_overflow(key, val.len(), ov_start_pg) {
                        self.pages.extend(ov_pages);
                        self.next_page += num_ov;
                        self.stat_overflow_pages += num_ov;
                        true
                    } else {
                        false
                    }
                } else if self.subdata {
                    current_leaf.push_subdata(key, val)
                } else {
                    current_leaf.push(key, val)
                };

                if !retry_success {
                    return Err(crate::error::Error::PageFull);
                }
            }
        }

        // Finalize last leaf
        if !current_leaf.is_empty() {
            let pgno = self.next_page;
            self.next_page += 1;
            self.stat_leaf_pages += 1;
            let buf = current_leaf.build(pgno);
            if let Some(min_key) = current_leaf_min_key {
                parent_entries.push((min_key, pgno));
            }
            self.pages.push(buf);
        }

        // If empty input, create one empty leaf?
        if self.stat_entries == 0 {
            let pgno = self.next_page;
            self.next_page += 1;
            self.stat_leaf_pages += 1;
            // Empty leaf is valid root for empty DB
            let empty_leaf = LeafPageBuilder::<A>::new(self.page_size);
            let buf = empty_leaf.build(pgno);
            parent_entries.push((&[], pgno));
            self.pages.push(buf);
        }

        self.stat_depth = 1;

        // 2. Build Branch Levels
        let mut current_level_entries = parent_entries;

        while current_level_entries.len() > 1 {
            self.stat_depth += 1;
            let mut next_level_entries: Vec<(&'a [u8], u64)> = Vec::new();
            let mut current_branch = BranchPageBuilder::<A>::new(self.page_size);
            let mut current_branch_min_key: Option<&'a [u8]> = None;

            for (key, child_pgno) in current_level_entries {
                if current_branch_min_key.is_none() {
                    current_branch_min_key = Some(key);
                }

                if !current_branch.push(key, child_pgno) {
                    if current_branch.is_empty() {
                        // Entry too big for branch? Unlikely unless key is huge.
                        return Err(crate::error::Error::PageFull);
                    }

                    let pgno = self.next_page;
                    self.next_page += 1;
                    self.stat_branch_pages += 1;
                    let buf = current_branch.build(pgno);

                    if let Some(min_key) = current_branch_min_key.take() {
                        next_level_entries.push((min_key, pgno));
                    }
                    self.pages.push(buf);

                    current_branch = BranchPageBuilder::new(self.page_size);
                    current_branch_min_key = Some(key);
                    if !current_branch.push(key, child_pgno) {
                        return Err(crate::error::Error::PageFull);
                    }
                }
            }

            // Finalize last branch
            let pgno = self.next_page;
            self.next_page += 1;
            self.stat_branch_pages += 1;
            let buf = current_branch.build(pgno);
            if let Some(min_key) = current_branch_min_key {
                next_level_entries.push((min_key, pgno));
            }
            self.pages.push(buf);

            current_level_entries = next_level_entries;
        }

        // Root is the only entry left (or the single leaf if depth 1)
        let root_page = current_level_entries[0].1;

        // Build DbRecord

        let db_record = DbRecord {
            pad: 0,
            flags: 0,
            depth: self.stat_depth as u16,
            branch_pages: self.stat_branch_pages,
            leaf_pages: self.stat_leaf_pages,
            overflow_pages: self.stat_overflow_pages,
            entries: self.stat_entries,
            root_page,
            size: 0,
        };

        Ok(TreeBuildResult {
            pages: self.pages,
            root_page,
            db_record,
            next_page: self.next_page,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::{Arch64, DynArch};
    use crate::page::generic::Page;

    #[test]
    fn test_btree_single_leaf() {
        let builder = BTreeBuilder::<Arch64>::new(4096, 10);
        let entries = vec![
            (b"key1".as_slice(), b"val1".as_slice()),
            (b"key2".as_slice(), b"val2".as_slice()),
        ];

        let result = builder.build(entries).expect("Build failed");

        assert_eq!(result.pages.len(), 1);
        assert_eq!(result.root_page, 10);
        assert_eq!(result.db_record.depth, 1);
        assert_eq!(result.db_record.entries, 2);

        // Check page content
        let page = Page::new(&result.pages[0], DynArch::Arch64).unwrap();
        match page {
            Page::Leaf(l) => assert_eq!(l.num_keys(), 2),
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_btree_split() {
        // Use small page size to force split.
        // We use 128 bytes to ensure multiple entries force a split.
        let builder = BTreeBuilder::<Arch64>::new(128, 100);

        let mut entries = Vec::new();
        for i in 0..10u8 {
            let k = vec![i]; // 1 byte
            let v = vec![i]; // 1 byte
            entries.push((k, v));
        }
        // entries needs to be sorted? Vec<u8> sort works.
        // My loop generates 0..9, already sorted.
        let entries_refs: Vec<(&[u8], &[u8])> = entries
            .iter()
            .map(|(k, v)| (k.as_slice(), v.as_slice()))
            .collect();

        let result = builder.build(entries_refs).expect("Build failed");

        assert!(result.pages.len() > 1);
        assert_eq!(result.db_record.entries, 10);
        assert!(result.db_record.depth > 1);

        // Validate root
        // Validate root. Root is the last generated page.

        let root_page_data = &result.pages.last().unwrap();
        let p = Page::new(root_page_data, DynArch::Arch64).unwrap();
        match p {
            Page::Branch(_) => {} // Good
            _ => panic!("Root should be branch for 10 items with small page"),
        }
    }
    #[test]
    fn test_btree_overflow() {
        let builder = BTreeBuilder::<Arch64>::new(4096, 500);

        // Large value > 2048
        let val = vec![0xBB; 3000];
        let entries = vec![(b"big".as_slice(), val.as_slice())];

        let result = builder.build(entries).expect("Build failed");

        // 1 Leaf Page + 1 Overflow Page (3000 fits in 4096, so 1 ov page)
        assert_eq!(result.db_record.overflow_pages, 1);
        assert_eq!(result.db_record.leaf_pages, 1);

        // Root is leaf (at 500+1 = 501? No, 500 is start.
        // 500: Overflow.
        // 501: Leaf (Root).
        assert_eq!(result.root_page, 501);

        // Verify root leaf has overflow node
        let root = &result.pages[1]; // Page indices in vector are relative 0..
        let page = Page::new(root, DynArch::Arch64).unwrap();
        match page {
            Page::Leaf(l) => {
                let node = l.get_node(0).unwrap();
                assert!(node.is_bigdata());
                assert_eq!(node.overflow_page_number().unwrap(), 500);
                assert_eq!(node.data_size(), 3000);
            }
            _ => panic!("Root not leaf"),
        }
    }
}
