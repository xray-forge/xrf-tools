use std::sync::{Mutex, PoisonError};

use tauri::{AppHandle, Runtime};
use xrf_app_core::TauriResult;
use xrf_app_core::preferences::Preferences;

use crate::settings::audio_settings::AudioSettings;

/// The audio settings, read from the backend's preferences as the plugin sets up and written back on each change.
pub struct AudioSettingsStore {
  /// `None` where the application has nowhere to keep preferences: the settings then last the run.
  preferences: Option<Preferences>,
  settings: Mutex<AudioSettings>,
}

impl AudioSettingsStore {
  /// The stored settings, or the defaults where none are stored or they do not read.
  pub fn open<R: Runtime>(application: &AppHandle<R>) -> Self {
    let preferences: Option<Preferences> = Preferences::open(application)
      .inspect_err(|error| log::error!("Audio settings last this run only: {error}"))
      .ok();
    let settings: AudioSettings = preferences
      .as_ref()
      .and_then(|preferences| preferences.read::<AudioSettings>(AudioSettings::PREFERENCE_KEY))
      .unwrap_or_default()
      .clamp();

    Self {
      preferences,
      settings: Mutex::new(settings),
    }
  }

  pub fn get(&self) -> AudioSettings {
    self.settings.lock().unwrap_or_else(PoisonError::into_inner).clone()
  }

  /// Keeps `settings`, every volume held in range, and answers what was kept.
  ///
  /// # Errors
  ///
  /// Returns an error when the preferences refuse the write; the settings still hold for this run.
  pub fn set(&self, settings: &AudioSettings) -> TauriResult<AudioSettings> {
    let settings: AudioSettings = settings.clamp();

    *self.settings.lock().unwrap_or_else(PoisonError::into_inner) = settings.clone();

    if let Some(preferences) = &self.preferences {
      preferences.write(AudioSettings::PREFERENCE_KEY, &settings)?;
    }

    Ok(settings)
  }
}
