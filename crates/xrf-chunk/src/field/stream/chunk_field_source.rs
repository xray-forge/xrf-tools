use xrf_error::XrfResult;

use crate::field::chunk_field::ChunkField;
use crate::field::chunk_field_kind::ChunkFieldKind;

/// Fields one after another, such as an ini section's, which a [`crate::ChunkFieldReplay`] reads a value from. The
/// reader names the kind it reads, since text alone does not say it.
pub trait ChunkFieldSource {
  /// The next field, read as `kind`.
  ///
  /// # Errors
  ///
  /// Returns an error when no field is left or the next one does not read as `kind`.
  fn read_field(&mut self, kind: ChunkFieldKind) -> XrfResult<ChunkField>;

  /// Whether no field is left.
  fn is_ended(&self) -> bool;
}
