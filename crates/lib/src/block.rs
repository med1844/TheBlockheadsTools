use crate::{BlockContent, block_type::BlockType, error::BhResult};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Copy)]
pub struct Block<'chunk>(&'chunk [u8; 64]);

impl<'chunk> Block<'chunk> {
    pub(crate) fn new(slice: &'chunk [u8; 64]) -> Self {
        Self(slice)
    }
}

impl<'chunk> Deref for Block<'chunk> {
    type Target = [u8; 64];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

pub struct BlockMut<'chunk>(&'chunk mut [u8; 64]);

impl<'chunk> BlockMut<'chunk> {
    pub(crate) fn new(slice: &'chunk mut [u8; 64]) -> Self {
        Self(slice)
    }
}

impl<'chunk> Deref for BlockMut<'chunk> {
    type Target = [u8; 64];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'chunk> DerefMut for BlockMut<'chunk> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

pub trait BlockView {
    fn fg(&self) -> BhResult<BlockType>;
    fn fg_raw(&self) -> u8;
    fn fg_content(&self) -> BhResult<BlockContent>;
    fn fg_content_raw(&self) -> u8;
    fn bg(&self) -> BhResult<BlockType>;
    fn bg_raw(&self) -> u8;
}

impl<T: Deref<Target = [u8; 64]>> BlockView for T {
    fn fg(&self) -> BhResult<BlockType> {
        BlockType::try_from_u8(self.fg_raw())
    }

    fn fg_raw(&self) -> u8 {
        self.deref()[0]
    }

    fn fg_content(&self) -> BhResult<BlockContent> {
        BlockContent::try_from_u8(self.fg_content_raw())
    }

    fn fg_content_raw(&self) -> u8 {
        self.deref()[3]
    }

    fn bg(&self) -> BhResult<BlockType> {
        BlockType::try_from_u8(self.bg_raw())
    }

    fn bg_raw(&self) -> u8 {
        self.deref()[1]
    }
}

pub trait BlockViewMut {
    fn set_fg<I: Into<u8>>(&mut self, value: I);
    fn set_fg_content<I: Into<u8>>(&mut self, value: I);
    fn set_bg<I: Into<u8>>(&mut self, value: I);
}

impl<T: DerefMut<Target = [u8; 64]>> BlockViewMut for T {
    fn set_fg<I: Into<u8>>(&mut self, value: I) {
        self.deref_mut()[0] = value.into();
    }

    fn set_fg_content<I: Into<u8>>(&mut self, value: I) {
        self.deref_mut()[3] = value.into();
    }

    fn set_bg<I: Into<u8>>(&mut self, value: I) {
        self.deref_mut()[1] = value.into();
    }
}
