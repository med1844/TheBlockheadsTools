use super::{BytesEncode, BytesDecode};
use crate::error::{Result, Error};
use std::borrow::Cow;
use std::str;

/// Describes a `&str`.
pub struct Str;

impl BytesEncode for Str {
    type EItem = str;

    fn bytes_encode<'item>(item: &'item Self::EItem) -> Result<Cow<'item, [u8]>> {
        Ok(Cow::Borrowed(item.as_bytes()))
    }
}

impl<'a> BytesDecode<'a> for Str {
    type DItem = &'a str;

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem> {
        str::from_utf8(bytes).map_err(|_| Error::InvalidUtf8)
    }
}

/// Describes a `&[u8]`.
pub struct Bytes;

impl BytesEncode for Bytes {
    type EItem = [u8];

    fn bytes_encode<'item>(item: &'item Self::EItem) -> Result<Cow<'item, [u8]>> {
        Ok(Cow::Borrowed(item))
    }
}

impl<'a> BytesDecode<'a> for Bytes {
    type DItem = &'a [u8];

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem> {
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_round_trip() {
        let input = "hello world";
        let encoded = Str::bytes_encode(input).unwrap();
        assert_eq!(encoded, input.as_bytes());
        
        let decoded = Str::bytes_decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_str_reject_invalid_utf8() {
        // invalid sequence (0xFF)
        let invalid = b"hello\xFFworld";
        let res = Str::bytes_decode(invalid);
        assert!(matches!(res, Err(Error::InvalidUtf8)));
    }

    #[test]
    fn test_bytes_round_trip() {
        let input = b"binary\0data";
        let encoded = Bytes::bytes_encode(input).unwrap();
        assert_eq!(&encoded[..], input);
        
        let decoded = Bytes::bytes_decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_zero_copy_verification() {
        let data = b"zero copy";
        
        // Str verification
        let decoded_str = Str::bytes_decode(data).unwrap();
        // Compare pointers to ensure it points to original data
        let decoded_ptr = decoded_str.as_ptr();
        let original_ptr = data.as_ptr();
        assert_eq!(decoded_ptr, original_ptr, "Str decode should be zero-copy");
        
        // Bytes verification
        let decoded_bytes = Bytes::bytes_decode(data).unwrap();
        let decoded_bytes_ptr = decoded_bytes.as_ptr();
        assert_eq!(decoded_bytes_ptr, original_ptr, "Bytes decode should be zero-copy");
    }
}


