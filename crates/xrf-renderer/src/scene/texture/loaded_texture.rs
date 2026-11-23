use crate::scene::texture::decoded_texture::DecodedTexture;
use crate::scene::texture::sequence::decoded_sequence::DecodedSequence;

/// What a slot's texture read as, waiting for its upload: one texture, or the frames of a sequence its name lists.
pub(crate) enum LoadedTexture {
  Still(DecodedTexture),
  Sequence(DecodedSequence),
}

impl LoadedTexture {
  /// Bytes the upload carries, which is what an upload budget spends.
  pub(crate) fn get_size(&self) -> u64 {
    self.get_decoded().get_size()
  }

  /// The texture, or a sequence's frames as one: what its sides, layout and levels are read from.
  pub(crate) fn get_decoded(&self) -> &DecodedTexture {
    match self {
      Self::Still(texture) => texture,
      Self::Sequence(sequence) => &sequence.frames,
    }
  }
}
