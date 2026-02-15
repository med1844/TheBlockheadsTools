use crate::arch::{Arch, DynArch};
use crate::build::database::DatabaseBuilder;
use crate::build::meta::MetaPageBuilder;
use crate::codec::BytesDecode;
use crate::cursor::Cursor;
use crate::database::Database;
use crate::db_record::DbRecord;
use crate::env::Env;
use crate::error::Result;
use crate::write::{ByteArena, SliceId};
use std::collections::HashMap;
use std::io::Write;

/// Read-only transaction.
pub struct RoTxn<'a> {
    env: &'a Env<'a>,
}

impl<'a> RoTxn<'a> {
    pub(crate) fn new(env: &'a Env<'a>) -> Result<Self> {
        Ok(Self { env })
    }

    /// Access the underlying environment.
    pub fn env(&self) -> &'a Env<'a> {
        self.env
    }

    /// Commit transaction (no-op for read-only).
    pub fn commit(self) -> Result<()> {
        Ok(())
    }

    /// Abort transaction (no-op for read-only).
    pub fn abort(self) {}

    /// Open a database.
    pub fn open_database<K, V>(&self, name: Option<&str>) -> Result<Option<Database<'a, K, V>>>
    where
        K: BytesDecode<'a>,
        V: BytesDecode<'a>,
    {
        let record = if let Some(db_name) = name {
            // Find named DB in Main DB
            // We need a cursor on Main DB
            let main_root = self.env.main_db_root();
            let mut cursor = Cursor::new(
                self.env.raw_data(),
                self.env.arch(),
                main_root,
                self.env.page_size(),
            );

            if let Some(record) = cursor.find_db(db_name)? {
                record
            } else {
                return Ok(None);
            }
        } else {
            self.env.main_db()
        };

        Ok(Some(Database::new(record)))
    }
}

// Type aliases for semantic clarity
type DbName = String;
type KeyId = SliceId;
type ValId = SliceId;

/// Write transaction that buffers entries in memory.
pub struct RwTxn<'e, W: Write> {
    pub(crate) env: &'e mut crate::env::EnvWrite<W>,
    arena: ByteArena,
    // Map: DB Name -> List of (KeyId, ValId)
    buffers: HashMap<DbName, Vec<(KeyId, ValId)>>,
}

impl<'e, W: Write> RwTxn<'e, W> {
    pub(crate) fn new(env: &'e mut crate::env::EnvWrite<W>) -> Self {
        Self {
            env,
            arena: ByteArena::new(),
            buffers: HashMap::new(),
        }
    }

    /// Create or reference a named database for writing.
    ///
    /// This returns a `Database` handle configured for writing under the given name.
    /// If `name` is `None`, it refers to the Main DB (usually not written to directly
    /// for data, but supported).
    pub fn create_database<K, V>(&mut self, name: Option<&str>) -> Result<Database<'static, K, V>> {
        let db_name = name.unwrap_or("main").to_string();
        // Initialize buffer if missing
        self.buffers.entry(db_name.clone()).or_default();

