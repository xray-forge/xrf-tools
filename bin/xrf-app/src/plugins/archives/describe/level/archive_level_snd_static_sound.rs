use serde::Serialize;
use xrf_db::SndStaticSound;
use xrf_vfs::XrayAssetType;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_reference::ArchiveReference;

/// One sound a level plants, as the viewer reads it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelSndStaticSound {
  /// The sound played, which does name a file. Absent where the record names none.
  pub sound: Option<ArchiveReference>,
  pub volume: f32,
  pub frequency: f32,
  /// Whether the sound only plays inside a window of the day, which four of the 670 shipped ones do. A pair of
  /// zeroes is no window at all rather than a window of no length.
  pub is_scheduled: bool,
  pub active_from: u32,
  pub active_to: u32,
}

impl ArchiveLevelSndStaticSound {
  /// Every sound a level plants, with the file each names resolved.
  pub fn of_all(source: &ArchiveDescribeSource, sounds: &[SndStaticSound]) -> Vec<Self> {
    sounds.iter().map(|sound| Self::of(source, sound)).collect()
  }

  /// One sound, taken over what it plays and when.
  fn of(source: &ArchiveDescribeSource, sound: &SndStaticSound) -> Self {
    Self {
      sound: (!sound.sound.is_empty()).then(|| ArchiveReference::resolve(source, XrayAssetType::Ogg, &sound.sound)),
      volume: sound.volume,
      frequency: sound.frequency,
      is_scheduled: sound.active.is_bounded(),
      active_from: sound.active.from,
      active_to: sound.active.to,
    }
  }
}
