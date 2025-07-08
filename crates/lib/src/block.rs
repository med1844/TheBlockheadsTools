use crate::{BlockContent, block_type::BlockType, error::BhResult};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Copy)]
pub struct Block<'chunk>(&'chunk [u8; 64]);

impl<'chunk> Block<'chunk> {
    pub(crate) fn new(slice: &'chunk [u8; 64]) -> Self {
        Self(slice)
    }

    pub fn to_hex_string_single_allocation(&self) -> String {
        fn to_hex(four_bit: u8) -> &'static str {
            match four_bit {
                0 => "0",
                1 => "1",
                2 => "2",
                3 => "3",
                4 => "4",
                5 => "5",
                6 => "6",
                7 => "7",
                8 => "8",
                9 => "9",
                10 => "A",
                11 => "B",
                12 => "C",
                13 => "D",
                14 => "E",
                15 => "F",
                _ => "G",
            }
        }

        let estimated_len = 64 * 2 + (8 * 7) + 7; // 128 + 56 + 7 = 191

        let mut result = Vec::with_capacity(estimated_len);

        for (i, &byte) in self.0.iter().enumerate() {
            result.push(to_hex(byte >> 4));
            result.push(to_hex(byte & 15));

            // Add a space after every byte, except the last in a group of 8
            if i & 7 != 7 {
                result.push(" ");
            }

            // Add a newline after every 8 bytes (i.e., at the end of each line),
            // but not after the very last byte in the entire block.
            if i & 7 == 7 && i < 63 {
                result.push("\n");
            }
        }
        result.join("")
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
    fn content(&self) -> BhResult<BlockContent>;
    fn content_raw(&self) -> u8;
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

    fn content(&self) -> BhResult<BlockContent> {
        BlockContent::try_from_u8(self.content_raw())
    }

    fn content_raw(&self) -> u8 {
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
