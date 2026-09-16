use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::animation::animation_envelope::AnimationEnvelope;

/// The colour grading a post-process effect applies, appended by version 2.
///
/// A version 1 effect carries none at all, which is not the same as one carrying an influence of zero: the engine
/// never reads the parameter for such a file and the texture slot stays empty.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PpeColorMap {
  /// How much of the graded colour is mixed in over time, `pp_cm_influence`.
  pub influence: AnimationEnvelope,
  /// The gradient texture the grading samples, `cm_tex1`. Empty in an effect that grades nothing.
  pub texture: String,
}

impl PpeColorMap {
  /// Reads the colour grading a version 2 effect appends.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload ends inside the envelope or before the texture name terminates.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      influence: AnimationEnvelope::read::<T, D>(reader)?,
      texture: reader.read_w1251_string()?,
    })
  }

  /// Writes the colour grading back.
  ///
  /// # Errors
  ///
  /// Returns an error when the envelope carries more keys than its `u16` count holds.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.influence.write::<T>(writer)?;
    writer.write_w1251_string(&self.texture)?;

    Ok(())
  }

  /// Whether the effect grades with a texture at all, which is a name being present rather than an influence being
  /// non-zero.
  pub fn is_used(&self) -> bool {
    !self.texture.is_empty()
  }
}
