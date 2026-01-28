use crate::arch::Arch;
use crate::constants::{MDB_MAGIC, P_META};
use crate::db_record::DbRecord;
use std::marker::PhantomData;

/// Builder for meta pages
pub struct MetaPageBuilder<A: Arch> {
    page_size: usize,
    _arch: PhantomData<A>,
}

impl<A: Arch> MetaPageBuilder<A> {
    pub fn new(page_size: usize) -> Self {
        Self {
            page_size,
            _arch: PhantomData,
        }
    }

    /// Build meta page
    pub fn build(
        &self,
        page_number: u64,  // 0 or 1
        last_page: u64,
        txn_id: u64,
        free_db: &DbRecord,
        main_db: &DbRecord,
    ) -> Vec<u8> {
        let mut buffer = vec![0u8; self.page_size];
        
        // Writes Page Header + Meta Header
        // Layout:
        // Header: pgno, pad, flags.
        // Meta: magic...
        
        let meta_offset = if A::PGNO_SIZE == 8 { // 64-bit
             // Header:
             // 0..8: pgno
             // 8..10: pad
             // 10..12: flags
             // 12..16: pad2
             
             A::write_pgno(page_number, &mut buffer[0..]);
             let flags = P_META;
             buffer[10..12].copy_from_slice(&flags.to_le_bytes());
             
             16
        } else { // 32-bit
             // Header:
             // 0..4: pgno
             // 4..6: pad
             // 6..8: flags
             
             A::write_pgno(page_number, &mut buffer[0..]);
             let flags = P_META;
             buffer[6..8].copy_from_slice(&flags.to_le_bytes());
             
             // Validates against legacy parser logic which expects magic at 12.
             12
        };
        
        // Write Meta struct at meta_offset
        
        // Magic & Version
        buffer[meta_offset..meta_offset+4].copy_from_slice(&MDB_MAGIC.to_le_bytes());
        buffer[meta_offset+4..meta_offset+8].copy_from_slice(&1u32.to_le_bytes());
        
        // Address (ptr) - 0 for new file
        // Mapsize (size_t)
        
        let mapsize = (last_page + 1) * (self.page_size as u64);
        
        let db0_offset;
        let db1_offset;
        let last_pg_offset;
        let txn_id_offset;
        
        if A::PGNO_SIZE == 8 { // 64-bit
            // +8: address(8) = 0
            // +16: mapsize(8)
            let mut tmp = [0u8; 8];
            A::write_size(mapsize, &mut tmp);
            buffer[meta_offset+16..meta_offset+24].copy_from_slice(&tmp[0..8]);
            
            db0_offset = meta_offset + 24;
            // Offsets relative to START OF META struct
            // db0 at 24.
            // db1 at 72.
            // last_pg at 120.
            db1_offset = meta_offset + 72;
            last_pg_offset = meta_offset + 120;
            txn_id_offset = meta_offset + 128; // last_pg is 8 bytes
            
        } else { // 32-bit
            // +8 address(4)
            // +12 mapsize(4)
            let mut tmp = [0u8; 8];
            A::write_size(mapsize, &mut tmp);
            buffer[meta_offset+12..meta_offset+16].copy_from_slice(&tmp[0..4]);
            
            db0_offset = meta_offset + 16;
            db1_offset = meta_offset + 44;
            last_pg_offset = meta_offset + 72;
            txn_id_offset = meta_offset + 76; // last_pg is 4 bytes
        }
        
        // Write DB0 (Free)
        let db0_bytes = self.serialize_db_record(free_db);
        buffer[db0_offset..db0_offset+db0_bytes.len()].copy_from_slice(&db0_bytes);
        
        // Write DB1 (Main)
        let db1_bytes = self.serialize_db_record(main_db);
        buffer[db1_offset..db1_offset+db1_bytes.len()].copy_from_slice(&db1_bytes);
        
        // Write last_pg
        let mut tmp = [0u8; 8];
        A::write_pgno(last_page, &mut tmp);
        let pgno_size = A::PGNO_SIZE;
        buffer[last_pg_offset..last_pg_offset+pgno_size].copy_from_slice(&tmp[0..pgno_size]);
        
        // Write txn_id
        A::write_pgno(txn_id, &mut tmp); // txnid_t is pgno_t alias (size_t)
        buffer[txn_id_offset..txn_id_offset+pgno_size].copy_from_slice(&tmp[0..pgno_size]);
        
        buffer
    }
    

    
    fn serialize_db_record(&self, record: &DbRecord) -> Vec<u8> {
        let mut buf = Vec::new();
        // Caller ensures record.pad is set correctly (e.g. page_size for free_db).
        
        buf.extend_from_slice(&record.pad.to_le_bytes());
        buf.extend_from_slice(&record.flags.to_le_bytes());
        buf.extend_from_slice(&record.depth.to_le_bytes());
        
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
    use crate::arch::{Arch64, DynArch};
    use crate::page::meta::MetaPage;
    
    #[test]
    fn test_meta_builder_arch64() {
        let builder = MetaPageBuilder::<Arch64>::new(4096);
        
        let free_db = DbRecord {
             pad: 4096, // Page size stored here
             flags: 0,
             depth: 0,
             branch_pages: 0,
             leaf_pages: 0,
             overflow_pages: 0,
             entries: 0,
             root_page: u64::MAX, // Empty
             size: 0,
        };
        
        let main_db = DbRecord {
             pad: 0,
             flags: 0,
             depth: 1,
             branch_pages: 1,
             leaf_pages: 1,
             overflow_pages: 0,
             entries: 10,
             root_page: 5,
             size: 0,
        };
        
        let buf = builder.build(0, 100, 1, &free_db, &main_db);
        
        // Verify parse
        let (meta, arch) = MetaPage::parse(&buf).expect("Parse failed");
        assert_eq!(arch, DynArch::Arch64);
        assert_eq!(meta.magic(), MDB_MAGIC);
        assert_eq!(meta.version(), 1);
        assert_eq!(meta.page_size(), 4096);
        assert_eq!(meta.last_page(), 100);
        assert_eq!(meta.txn_id(), 1);
        
        let db0 = meta.free_db();
        assert_eq!(db0.pad, 4096);
        
        let db1 = meta.main_db();
        assert_eq!(db1.entries, 10);
        assert_eq!(db1.root_page, 5);
    }
}