        Ok(Database::new_write(Some(db_name)))
    }

    /// Internal method for Database::put to append data to the buffer.
    /// Note: This copies data into the arena, avoiding strict Key/Value Vec allocations
    /// but still copying bytes once.
    pub(crate) fn append(&mut self, db_name: &str, key: &[u8], val: &[u8]) {
        let kid = self.arena.add(key);
        let vid = self.arena.add(val);

        if let Some(buf) = self.buffers.get_mut(db_name) {
            buf.push((kid, vid));
        } else {
            self.buffers.insert(db_name.to_string(), vec![(kid, vid)]);
        }
    }

    /// Commit the transaction.
    ///
    /// This sorts all buffered entries, builds the B-Trees, and writes the complete
    /// LMDB file structure to the underlying writer.
    pub fn commit(self) -> Result<()> {
        match self.env.arch {
            DynArch::Arch32 => self.commit_impl::<crate::arch::Arch32>(),
            DynArch::Arch64 => self.commit_impl::<crate::arch::Arch64>(),
        }
    }

    fn commit_impl<A: Arch>(self) -> Result<()> {
        let page_size = self.env.page_size;
        let mut db_builder = DatabaseBuilder::<A>::new(page_size);

        // Feed buffers to builder.
        // We sort entries by looking up keys in the arena, then stream them to the builder.
        for (name, entries) in self.buffers {
            let mut indices = entries;
            let arena = &self.arena;

            // Sort by key content
            indices.sort_by(|a, b| {
                let ka = arena.get(a.0);
                let kb = arena.get(b.0);
                ka.cmp(kb)
            });

            // Stream references to DatabaseBuilder
            let iter = indices
                .iter()
                .map(|(kid, vid)| (arena.get(*kid), arena.get(*vid)));

            db_builder.add_sorted_database(&name, iter)?;
        }

        let result = db_builder.build()?;

        // Write to writer
        let writer = &mut self.env.writer;

        // Write Meta Pages
        let meta_builder = MetaPageBuilder::<A>::new(page_size);

        // C LMDB always sets MDB_INTEGERKEY on the free DB
        let free_db = DbRecord {
            pad: page_size as u32,
            flags: crate::constants::MDB_INTEGERKEY,
            depth: 0,
            branch_pages: 0,
            leaf_pages: 0,
            overflow_pages: 0,
            entries: 0,
            root_page: u64::MAX,
            size: 0,
        };

        // Empty main DB for the initial meta page (txn_id=0)
        let empty_main_db = DbRecord {
            pad: 0,
            flags: 0,
            depth: 0,
            branch_pages: 0,
            leaf_pages: 0,
            overflow_pages: 0,
            entries: 0,
            root_page: u64::MAX,
            size: 0,
        };

        // Page 0: Initial empty state (txn_id=0), matching C LMDB's commit pattern.
        // C LMDB writes Meta 0 as the "before" state and Meta 1 as the "after" state.
        let buf0 = meta_builder.build(0, 1, 0, &free_db, &empty_main_db);
        writer.write_all(&buf0).map_err(crate::error::Error::Io)?;

        // Page 1: Committed state (txn_id=1) with actual data
        let buf1 = meta_builder.build(1, result.last_page, 1, &free_db, &result.main_db);
        writer.write_all(&buf1).map_err(crate::error::Error::Io)?;

        // Data Pages
        for page_buf in result.pages {
            writer
                .write_all(&page_buf)
                .map_err(crate::error::Error::Io)?;
        }

        writer.flush().map_err(crate::error::Error::Io)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::types::{Bytes, Str};
    use crate::constants::{DEFAULT_MAPSIZE, MDB_INTEGERKEY};
    use crate::page::meta::MetaPage;

    /// Write a simple DB via EnvWrite + RwTxn, then verify the raw meta pages
    /// match C LMDB's conventions that the game expects.
    #[test]
    fn test_meta_page_clmdb_compat() {
        // -- Write --
        let mut buf: Vec<u8> = Vec::new();
        {
            let mut env = crate::env::EnvWrite::new(&mut buf, DynArch::Arch32);
            let mut wtxn = env.write_txn().unwrap();
            let db = wtxn
                .create_database::<Bytes, Bytes>(Some("test"))
                .unwrap();
            db.put(&mut wtxn, &b"key1".as_slice(), &b"val1".as_slice())
                .unwrap();
            db.put(&mut wtxn, &b"key2".as_slice(), &b"val2".as_slice())
                .unwrap();
            wtxn.commit().unwrap();
        }

        let page_size = 4096usize;
        assert!(buf.len() >= page_size * 2, "File must have at least 2 pages");

        // -- Parse Meta 0 --
        let (meta0, arch0) = MetaPage::parse(&buf[0..page_size]).unwrap();
        assert_eq!(arch0, DynArch::Arch32);

        // Fix 1: mapsize must be DEFAULT_MAPSIZE (100MB), not exact file size
        assert_eq!(
            meta0.free_db().pad as u64 * (meta0.last_page() + 1),
            (meta0.last_page() + 1) * page_size as u64,
            "Sanity: page_size from free_db.pad should be consistent"
        );
        // Check mapsize by reading raw bytes at 32-bit meta offset 12 (within meta struct)
        let meta_off: usize = 12; // 32-bit meta struct starts at byte 12
        let mapsize = u32::from_le_bytes(buf[meta_off + 12..meta_off + 16].try_into().unwrap());
        assert_eq!(
            mapsize as u64, DEFAULT_MAPSIZE,
            "mapsize must be 100MB default, not exact file size"
        );

        // Fix 2: Meta 0 txn_id must be 0 (initial empty state)
        assert_eq!(meta0.txn_id(), 0, "Meta 0 txn_id must be 0 (initial state)");

        // Fix 2 (cont): Meta 0 main_db must be empty
        let meta0_main = meta0.main_db();
        assert_eq!(
            meta0_main.root_page,
            u32::MAX as u64,
            "Meta 0 main_db.root must be 0xFFFFFFFF (empty)"
        );
        assert_eq!(meta0_main.entries, 0, "Meta 0 main_db.entries must be 0");
        assert_eq!(meta0_main.depth, 0, "Meta 0 main_db.depth must be 0");

        // Fix 2 (cont): Meta 0 last_page should be 1 (just the two meta pages)
        assert_eq!(meta0.last_page(), 1, "Meta 0 last_page must be 1");

        // -- Parse Meta 1 --
        let (meta1, arch1) = MetaPage::parse(&buf[page_size..page_size * 2]).unwrap();
        assert_eq!(arch1, DynArch::Arch32);

        // Fix 2 (cont): Meta 1 txn_id must be 1 (committed state)
        assert_eq!(meta1.txn_id(), 1, "Meta 1 txn_id must be 1 (committed)");

        // Meta 1 should have real data
        let meta1_main = meta1.main_db();
        assert!(
            meta1_main.root_page != u32::MAX as u64,
            "Meta 1 main_db.root must point to a real page"
        );
        assert!(meta1_main.entries > 0, "Meta 1 main_db.entries must be > 0");

        // Fix 1: free_db.flags must have MDB_INTEGERKEY
        let free0 = meta0.free_db();
        assert_eq!(
            free0.flags, MDB_INTEGERKEY,
            "Meta 0 free_db.flags must have MDB_INTEGERKEY (0x08)"
        );
        let free1 = meta1.free_db();
        assert_eq!(
            free1.flags, MDB_INTEGERKEY,
            "Meta 1 free_db.flags must have MDB_INTEGERKEY (0x08)"
        );

        // -- Verify data is still readable --
        let env = crate::env::Env::new(&buf).unwrap();
        let rtxn = env.read_txn().unwrap();
        let db = env
            .open_database::<Str, Bytes>(&rtxn, Some("test"))
            .unwrap()
            .expect("test DB must exist");

        let val = db.get(&rtxn, "key1").unwrap().expect("key1 must exist");
        assert_eq!(val, b"val1");
        let val = db.get(&rtxn, "key2").unwrap().expect("key2 must exist");
        assert_eq!(val, b"val2");

        // Fix 3: Main DB leaf nodes must have F_SUBDATA flag on sub-database entries.
        // C LMDB's mdb_dbi_open checks: (node->mn_flags & (F_DUPDATA|F_SUBDATA)) != F_SUBDATA
        // Without F_SUBDATA, the game returns MDB_INCOMPATIBLE (-30784).
        let main_root = meta1_main.root_page;
        let main_page_offset = main_root as usize * page_size;
        let main_page_data = &buf[main_page_offset..main_page_offset + page_size];
        let main_page = crate::page::generic::Page::new(main_page_data, DynArch::Arch32).unwrap();
        match main_page {
            crate::page::generic::Page::Leaf(leaf) => {
                assert!(leaf.num_keys() > 0, "Main DB must have entries");
                for i in 0..leaf.num_keys() {
                    let node = leaf.get_node(i).unwrap();
                    assert!(
                        node.is_subdata(),
                        "Main DB node {} must have F_SUBDATA flag", i
                    );
                }
            }
            _ => panic!("Main DB root must be a leaf page"),
        }
    }
}
