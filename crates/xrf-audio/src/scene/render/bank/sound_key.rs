use std::sync::Arc;

/// A sound by its source's scope, its reference, and the version of its file it was read under.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct SoundKey {
  pub(crate) scope: Arc<str>,
  pub(crate) reference: Arc<str>,
  /// [`crate::AudioAssetSource::get_sound_version`]'s answer when the sound was asked for.
  pub(crate) version: Option<u64>,
}
