use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::XrfResult;

use crate::level_visual::LevelVisual;

/// Every visual of a compiled level, the `fsL_VISUALS` chunk.
#[derive(Debug)]
pub struct LevelVisualsChunk {
  pub visuals: Vec<LevelVisual>,
}

impl LevelVisualsChunk {
  /// `fsL_VISUALS` (`xray-16/src/Common/LevelStructure.hpp:8`).
  pub const CHUNK_ID: u32 = 3;

  /// Reads every visual from a reader positioned at the visuals chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when any visual cannot be read. One unreadable visual fails the chunk rather than being
  /// skipped: a gap in the run would silently renumber every visual after it, and the number is the identity.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut visuals: Vec<LevelVisual> = Vec::with_capacity(chunks.len());

    for mut chunk in chunks {
      visuals.push(LevelVisual::read_from_chunk::<T, D>(&mut chunk)?);
    }

    Ok(Self { visuals })
  }

  /// Visuals that draw from the level's shared buffers, which is fewer than the run holds.
  pub fn count_drawable(&self) -> usize {
    self.visuals.iter().filter(|it| it.is_drawable()).count()
  }

  /// Visuals the compiler gave a `level.geomX` fast path.
  pub fn count_fastpath(&self) -> usize {
    self.visuals.iter().filter(|it| it.fastpath.is_some()).count()
  }
}
