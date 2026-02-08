use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use maybe_owned::MaybeOwned;
use std::io::{Read, Write};

pub trait FromGzip: Sized {
    fn from_compressed_gzip(bytes: &[u8]) -> Result<Self, std::io::Error>;
}

pub trait ToGzip: Sized {
    fn to_gzip(&self) -> std::io::Result<Vec<u8>>;
}

impl<B: AsRef<[u8]>> ToGzip for B {
    fn to_gzip(&self) -> std::io::Result<Vec<u8>> {
        let a = self.as_ref();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(a)?;
        encoder.finish()
    }
}

pub fn decompress_gzip_to(bytes: &[u8], output: &mut Vec<u8>) -> Result<usize, std::io::Error> {
    let mut decoder = GzDecoder::new(bytes);
    decoder.read_to_end(output)
}

pub fn decompress_gzip(bytes: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut output = Vec::new();
    decompress_gzip_to(bytes, &mut output)?;
    Ok(output)
}

pub fn compress_gzip_to<W: Write>(bytes: &[u8], output: W) -> std::io::Result<()> {
    let mut encoder = GzEncoder::new(output, Compression::best());
    encoder.write_all(bytes)?;
    encoder.finish()?;
    Ok(())
}

pub fn compress_gzip(bytes: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut output = Vec::new();
    compress_gzip_to(bytes, &mut output)?;
    Ok(output)
}

#[derive(Debug, Clone)]
pub enum Gzip<T> {
    Compressed(Vec<u8>),
    Uncompressed(T),
}

impl<T: FromGzip> Gzip<T> {
    fn ensure_decompressed(&mut self) -> Result<(), std::io::Error> {
        let current_state = std::mem::replace(self, Gzip::Compressed(Vec::new()));
        *self = match current_state {
            Gzip::Compressed(vec) => match T::from_compressed_gzip(vec.as_slice()) {
                Ok(obj) => Gzip::Uncompressed(obj),
                Err(e) => {
                    *self = Gzip::Compressed(vec);
                    return Err(e);
                }
            },
            obj @ Gzip::Uncompressed(_) => obj,
        };
        Ok(())
    }

    pub fn as_uncompressed(&mut self) -> Result<&T, std::io::Error> {
        self.ensure_decompressed()?;
        if let Self::Uncompressed(val) = self {
            Ok(val)
        } else {
            Err(std::io::Error::other(
                "Internal error: Gzip state unexpected after decompression",
            ))
        }
    }

    pub fn as_uncompressed_mut(&mut self) -> Result<&mut T, std::io::Error> {
        self.ensure_decompressed()?;
        if let Self::Uncompressed(val) = self {
            Ok(val)
        } else {
            Err(std::io::Error::other(
                "Internal error: Gzip state unexpected after decompression",
            ))
        }
    }
}

impl<T: ToGzip> Gzip<T> {
    pub fn to_compressed<'s>(&'s self) -> std::io::Result<MaybeOwned<'s, Vec<u8>>> {
        match self {
            Gzip::Compressed(vec) => Ok(MaybeOwned::Borrowed(vec)),
            Gzip::Uncompressed(t) => Ok(MaybeOwned::Owned(t.to_gzip()?)),
        }
    }
}

impl<T> Gzip<T> {
    pub fn from_compressed(bytes: Vec<u8>) -> Self {
        Self::Compressed(bytes)
    }
}
