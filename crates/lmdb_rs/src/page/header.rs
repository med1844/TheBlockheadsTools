use crate::constants::{P_BRANCH, P_LEAF, P_LEAF2, P_META, P_OVERFLOW};
use crate::page::PageResult as Result;
use std::convert::TryInto;
use std::fmt::Debug; // Ensure TryInto is used or remove if not necessary

/// Zero-copy page header (16 bytes, arch-independent header portion)
#[derive(Clone)]
pub struct PageHeader<'a> {
    data: &'a [u8],
}
impl<'a> PageHeader<'a> {
    /// Return the raw byte slice of this page
    pub fn data(&self) -> &'a [u8] {
        self.data
    }
}
// Page layout (from mdb.c):
// union {
//     pgno_t   p_pgno;  // +0 (4 or 8 bytes)
//     void *   p_next;
// } mp_p;
// uint16_t mp_pad;      // +8 (on 64-bit with 8-byte pgno) or +4 (on 32-bit)
// uint16_t mp_flags;    // +10 (on 64-bit) or +6 (on 32-bit)
// union {
//     struct {
//         indx_t pb_lower; // +12 (on 64-bit) or +8
//         indx_t pb_upper; // +14 (on 64-bit) or +10
//     } pb;
//     uint32_t pb_pages;   // +12 (on 64-bit) or +8
// } mp_pb;
// indx_t mp_ptrs[];     // +16 (PAGEHDRSZ)

// WAIT! "mp_flags" location varies by architecture?
// mdb.c:
// struct MDB_page {
//   union { pgno_t p_pgno; ... } mp_p;
//   uint16_t mp_pad;
//   uint16_t mp_flags;
//   ...
// }
//
// If pgno_t is 4 bytes (32-bit):
// +0: p_pgno (4)
// +4: mp_pad (2)
// +6: mp_flags (2)
// +8: mp_pb (4)
// +12: mp_ptrs (start) -> PAGEHDRSZ = 12 ???
//
// BUT constants.rs says PAGE_HEADER_SIZE = 16.
// mdb.c line 1026: PAGEHDRSZ = offsetof(MDB_page, mp_ptrs)
//
// On 32-bit:
// offsetof(ptrs) SHOULD be 12 if struct is packed. But is it?
//
// Let's re-read mdb.c:
// typedef struct MDB_page {
//     union {
//         pgno_t		p_pgno;
//         struct MDB_page *p_next;
//     } mp_p;
//     uint16_t	mp_pad;
//     uint16_t	mp_flags;
//     ...
// } MDB_page;
//
// On 32-bit system:
// p_pgno is 4 bytes. p_next is 4 bytes. Union is 4 bytes.
// mp_pad is 2 bytes. mp_flags is 2 bytes.
// Total so far: 4 + 2 + 2 = 8 bytes.
// mp_pb is 4 bytes (2x uint16 or 1x uint32).
// Total so far: 12 bytes.
// mp_ptrs starts at 12.
//
// So on 32-bit, PAGEHDRSZ is 12?
//
// LMDB file format is architecture-dependent.
// If the file was created on 32-bit, header might be 12 bytes?
//
// However, the "Meta Page 0" we saw had flags at offset 10 (0x0A).
// That means it matches the 64-bit layout (pgno=8, pad=2, flags=2 => 12? No 10?)
// 8 (pgno) + 2 (pad) = 10. Flags at 10. Correct.
//
// So if flags are at 6, it's 32-bit layout?
//
// Let's check `Arch::PGNO_SIZE`.
//
// We need to know architecture to parse the header IF the layout differs.
//
// Our `constants.rs` has `PAGE_HEADER_SIZE = 16`. This might be wrong for 32-bit?
// Let's check if there is a fixed size.
//
// In our `MetaPage` parsing, we detected 64-bit architecture for the 32-bit case maybe?
//
// Actually, `MDB_page` layout is strict.
// 32-bit:
//   mp_p (4)
//   mp_pad (2)
//   mp_flags (2)
//   mp_pb (4)
//   TOTAL: 12 bytes?
//
// 64-bit:
//   mp_p (8)
//   mp_pad (2)
//   mp_flags (2)
//   mp_pb (4)
//   TOTAL: 16 bytes.
//
// So `PAGEHDRSZ` really depends on arch.
//
// We need `PageHeader::parse` to take `DynArch` or generic `A: Arch`.
// Or we can try to guess?
//
// For Meta Page parsing, we used `detect_arch` based on `MDB_meta` content.
// `MDB_meta` is inside the payload.
//
// If we want to check `mp_flags` for `P_META`, we need to find WHERE `mp_flags` is.
// It is at offset 6 (32-bit) or 10 (64-bit).
//
// We can check BOTH locations.
// One of them should be `P_META` (0x08).
//
// BUT, what if `p_pgno` is large?
// On 64-bit: `p_pgno` is 8 bytes.
// On 32-bit: `p_pgno` is 4 bytes. Bytes 4-7 are `mp_pad` and `mp_flags`.
//
// If we read a 64-bit file as 32-bit:
// Bytes 0-3: pgno (low)
// Bytes 4-5: pgno (mid) -> interpreted as pad
// Bytes 6-7: pgno (high) -> interpreted as flags
//
// If pgno is 0 (Meta Page 0), then all are 0.
// So `mp_flags` (at 6) would be 0.
// But `mp_flags` (at 10) would be 8 (P_META).
//
// So for Page 0, looking at offset 6 will see 0 (invalid flag?).
// Looking at offset 10 will see 8 (valid flag).
//
// So we can detect arch from the header of Page 0!
//
// If `u16` at offset 6 is valid flag => 32-bit?
// If `u16` at offset 10 is valid flag => 64-bit?
//
// Valid flags are bitmasks. 0 is not a valid page type (must be branch, leaf, overflow, meta).
//
// Exception: FreeDB pages might have specific flags?
//
// Let's assume we pass a flag to `parse` or have a `parse_with_arch`.
//
// But `MetaPage::parse` calls `detect_arch`.
//
// Maybe `PageHeader` should just wrap the bytes and provide accessors that take `DynArch`?
// Or we resolve header size during `MetaPage::parse`.
//
// Let's define `PageHeader` to handle both.

