use crate::arch::DynArch;
use crate::arch::{Arch, Arch32};
use crate::constants::{P_BRANCH, P_LEAF, P_LEAF2, P_META, P_OVERFLOW};
use crate::error::{Error, Result};
use crate::page::branch::BranchPage;
use crate::page::header::PageHeader;
use crate::page::leaf::LeafPage;
use crate::page::meta::MetaPage;

/// Generic Page wrapper
#[derive(Debug)]
pub enum Page<'a> {
    Meta(MetaPage<'a>),
    Branch(BranchPage<'a>),
    Leaf(LeafPage<'a>),
    Overflow(PageHeader<'a>),
    // Could support P_LEAF2 / P_SUBP / P_LOOSE if needed
    Other(PageHeader<'a>),
}

impl<'a> Page<'a> {
    pub fn new(data: &'a [u8], arch: DynArch) -> Result<Self> {
        // Peek header
        if data.len() < 16 {
             // Basic size check (min header is 12 or 16)
              return Err(Error::UnexpectedEof { expected: 16, available: data.len() });
        }
        
        let header = PageHeader::new(data);
        let flags = header.flags(arch);
        
        if (flags & P_META) != 0 {
            let meta = MetaPage::new(data)?; 
            Ok(Page::Meta(meta))
        } else if (flags & P_BRANCH) != 0 {
            Ok(Page::Branch(BranchPage::new(data, arch)?))
        } else if (flags & P_LEAF) != 0 || (flags & P_LEAF2) != 0 {
            Ok(Page::Leaf(LeafPage::new(data, arch)?))
        } else if (flags & P_OVERFLOW) != 0 {
            Ok(Page::Overflow(header))
        } else {
            Ok(Page::Other(header))
        }
    }
    
    pub fn page_number(&self, arch: DynArch) -> Option<u64> {
        match self {
            Page::Meta(p) => Some(p.header().page_number(arch).unwrap_or(0)),
            Page::Branch(p) => p.header().page_number(arch).ok(),
            Page::Leaf(p) => p.header().page_number(arch).ok(),
            Page::Overflow(h) => h.page_number(arch).ok(),
            Page::Other(h) => h.page_number(arch).ok(),
        }
    }
    
    pub fn overflow_pages(&self, arch: DynArch) -> Option<u32> {
        match self {
             Page::Overflow(h) => {
                 match arch {
                     DynArch::Arch32 => {
                         // pgno(4) + pad(2) + flags(2) = 8. pb_pages at 8. (u32)
                         Arch32::read_size(&h.data()[8..]).ok().map(|x| x as u32)
                     },
                     DynArch::Arch64 => {
                         // pgno(8) + pad(2) + flags(2) = 12. pb_pages at 12. (u32, even on 64-bit)
                         // Note: We use Arch32::read_size because pb_pages is always u32.
                         Arch32::read_size(&h.data()[12..]).ok().map(|x| x as u32)
                     }
                 }
             },
             _ => None,
        }
    }
    pub fn flags(&self, arch: DynArch) -> u16 {
        match self {
            Page::Meta(p) => p.header().flags(arch),
            Page::Branch(p) => p.header().flags(arch),
            Page::Leaf(p) => p.header().flags(arch),
            Page::Overflow(h) => h.flags(arch),
            Page::Other(h) => h.flags(arch),
        }
    }
}
