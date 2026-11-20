use std::time::Duration;

use serde::Serialize;

/// What a texture description run wrote: sheets packed, or sprites cut out of them.
///
/// What a non-strict run passed over is counted beside what it wrote, so a run that skipped half its input does not
/// read as a complete one.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptionSpriteResult {
  #[serde(with = "xrf_utils::duration_ms")]
  pub duration: Duration,
  /// Described files written: a packed sheet, or an unpacked sheet's folder of sprites.
  pub files: u32,
  /// Described files passed over because their sheet could not be read.
  pub skipped_files: u32,
  /// Sprites placed into a sheet, or written out of one.
  pub sprites: u32,
  /// Sprites passed over because they could not be read or did not fit their sheet.
  pub skipped_sprites: u32,
}

impl DescriptionSpriteResult {
  /// One file written, holding `sprites` and passing over `skipped_sprites`.
  pub(crate) fn written(sprites: u32, skipped_sprites: u32) -> Self {
    Self {
      files: 1,
      sprites,
      skipped_sprites,
      ..Self::default()
    }
  }

  /// One file passed over whole.
  pub(crate) fn skipped() -> Self {
    Self {
      skipped_files: 1,
      ..Self::default()
    }
  }

  /// Both counts together; the duration is the whole run's, set once the counts are in.
  pub(crate) fn merged(self, other: Self) -> Self {
    Self {
      duration: self.duration,
      files: self.files + other.files,
      skipped_files: self.skipped_files + other.skipped_files,
      sprites: self.sprites + other.sprites,
      skipped_sprites: self.skipped_sprites + other.skipped_sprites,
    }
  }
}
