use crate::error::{Error, Result};
use std::convert::TryInto;
use std::fmt::Debug;

mod sealed {
    pub trait Sealed {}
}

/// Sealed trait for architecture-specific sizes
pub trait Arch: sealed::Sealed + Copy + Clone + Debug + 'static + Send + Sync {
    /// Size of page numbers (4 for 32-bit, 8 for 64-bit)
    const PGNO_SIZE: usize;
    /// Size of size_t equivalent
    const SIZE_T_SIZE: usize;

    /// Read a page number from bytes
    fn read_pgno(bytes: &[u8]) -> Result<u64>;
    /// Read a size_t from bytes
    fn read_size(bytes: &[u8]) -> Result<u64>;
    /// Write a page number to bytes
    fn write_pgno(value: u64, buf: &mut [u8]);
    /// Write a size_t to bytes
    fn write_size(value: u64, buf: &mut [u8]);

    /// Get dynamic architecture enum
    fn as_dyn_arch() -> DynArch;
}

#[derive(Copy, Clone, Debug)]
pub struct Arch32;

impl sealed::Sealed for Arch32 {}

impl Arch for Arch32 {
    const PGNO_SIZE: usize = 4;
    const SIZE_T_SIZE: usize = 4;

    fn read_pgno(bytes: &[u8]) -> Result<u64> {
        if bytes.len() < 4 {
            return Err(Error::UnexpectedEof { expected: 4, available: bytes.len() });
        }
        let val = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        Ok(val as u64)
    }

    fn read_size(bytes: &[u8]) -> Result<u64> {
        if bytes.len() < 4 {
            return Err(Error::UnexpectedEof { expected: 4, available: bytes.len() });
        }
        let val = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        Ok(val as u64)
    }

    fn write_pgno(value: u64, buf: &mut [u8]) {
        if buf.len() >= 4 {
            buf[0..4].copy_from_slice(&(value as u32).to_le_bytes());
        }
    }

    fn write_size(value: u64, buf: &mut [u8]) {
        if buf.len() >= 4 {
            buf[0..4].copy_from_slice(&(value as u32).to_le_bytes());
        }
    }

    fn as_dyn_arch() -> DynArch {
        DynArch::Arch32
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Arch64;

impl sealed::Sealed for Arch64 {}

impl Arch for Arch64 {
    const PGNO_SIZE: usize = 8;
    const SIZE_T_SIZE: usize = 8;

    fn read_pgno(bytes: &[u8]) -> Result<u64> {
        if bytes.len() < 8 {
            return Err(Error::UnexpectedEof { expected: 8, available: bytes.len() });
        }
        let val = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        Ok(val)
    }

    fn read_size(bytes: &[u8]) -> Result<u64> {
        if bytes.len() < 8 {
            return Err(Error::UnexpectedEof { expected: 8, available: bytes.len() });
        }
        let val = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        Ok(val)
    }

    fn write_pgno(value: u64, buf: &mut [u8]) {
        if buf.len() >= 8 {
            buf[0..8].copy_from_slice(&value.to_le_bytes());
        }
    }

    fn write_size(value: u64, buf: &mut [u8]) {
        if buf.len() >= 8 {
            buf[0..8].copy_from_slice(&value.to_le_bytes());
        }
    }

    fn as_dyn_arch() -> DynArch {
        DynArch::Arch64
    }
}

/// Runtime architecture selection for reading unknown files
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DynArch {
    Arch32,
    Arch64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arch32_read_pgno() {
        let bytes = [0x01, 0x02, 0x00, 0x00]; // 513
        let pgno = Arch32::read_pgno(&bytes).unwrap();
        assert_eq!(pgno, 513);
    }

    #[test]
    fn test_arch64_read_pgno() {
        let bytes = [0x01, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // 513
        let pgno = Arch64::read_pgno(&bytes).unwrap();
        assert_eq!(pgno, 513);
    }

    #[test]
    fn test_arch32_write_read_roundtrip() {
        let mut buf = [0u8; 4];
        let val = 0x12345678;
        Arch32::write_pgno(val, &mut buf);
        assert_eq!(Arch32::read_pgno(&buf).unwrap(), val);
    }

    #[test]
    fn test_arch64_write_read_roundtrip() {
        let mut buf = [0u8; 8];
        let val = 0x1234567890ABCDEF;
        Arch64::write_pgno(val, &mut buf);
        assert_eq!(Arch64::read_pgno(&buf).unwrap(), val);
    }

    #[test]
    fn test_arch32_short_read() {
        let bytes = [0x00; 3];
        assert!(matches!(Arch32::read_pgno(&bytes), Err(Error::UnexpectedEof { .. })));
    }
}
