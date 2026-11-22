use xrf_vfs::XrayInputAnswer;
use xrf_world::WorldSoundInfo;

/// What was read of one sound file, kept with what the snapshot it was read in answered for the file.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LevelSoundFileEntry {
  /// The source and revision the file was read from; none for a reference no logical path spells.
  pub answer: Option<XrayInputAnswer>,
  /// What the engine reads of it on load; none where it is missing or not a sound the engine loads.
  pub info: Option<WorldSoundInfo>,
}
