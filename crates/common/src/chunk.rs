//! Data structure for compactly storing blocks in the world.

use utils::PackedArray;

use crate::{blocks, BlockId};

/// The dimensions of a chunk (cube).
pub const CHUNK_DIM: usize = 16;
/// The volume of a chunk in blocks.
pub const CHUNK_VOLUME: usize = CHUNK_DIM * CHUNK_DIM * CHUNK_DIM;

/// Position of a chunk relative to the zone origin.
/// Measured in units of CHUNK_DIM = 16 blocks.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// The starting number of bits per block to use in a chunk.
const INITIAL_BITS_PER_BLOCK: usize = 3;

/// Efficiently and compactly stores a 16x16x16 chunk of blocks.
///
/// Internally, a chunk contains a packed array of bits and a palette.
/// Each entry in the packed array is an index into the palette, which
/// is a `Vec<BlockId>`. For chunks with small numbers of blocks, we can
/// use as few as 3-4 bits per block.
pub struct Chunk {
    /// Stores indexes into `palette` of blocks for each position.
    indexes: PackedArray,
    /// The set of distinct block states in this chunk.
    ///
    /// This palette must remain stable unless `indexes` is updated
    /// in accordance.
    palette: Vec<BlockId>,
}

impl Chunk {
    /// Creates a new chunk initialized with air.
    pub fn new() -> Self {
        Self {
            indexes: PackedArray::new(CHUNK_VOLUME, INITIAL_BITS_PER_BLOCK),
            palette: vec![BlockId::new(blocks::Air)],
        }
    }

    /// Gets the block at the given position within this chunk.
    ///
    /// # Panics
    /// Panics if `x, y, or z >= CHUNK_DIM`.
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockId {
        Self::check_bounds(x, y, z);
        let index = self
            .indexes
            .get(Self::ordinal(x, y, z))
            .expect("bounds checked") as usize;

        self.palette[index]
    }

    /// Sets the block at the given position within this chunk.
    ///
    /// # Panics
    /// Panics if `x, y, or z >= CHUNK_DIM`.
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockId) {
        Self::check_bounds(x, y, z);
        let index = self.find_in_palette(block);
        self.indexes.set(Self::ordinal(x, y, z), index as u64);
    }

    fn find_in_palette(&mut self, block: BlockId) -> usize {
        match self.palette.iter().position(|b| *b == block) {
            Some(pos) => pos,
            None => {
                let pos = self.palette.len();
                self.grow_palette(block);
                pos
            }
        }
    }

    fn grow_palette(&mut self, block: BlockId) {
        self.palette.push(block);

        // If the new length of the palette exceeds the
        // max value in the `indexes` packed array, we need
        // to resize the indexes.
        if self.palette.len() - 1 > self.indexes.max_value() as usize {
            self.indexes = self.indexes.resized(self.indexes.bits_per_value() + 1);
        }
    }

    fn check_bounds(x: usize, y: usize, z: usize) {
        assert!(x < CHUNK_DIM, "x coordinate {} out of bounds", x);
        assert!(y < CHUNK_DIM, "y coordinate {} out of bounds", y);
        assert!(z < CHUNK_DIM, "z coordinate {} out of bounds", z);
    }

    fn ordinal(x: usize, y: usize, z: usize) -> usize {
        (y * CHUNK_DIM * CHUNK_DIM) + (z * CHUNK_DIM) + x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_smoke() {
        let mut chunk = Chunk::new();

        for x in 0..CHUNK_DIM {
            for y in 0..CHUNK_DIM {
                for z in 0..CHUNK_DIM {
                    assert!(chunk.get(x, y, z).is::<blocks::Air>());
                    chunk.set(x, y, z, BlockId::new(blocks::Dirt));
                    assert!(chunk.get(x, y, z).is::<blocks::Dirt>());
                }
            }
        }
    }
}
