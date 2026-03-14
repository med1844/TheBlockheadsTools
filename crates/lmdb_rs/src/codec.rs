use snafu::Snafu;
use std::borrow::Cow;

#[derive(Debug, Snafu)]
pub enum CodecError {
    /// Failed to decode bytes as a UTF-8 string.
    #[snafu(display("Invalid UTF-8 sequence"))]
    InvalidUtf8,

    /// A codec-specific encode or decode error.
    #[snafu(display("Codec error: {message}"))]
    Codec { message: String },
}

pub type Result<T> = std::result::Result<T, CodecError>;

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
