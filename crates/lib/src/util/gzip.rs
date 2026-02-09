use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::io::{Read, Write};

pub fn decompress_into(bytes: &[u8], output: &mut Vec<u8>) -> std::io::Result<usize> {
    let mut decoder = GzDecoder::new(bytes);
    decoder.read_to_end(output)
}

pub fn decompress_exact_into(bytes: &[u8], output: &mut [u8]) -> std::io::Result<()> {
    let mut decoder = GzDecoder::new(bytes);
    decoder.read_exact(output)
}

pub fn decompress(bytes: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut output = Vec::new();
    decompress_into(bytes, &mut output)?;
    Ok(output)
}

pub fn compress_into<W: Write>(bytes: &[u8], output: W) -> std::io::Result<()> {
    let mut encoder = GzEncoder::new(output, Compression::best());
    encoder.write_all(bytes)?;
    encoder.finish()?;
    Ok(())
}

pub fn compress(bytes: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut output = Vec::new();
    compress_into(bytes, &mut output)?;
    Ok(output)
}
