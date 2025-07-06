use crate::error::{BhError, BhResult};

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
/// ```
/// 31| 992| 993| 994|     1023|
/// 30| 960| 961| 962|      991|
///              ...
///  2|  64|  65|  66|       95|
///  1|  32|  33|  34|       63|
///  0|   0|   1|   2|       31|
///  Y`----|----|----|---------|
///   X   0|   1|   2|  ...  31|
/// ```
#[derive(Debug, Clone, Copy)]
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
    pub x: u64,
    pub y: u8,
}

impl ChunkCoord {
    /// Attempts to create a `ChunkCoord` from a string in the format "x_y".
    /// Returns `Err(BhError::ParseError)` for malformed strings or invalid numbers,
    /// or `Err(BhError::CoordError)` if coordinates are out of their initial type bounds.
    pub fn from_str<S: AsRef<str>>(s: S) -> BhResult<Self> {
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

        let x = x_str.parse::<u64>().map_err(|e| {
            BhError::ParseError(format!("Failed to parse x coordinate as u64: {}", e))
        })?;
        let y = y_str.parse::<u8>().map_err(|e| {
            BhError::ParseError(format!("Failed to parse y coordinate as u8: {}", e))
        })?;

        Self::new(x, y)
    }

    /// Creates a new `ChunkCoord` after validating its coordinates.
    /// Returns `Err(BhError::CoordError)` if `y` is out of its valid range (0..32).
    pub fn new(x: u64, y: u8) -> BhResult<Self> {
        check_coord_limit(y as u64, 32)?;
        Ok(Self { x, y })
    }
}

/// Block coordinates in world. 0 <= x < world_v2.world_width_macro * 32, 0 <= y < 1024
#[derive(Debug, Clone, Copy)]
pub struct BlockCoord {
    x: u64,
    y: u16,
}

impl BlockCoord {
    /// Creates a new `BlockCoord` after validating its coordinates.
    /// Returns `Err(BhError::CoordError)` if `y` is out of its valid range (0..1024).
    pub fn new(x: u64, y: u16) -> BhResult<Self> {
        check_coord_limit(y as u64, 1024)?;
        Ok(Self { x, y: y as u16 })
    }
}

impl Into<ChunkCoord> for BlockCoord {
    fn into(self) -> ChunkCoord {
        ChunkCoord::new(self.x >> 5, (self.y >> 5) as u8).expect("y < 1024, thus y >> 5 < 32")
    }
}

impl Into<ChunkBlockCoord> for BlockCoord {
    fn into(self) -> ChunkBlockCoord {
        ChunkBlockCoord::new((self.x & 31) as u8, (self.y & 31) as u8)
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
