use std::fmt::Display;

use crate::{BhError, BhResult};

// helper function to check if given coord is smaller than max value.
fn check_coord_limit(val: u64, max_val: u64) -> BhResult<()> {
    match val < max_val {
        false => Err(BhError::CoordError {
            input: val,
            limit: max_val,
        }),
        true => Ok(()),
    }
}

/// Block coordinate within a chunk. 0 <= x < 32, 0 <= y < 32.
/// Block coords within the chunk and their corresponding offset:
/// ```text
/// 31| 992| 993| 994|     1023|
/// 30| 960| 961| 962|      991|
///              ...
///  2|  64|  65|  66|       95|
///  1|  32|  33|  34|       63|
///  0|   0|   1|   2|       31|
///  Y`----|----|----|---------|
///   X   0|   1|   2|  ...  31|
/// ```
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ChunkBlockCoord {
    x: u8,
    y: u8,
}

impl ChunkBlockCoord {
    pub fn new(x: u8, y: u8) -> BhResult<Self> {
        check_coord_limit(x as u64, 32)?;
        check_coord_limit(y as u64, 32)?;
        Ok(Self { x, y })
    }

    pub fn x(&self) -> u8 {
        self.x
    }

    pub fn y(&self) -> u8 {
        self.y
    }
}

impl Display for ChunkBlockCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChunkBlockCoord({}, {})", self.x, self.y)
    }
}

pub trait ChunkOffset {
    fn to_offset(self) -> usize;
}

impl ChunkOffset for ChunkBlockCoord {
    fn to_offset(self) -> usize {
        ((self.y as usize) << 5 | (self.x as usize)) << 6
    }
}

impl ChunkOffset for &ChunkBlockCoord {
    fn to_offset(self) -> usize {
        ((self.y as usize) << 5 | (self.x as usize)) << 6
    }
}

/// Chunk coordinates in world. 0 <= x < world_v2.world_width_macro, 0 <= y < 32
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ChunkCoord {
    x: u32,
    y: u8,
}

impl ChunkCoord {
    /// Attempts to create a `ChunkCoord` from a string in the format "x_y".
    /// Returns `Err(BhError::ParseError)` for malformed strings or invalid numbers,
    /// or `Err(BhError::CoordError)` if coordinates are out of their initial type bounds.
    pub fn try_from_str<S: AsRef<str>>(s: S) -> BhResult<Self> {
        let s = s.as_ref();
        let mut parts = s.split('_');

        let x_str = parts
            .next()
            .ok_or_else(|| BhError::ParseError(format!("Missing x coordinate in {}", s)))?;
        let y_str = parts
            .next()
            .ok_or_else(|| BhError::ParseError(format!("Missing y coordinate in {}", s)))?;

        if parts.next().is_some() {
            return Err(BhError::ParseError(format!(
                "Too many parts in coordinate {}. Expected 'x_y' format.",
                s
            )));
        }

        let x = x_str.parse::<u32>().map_err(|e| {
            BhError::ParseError(format!("Failed to parse x coordinate as u32: {}", e))
        })?;
        let y = y_str.parse::<u8>().map_err(|e| {
            BhError::ParseError(format!("Failed to parse y coordinate as u8: {}", e))
        })?;

        Self::new(x, y)
    }

    /// Creates a new `ChunkCoord` after validating its coordinates.
    /// Returns `Err(BhError::CoordError)` if `y` is out of its valid range (0..32).
    pub fn new(x: u32, y: u8) -> BhResult<Self> {
        check_coord_limit(y as u64, 32)?;
        Ok(Self { x, y })
    }

    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u8 {
        self.y
    }
}

impl Display for ChunkCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}", self.x, self.y)
    }
}

/// Block coordinates in world. 0 <= x < world_v2.world_width_macro * 32, 0 <= y < 1024
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct BlockCoord {
    x: u32,
    y: u16,
}

impl BlockCoord {
    /// Creates a new `BlockCoord` after validating its coordinates.
    /// Returns `Err(BhError::CoordError)` if `y` is out of its valid range (0..1024).
    pub fn new(x: u32, y: u16) -> BhResult<Self> {
        check_coord_limit(y as u64, 1024)?;
        Ok(Self { x, y })
    }

