use crate::arch::Arch;
use crate::build::btree::BTreeBuilder;
use crate::db_record::DbRecord;
use std::marker::PhantomData;

pub type DbEntry = (Vec<u8>, Vec<u8>);

/// Builder for complete LMDB database file
pub struct DatabaseBuilder<A: Arch> {
    page_size: usize,
    /// Next available page number
    next_page: u64,
    /// Built named databases: (name, pages, db_record)
    built_dbs: Vec<(String, Vec<Vec<u8>>, DbRecord)>,
    _arch: PhantomData<A>,
}

pub struct BuildResult {
    pub pages: Vec<Vec<u8>>,
    pub main_db: DbRecord,
    pub last_page: u64,
}

/// Helper struct for Main DB Entries
/// Main DB contains entries where Key = "db_name", Value = "DbRecord".
/// We need to encode DbRecord to bytes as it is stored as the value in the Main DB leaf nodes.
impl<A: Arch> DatabaseBuilder<A> {
    pub fn new(page_size: usize) -> Self {
        Self {
            page_size,
            next_page: 2, // Data pages start at 2
            built_dbs: Vec::new(),
            _arch: PhantomData,
        }
    }

    /// Add a named database with its entries.
    /// entries: Iterator of (key, val). keys must be sorted.
    pub fn add_sorted_database<'a, I>(
        &mut self,
        name: &str,
        entries: I,
    ) -> Result<(), crate::error::Error>
    where
        I: Iterator<Item = (&'a [u8], &'a [u8])>,
    {
        // Build immediately
        let builder = BTreeBuilder::<A>::new(self.page_size, self.next_page);
        let result = builder.build(entries)?;

        self.built_dbs
            .push((name.to_string(), result.pages, result.db_record));
        self.next_page = result.next_page;
        Ok(())
    }

    /// Legacy support method (if needed by other code, though likely unused now)
    /// Wraps add_sorted_database
    pub fn add_database(&mut self, name: &str, entries: Vec<DbEntry>) {
        // We assume input is sorted or sort it? The old one didn't sort here, it sorted in `build`.
        // But `add_sorted_database` expects sorted.
        // Let's sort to be safe if reusing this for legacy.
        let mut sorted = entries;
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        let iter = sorted.iter().map(|(k, v)| (k.as_slice(), v.as_slice()));
        let _ = self.add_sorted_database(name, iter);
    }

    pub fn build(self) -> Result<BuildResult, crate::error::Error> {
        // 1. Create Main DB entries sorted by name
        // We do NOT sort `built_dbs` directly because that would scramble the physical page order.
        let mut indices: Vec<usize> = (0..self.built_dbs.len()).collect();
        indices.sort_by(|&i, &j| {
            self.built_dbs[i]
                .0
                .as_bytes()
                .cmp(self.built_dbs[j].0.as_bytes())
        });

        let mut main_db_entries: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        for &i in &indices {
            let (name, _, record) = &self.built_dbs[i];
            let record_bytes = Self::serialize_db_record(record);
            main_db_entries.push((name.as_bytes().to_vec(), record_bytes));
        }

        let mut all_pages = Vec::new();

        // 2. Aggregate Pages in original allocation order
        for (_, pages, _) in self.built_dbs {
            all_pages.extend(pages);
        }

        // 3. Build Main DB (entries are sub-database records, need F_SUBDATA flag)
        let builder = BTreeBuilder::<A>::new(self.page_size, self.next_page).with_subdata();
        let result = builder.build(
            main_db_entries
                .iter()
                .map(|(k, v)| (k.as_slice(), v.as_slice())),
        )?;

        all_pages.extend(result.pages);
        let final_next_page = result.next_page;

        Ok(BuildResult {
            pages: all_pages,
            main_db: result.db_record,
            last_page: final_next_page - 1,
        })
    }

    fn serialize_db_record(record: &DbRecord) -> Vec<u8> {
        // Serialize DbRecord to bytes matching C struct layout (MDB_db).
        // The layout depends on the Architecture (32-bit vs 64-bit).
        // Matches `src/db_record.rs` parsing logic.

        let mut buf = Vec::new();
        // pad(4)
        buf.extend_from_slice(&record.pad.to_le_bytes());
        // flags(2)
        buf.extend_from_slice(&record.flags.to_le_bytes());
        // depth(2)
        buf.extend_from_slice(&record.depth.to_le_bytes());

        // branch
        let mut tmp = [0u8; 8];
        A::write_pgno(record.branch_pages, &mut tmp);
        buf.extend_from_slice(&tmp[0..A::PGNO_SIZE]);

        A::write_pgno(record.leaf_pages, &mut tmp);
        buf.extend_from_slice(&tmp[0..A::PGNO_SIZE]);

        A::write_pgno(record.overflow_pages, &mut tmp);
        buf.extend_from_slice(&tmp[0..A::PGNO_SIZE]);

        A::write_size(record.entries, &mut tmp);
        buf.extend_from_slice(&tmp[0..A::SIZE_T_SIZE]);

        A::write_pgno(record.root_page, &mut tmp);
        buf.extend_from_slice(&tmp[0..A::PGNO_SIZE]);

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::Arch64;

    #[test]
    fn test_database_builder_struct() {
        let mut builder = DatabaseBuilder::<Arch64>::new(4096);
        builder.add_database("db1", vec![(b"k".to_vec(), b"v".to_vec())]);

        let res = builder.build().expect("Build failed");

        // Main DB should contain "db1"
        assert!(res.main_db.entries >= 1);

        // Pages should exist
        assert!(!res.pages.is_empty());
    }
}
