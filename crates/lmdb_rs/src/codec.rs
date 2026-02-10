use crate::error::Result;
use std::borrow::Cow;

/// A trait that allows to encode a type into bytes.
/// A trait that allows to encode a type into bytes.
pub trait BytesEncode {
    type EItem: ?Sized;

    fn bytes_encode<'item>(item: &'item Self::EItem) -> Result<Cow<'item, [u8]>>;
}

/// A trait that allows to decode a type from bytes.
pub trait BytesDecode<'a> {
    type DItem: 'a;

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem>;
}

pub mod types;