    pub fn from_decomposed(chunk_coord: ChunkCoord, chunk_block_coord: ChunkBlockCoord) -> Self {
        // Both type has y checked at creation, we can trust them and unwrap.
        Self::new(
            (chunk_coord.x << 5) + chunk_block_coord.x as u32,
            ((chunk_coord.y as u16) << 5) + chunk_block_coord.y as u16,
        )
        .unwrap()
    }

    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u16 {
        self.y
    }
}

impl Display for BlockCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockCoord({}, {})", self.x, self.y)
    }
}

impl From<BlockCoord> for ChunkCoord {
    fn from(value: BlockCoord) -> Self {
        ChunkCoord::new(value.x >> 5, (value.y >> 5) as u8).expect("y < 1024, thus y >> 5 < 32")
    }
}

impl From<BlockCoord> for ChunkBlockCoord {
    fn from(value: BlockCoord) -> Self {
        ChunkBlockCoord::new((value.x & 31) as u8, (value.y & 31) as u8)
            .expect("x & 31 < 32 for all x: u64")
    }
}

impl ChunkOffset for BlockCoord {
    fn to_offset(self) -> usize {
        <Self as Into<ChunkBlockCoord>>::into(self).to_offset()
    }
}

impl BlockCoord {
    pub fn decompose(self) -> (ChunkCoord, ChunkBlockCoord) {
        (self.into(), self.into())
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockCoord, ChunkBlockCoord, ChunkCoord, ChunkOffset};

    #[test]
    fn test_chunk_block_coord_limits() {
        assert!(ChunkBlockCoord::new(0, 0).is_ok());
        assert!(ChunkBlockCoord::new(31, 31).is_ok());
        assert!(ChunkBlockCoord::new(32, 0).is_err());
        assert!(ChunkBlockCoord::new(0, 32).is_err());
    }

    #[test]
    fn test_chunk_coord_limits() {
        assert!(ChunkCoord::new(0, 0).is_ok());
        assert!(ChunkCoord::new(u32::MAX, 31).is_ok());
        assert!(ChunkCoord::new(0, 32).is_err());
    }

    #[test]
    fn test_block_coord_limits() {
        assert!(BlockCoord::new(0, 0).is_ok());
        assert!(BlockCoord::new(u32::MAX, 1023).is_ok());
        assert!(BlockCoord::new(0, 1024).is_err());
    }

    #[test]
    fn test_chunk_block_coord_offset() {
        let coord = ChunkBlockCoord::new(0, 0).unwrap();
        assert_eq!(coord.to_offset(), 0);

        let coord = ChunkBlockCoord::new(1, 0).unwrap();
        assert_eq!(coord.to_offset(), 64);

        let coord = ChunkBlockCoord::new(0, 1).unwrap();
        assert_eq!(coord.to_offset(), 2048);

        let coord = ChunkBlockCoord::new(31, 31).unwrap();
        let expected = ((31 * 32) + 31) * 64;
        assert_eq!(coord.to_offset(), expected);
    }

    #[test]
    fn test_coord_round_trip() {
        let test_cases = vec![
            (0, 0),
            (31, 31),
            (32, 0),
            (32, 32),
            (100, 100),
            (1024, 1023),
        ];

        for (x, y) in test_cases {
            if let Ok(block_coord) = BlockCoord::new(x, y) {
                let (chunk_coord, chunk_block_coord) = block_coord.decompose();
                let params_recomposed = BlockCoord::from_decomposed(chunk_coord, chunk_block_coord);
                assert_eq!(
                    block_coord, params_recomposed,
                    "Round trip failed for ({}, {})",
                    x, y
                );
            }
        }
    }

    #[test]
    fn test_chunk_coord_parse() {
        assert_eq!(
            ChunkCoord::try_from_str("10_20").unwrap(),
            ChunkCoord::new(10, 20).unwrap()
        );
        assert!(ChunkCoord::try_from_str("10_32").is_err());
        assert!(ChunkCoord::try_from_str("abc_20").is_err());
        assert!(ChunkCoord::try_from_str("10_abc").is_err());
        assert!(ChunkCoord::try_from_str("10_20_30").is_err());
        assert!(ChunkCoord::try_from_str("10").is_err());
    }
}