impl<'a> PageHeader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn page_number(&self, arch: crate::arch::DynArch) -> Result<u64> {
        match arch {
            crate::arch::DynArch::Arch32 => {
                let val = u32::from_le_bytes(self.data[0..4].try_into().unwrap());
                Ok(val as u64)
            }
            crate::arch::DynArch::Arch64 => {
                let val = u64::from_le_bytes(self.data[0..8].try_into().unwrap());
                Ok(val)
            }
        }
    }

    pub fn flags(&self, arch: crate::arch::DynArch) -> u16 {
        match arch {
            crate::arch::DynArch::Arch32 => u16::from_le_bytes(self.data[6..8].try_into().unwrap()),
            crate::arch::DynArch::Arch64 => {
                u16::from_le_bytes(self.data[10..12].try_into().unwrap())
            }
        }
    }

    // Helper to guess arch from flags
    pub fn guess_arch(&self) -> Option<crate::arch::DynArch> {
        let flags32 = u16::from_le_bytes(self.data[6..8].try_into().unwrap());
        let flags64 = u16::from_le_bytes(self.data[10..12].try_into().unwrap());

        let valid32 = is_valid_flags(flags32);
        let valid64 = is_valid_flags(flags64);

        match (valid32, valid64) {
            (true, false) => Some(crate::arch::DynArch::Arch32),
            (false, true) => Some(crate::arch::DynArch::Arch64),
            (true, true) => {
                // Ambiguous.
                // If page number is small (0), both might look valid if 0 is considered invalid.
                // P_META is 0x08.
                None
            }
            (false, false) => None,
        }
    }

    pub fn is_meta(&self, arch: crate::arch::DynArch) -> bool {
        (self.flags(arch) & P_META) != 0
    }

    pub fn lower(&self, arch: crate::arch::DynArch) -> u16 {
        match arch {
            crate::arch::DynArch::Arch32 => {
                u16::from_le_bytes(self.data[8..10].try_into().unwrap())
            }
            crate::arch::DynArch::Arch64 => {
                u16::from_le_bytes(self.data[12..14].try_into().unwrap())
            }
        }
    }

    pub fn upper(&self, arch: crate::arch::DynArch) -> u16 {
        match arch {
            crate::arch::DynArch::Arch32 => {
                u16::from_le_bytes(self.data[10..12].try_into().unwrap())
            }
            crate::arch::DynArch::Arch64 => {
                u16::from_le_bytes(self.data[14..16].try_into().unwrap())
            }
        }
    }

    pub fn header_size(arch: crate::arch::DynArch) -> usize {
        match arch {
            crate::arch::DynArch::Arch32 => 12,
            crate::arch::DynArch::Arch64 => 16,
        }
    }

    pub fn num_keys(&self, arch: crate::arch::DynArch) -> usize {
        let lower = self.lower(arch) as usize;
        let header_sz = Self::header_size(arch);

        if lower < header_sz {
            return 0;
        }
        (lower - header_sz) / 2
    }
}

fn is_valid_flags(flags: u16) -> bool {
    // Must have at least one type bit set?
    // Types: BRANCH(1), LEAF(2), OVERFLOW(4), META(8)
    // One of these MUST be set for a valid page?
    // mdb.c checks msg_flags against P_BRANCH|P_LEAF|P_LEAF2|P_OVERFLOW|P_META
    let type_mask = P_BRANCH | P_LEAF | P_LEAF2 | P_OVERFLOW | P_META;
    (flags & type_mask) != 0
}

impl<'a> Debug for PageHeader<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PageHeader {{ ... }}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::DynArch;

    #[test]
    fn test_header_32() {
        let mut data = [0u8; 16];
        // pgno = 1
        data[0] = 1;
        // flags at 6 = P_LEAF (2)
        data[6] = 2;

        let h = PageHeader::new(&data);
        assert_eq!(h.flags(DynArch::Arch32), 2);
        assert_eq!(h.page_number(DynArch::Arch32).unwrap(), 1);
        assert!(h.guess_arch() == Some(DynArch::Arch32) || h.guess_arch().is_none());
    }

    #[test]
    fn test_header_64() {
        let mut data = [0u8; 16];
        // pgno = 1
        data[0] = 1;
        // flags at 10 = P_LEAF (2)
        data[10] = 2;

        let h = PageHeader::new(&data);
        assert_eq!(h.flags(DynArch::Arch64), 2);
        assert_eq!(h.page_number(DynArch::Arch64).unwrap(), 1);
    }
}
