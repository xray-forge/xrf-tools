use serde::{Deserialize, Serialize};
use xrf_app_core::preferences::PreferenceKey;
use xrf_audio::AudioOutputChoice;

use crate::settings::audio_volumes::AudioVolumes;

/// How the application plays sound, kept by the backend between runs so it holds before any page loads; stored
/// settings that no longer read whole are left for the defaults.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSettings {
  /// The device sound plays on.
  pub output: AudioOutputChoice,
  pub volumes: AudioVolumes,
  /// Whether sound falls silent while no window of the application has the focus.
  pub is_muted_in_background: bool,
}

impl AudioSettings {
  pub(crate) const PREFERENCE_KEY: PreferenceKey = PreferenceKey::new("audio.settings");

  /// The settings with every volume in range.
  pub fn clamp(&self) -> Self {
    Self {
      volumes: self.volumes.clamp(),
      ..self.clone()
    }
  }
}

impl Default for AudioSettings {
  fn default() -> Self {
    Self {
      output: AudioOutputChoice::SystemDefault,
      volumes: AudioVolumes::default(),
      is_muted_in_background: true,
    }
  }
}
