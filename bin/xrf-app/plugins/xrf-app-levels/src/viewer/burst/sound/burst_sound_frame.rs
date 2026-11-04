use std::sync::Arc;

use xrf_audio::{AudioAssetSource, AudioSceneFrame, AudioSceneGeometry};

/// A scene's sound frame a burst drew, at the burst's time, and where the scene's sounds and walls are read from, so the
/// burst's own session plays the scene as the live one does.
pub struct BurstSoundFrame {
  pub frame: Box<AudioSceneFrame>,
  pub assets: Arc<dyn AudioAssetSource>,
  pub geometry: Arc<dyn AudioSceneGeometry>,
}
