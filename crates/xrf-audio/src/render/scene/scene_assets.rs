use std::sync::Arc;

use crate::host::audio_asset_source::AudioAssetSource;

/// Where a scene's sounds are read: its host's source, and the scope that names them, so two scenes reading the same
/// files share what was decoded of them.
pub(crate) struct SceneAssets {
  pub(crate) scope: Arc<str>,
  pub(crate) source: Arc<dyn AudioAssetSource>,
}

impl SceneAssets {
  pub(crate) fn new(source: Arc<dyn AudioAssetSource>) -> Self {
    Self {
      scope: Arc::from(source.get_sound_scope()),
      source,
    }
  }
}
