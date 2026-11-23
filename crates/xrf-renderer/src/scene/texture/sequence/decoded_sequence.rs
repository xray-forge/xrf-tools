use crate::scene::texture::decoded_texture::DecodedTexture;
use crate::scene::texture::sequence::texture_sequence_timing::TextureSequenceTiming;

/// A sequence's frames read off their files, laid out for upload as one texture whose layers are the frames in order,
/// with when each shows.
pub(crate) struct DecodedSequence {
  /// Every frame, one layer each: its `layers` is how many frames the sequence holds.
  pub frames: DecodedTexture,
  pub timing: TextureSequenceTiming,
}

impl DecodedSequence {
  /// Stacks frames, each a flat texture: one whose layout, sides or levels differ from the first's is left out with a
  /// warning, since one slot's texture takes every frame in turn; none where no frame is left.
  pub(crate) fn stack(
    reference: &str,
    frames: impl IntoIterator<Item = (String, DecodedTexture)>,
    timing: TextureSequenceTiming,
  ) -> Option<Self> {
    let mut stacked: Option<DecodedTexture> = None;

    for (name, frame) in frames {
      if !frame.is_flat() {
        log::warn!("Texture sequence '{reference}' leaves out frame '{name}', which is not a flat texture");

        continue;
      }

      let Some(first) = &mut stacked else {
        stacked = Some(frame);

        continue;
      };

      if frame.format != first.format
        || frame.width != first.width
        || frame.height != first.height
        || frame.levels.len() != first.levels.len()
      {
        log::warn!(
          "Texture sequence '{reference}' leaves out frame '{name}': {} by {} {:?} in {} levels, unlike its first frame",
          frame.width,
          frame.height,
          frame.format,
          frame.levels.len()
        );

        continue;
      }

      // Each level holds every layer's bytes in turn, which is how an array is uploaded.
      for (level, bytes) in first.levels.iter_mut().zip(frame.levels) {
        level.extend_from_slice(&bytes);
      }

      first.layers += 1;
    }

    Some(Self {
      frames: stacked?,
      timing,
    })
  }

  /// Frames it holds.
  pub(crate) fn get_count(&self) -> u32 {
    self.frames.layers
  }
}
