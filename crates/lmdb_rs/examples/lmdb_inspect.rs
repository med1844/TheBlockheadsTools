use clap::Parser;
use lmdb_rs::page::generic::Page;
use lmdb_rs::page::meta::MetaPage;
use lmdb_rs::page::header::PageHeader;
use lmdb_rs::constants::{P_BRANCH, P_LEAF, P_OVERFLOW, P_META};
use memmap2::MmapOptions;
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the LMDB data file (usually data.mdb)
    #[arg(value_name = "FILE")]
    path: PathBuf,
}

fn hex_dump(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Inspecting file: {:?}", args.path);

    let file = File::open(&args.path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };

    println!("File size: {} bytes", mmap.len());

    // Basic size check
    if mmap.len() < 512 {
        println!("File too small ({} bytes)", mmap.len());
        return Ok(());
    }

    // 1. Try Parse Meta 0
    println!("Page 0 Header (0..32): {}", hex_dump(&mmap[0..32]));
    let meta0_res = MetaPage::parse(&mmap);

    // 2. Try Parse Meta 1
    // We need page size to find offset. If Meta 0 valid, use that.
    // If Meta 0 invalid, we can guess 4096 or try based on standard offsets.
    // Standard LMDB: Meta 1 is at offset PAGE_SIZE.

    let mut page_size = 4096; // Default guess
    if let Ok((ref m0, _)) = meta0_res {
        page_size = m0.page_size() as usize;
    }

    let meta1_offset = page_size;
    let meta1_res = if mmap.len() >= meta1_offset + 512 {
        MetaPage::parse(&mmap[meta1_offset..])
    } else {
        Err(lmdb_rs::error::Error::UnexpectedEof {
            expected: meta1_offset + 512,
            available: mmap.len(),
        })
    };

    println!("\n--- Meta Page Analysis ---");

    // Pick Active
    let (active_meta, active_arch) = match (meta0_res, meta1_res) {
        (Ok((m0, arch0)), Ok((m1, _))) => {
            if m1.txn_id() > m0.txn_id() {
                println!("Active Meta: Page 1 (Higher TxnID)");
                (m1, arch0)
            } else {
                println!("Active Meta: Page 0 (Higher or Equal TxnID)");
                (m0, arch0)
            }
        }
        (Ok((m0, arch0)), Err(_)) => {
            println!("Active Meta: Page 0 (Page 1 Invalid)");
            (m0, arch0)
        }
        (Err(_), Ok((m1, arch1))) => {
            println!("Active Meta: Page 1 (Page 0 Invalid)");
            (m1, arch1)
        }
        (Err(_), Err(_)) => {
            println!("FATAL: No valid meta pages found.");
            return Ok(());
        }
    };

    // Dump details of Active Meta
    println!("\n--- Active Meta Page Details ---");
    println!("  Architecture: {:?}", active_arch);
    println!("  Magic:        {:#x}", active_meta.magic());
    println!("  Version:      {}", active_meta.version());
    println!("  Page Size:    {}", active_meta.page_size());
    println!("  Last Page:    {}", active_meta.last_page());
    println!("  Txn ID:       {}", active_meta.txn_id());

    println!("  Free DB:");
    print_db_record(&active_meta.free_db(), "    ");

    println!("  Main DB:");
    print_db_record(&active_meta.main_db(), "    ");

// Helper (moved from local print_db_record call if generic unavailable, but looks like we need to define it or use Debug)
// Oh wait, print_db_record is not defined in the snippet I saw?
// I need to check if existing code has print_db_record.
// Yes, line 97 calls print_db_record. But I didn't see definition logic in current file snippet?
// Ah, allow me to check if print_db_record is imported or defined locally.


    println!("\n  Note: To see sub-databases, tree traversal is required (Future Step).");
    println!("        Sub-databases are stored as KV pairs in the Main DB.");

    // Helper: Collect Free Pgnos
    let mut free_pgnos = std::collections::HashSet::new();
    println!("\n--- Analyzing FreeDB ---");
    let free_db = active_meta.free_db();
    if free_db.root_page != u64::MAX && free_db.entries > 0 { // u64::MAX or 0?
        // Root page might be valid even if empty?
        let env_data = &mmap[..];
        let mut cursor = lmdb_rs::cursor::Cursor::new(env_data, active_arch, free_db.root_page, page_size);
        match cursor.iter_start() {
             Ok(iter) => {
                 for item in iter {
                     if let Ok((_key, val)) = item {
                         // FreeDB: Key=TxnID, Value=Pgno (or multiple Pgnos if DUPSORT?)
                         // Actually, FreeDB usually just stores Pgnos as values.
                         // Val should be pgno_t.
                         let pgno_res = match active_arch {
                             lmdb_rs::arch::DynArch::Arch32 => {
                                 if val.len() >= 4 {
                                     Some(u32::from_le_bytes(val[0..4].try_into().unwrap()) as u64)
                                 } else { None }
                             },
                             lmdb_rs::arch::DynArch::Arch64 => {
                                 if val.len() >= 8 {
                                     Some(u64::from_le_bytes(val[0..8].try_into().unwrap()))
                                 } else { None }
                             }
                         };
                         if let Some(p) = pgno_res {
                             free_pgnos.insert(p);
                         }
                     }
                 }
                 println!("Found {} free pages in FreeDB.", free_pgnos.len());
             },
             Err(e) => println!("Warning: Could not iterate FreeDB: {:?}", e),
        }
    }

    let page_size = active_meta.page_size() as usize;
    let last_page = active_meta.last_page();

    // Dump All Pages
    println!("\n--- Dumping All Pages (0 to {}) ---", last_page);
    let mut pgno = 0;
    while pgno <= last_page {
        let offset = (pgno as usize) * page_size;
        if offset + page_size > mmap.len() {
            println!("Page {}: EOF (Start: {}, File Size: {})", pgno, offset, mmap.len());
            break;
        }
        let page_data = &mmap[offset..offset + page_size];

        if pgno == 346 {
            println!("    [DEBUG] Page 346 Header: {}", hex_dump(&page_data[0..16]));
        }

        // Check if Free
        let is_free = free_pgnos.contains(&pgno);

        match Page::new(page_data, active_arch) {
            Ok(page) => {
                 // Format Page Type
                 let type_str = match &page {
                     Page::Meta(_) => "Meta",
                     Page::Branch(_) => "Branch",
                     Page::Leaf(_) => "Leaf",
                     Page::Overflow(_) => "Overflow",
                     Page::Other(_) => "Other",
                 };

                if is_free {
                    println!("Page {}: [FREE] Parsed as {} (Flags: {:#x})", pgno, type_str, page.flags(active_arch));
                } else {
                    // Normal Print
                     match &page {
                        Page::Meta(m) => println!("Page {}: Meta (TxnID {})", pgno, m.txn_id()),
                        Page::Branch(b) => println!("Page {}: Branch, keys={}", pgno, b.num_keys()),
                        Page::Leaf(l) => println!("Page {}: Leaf, keys={}", pgno, l.num_keys()),
                        Page::Overflow(_) => {
                             let n = page.overflow_pages(active_arch).unwrap_or(0);
                             println!("Page {}: Overflow, spanned_pages={}", pgno, n);
                             if n > 0 && n < 1_000_000_000 { // Sanity check for skip
                                 if n > 1 {
                                     println!(" (Skipping {} following pages)", n - 1);
                                     pgno += (n as u64) - 1;
                                 }
                             }
                        },
                        Page::Other(_) => println!("Page {}: Other/Empty", pgno),
                     }
                }
            },
            Err(e) => {
                if is_free {
                    println!("Page {}: [FREE] Garbage/Error: {}", pgno, e);
                } else {
                    // Fallback: Check flags manually like mdb_inspect.c
                    let header = PageHeader::new(page_data);
                    let flags = header.flags(active_arch);
                    
                    if (flags & P_META) != 0 {
                         println!("Page {}: Meta (Error: {:?})", pgno, e);
                    } else if (flags & P_BRANCH) != 0 {
                         println!("Page {}: Branch (Error: {:?})", pgno, e);
                    } else if (flags & P_LEAF) != 0 {
                         println!("Page {}: Leaf (Error: {:?})", pgno, e);
                    } else if (flags & P_OVERFLOW) != 0 {
                         println!("Page {}: Overflow (Error: {:?})", pgno, e);
                    } else {
                         println!("Page {}: Error parsing: {}", pgno, e);
                         println!("    [DEBUG] Page Header: {}", hex_dump(&page_data[0..16]));
                    }
                }
            }
        }
        pgno += 1;
    }

    // 4. Trace Iteration (List Databases)
    println!("\n--- Databases (via Cursor Trace) ---");
    let root_page = active_meta.main_db().root_page;
    let arch = active_arch;

    // Safety: mmap is valid for the duration.
    let data = &mmap[..];
    let mut cursor = lmdb_rs::cursor::Cursor::new(data, arch, root_page, page_size);

    match cursor.list_dbs() {
        Ok(dbs) => {
            println!("Found {} named databases:", dbs.len());
            for (name, db) in dbs {
                println!(
                    "  - {} (Size: {}): Root Page {}, Entries {}, Depth {}",
                    name, db.size, db.root_page, db.entries, db.depth
                );
            }
        }
        Err(e) => println!("Error listing databases: {:?}", e),
    }

    Ok(())
}

fn print_db_record(db: &lmdb_rs::db_record::DbRecord, indent: &str) {
    println!("{}Flags:          {:#x}", indent, db.flags);
    println!("{}Depth:          {}", indent, db.depth);
    println!("{}Branch Pages:   {}", indent, db.branch_pages);
    println!("{}Leaf Pages:     {}", indent, db.leaf_pages);
    println!("{}Overflow Pages: {}", indent, db.overflow_pages);
    println!("{}Entries:        {}", indent, db.entries);
    println!("{}Root Page:      {}", indent, db.root_page);
}
