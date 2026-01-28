use crate::env::Env;
use crate::error::Result;
use crate::database::Database;
use crate::codec::BytesDecode;
use crate::cursor::Cursor;
use crate::arch::{Arch, DynArch};
use crate::build::database::DatabaseBuilder;
use crate::build::meta::MetaPageBuilder;
use crate::db_record::DbRecord;
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
            let mut cursor = Cursor::new(self.env.raw_data(), self.env.arch(), main_root, self.env.page_size());
            
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

        let free_db = DbRecord {
            pad: page_size as u32,
            flags: 0,
            depth: 0,
            branch_pages: 0,
            leaf_pages: 0,
            overflow_pages: 0,
            entries: 0,
            root_page: u64::MAX,
            size: 0,
        };

        // Page 0
        let buf0 = meta_builder.build(0, result.last_page, 1, &free_db, &result.main_db);
        writer.write_all(&buf0).map_err(crate::error::Error::Io)?;

        // Page 1
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
