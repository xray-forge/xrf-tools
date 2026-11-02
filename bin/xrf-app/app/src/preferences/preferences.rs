use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_store::{Store, StoreBuilder};
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;

use crate::preferences::preference_key::PreferenceKey;

/// What the backend remembers between runs, as opposed to what the document keeps in its own local storage: kept in the
/// application's own data, beside the executable for a portable install.
pub struct Preferences<R: Runtime> {
  store: Arc<Store<R>>,
}

impl<R: Runtime> Clone for Preferences<R> {
  fn clone(&self) -> Self {
    Self {
      store: Arc::clone(&self.store),
    }
  }
}

impl<R: Runtime> Preferences<R> {
  /// How long a write waits for a quieter moment before it reaches the disk.
  const AUTO_SAVE_DEBOUNCE: Duration = Duration::from_secs(1);

  /// Open the preference file, which is created by its first write.
  pub fn open(application: &AppHandle<R>) -> TauriResult<Self> {
    let path: PathBuf = application
      .state::<AppDirectories>()
      .get_preferences_file()
      .ok_or_else(|| String::from("No application data directory to keep the preferences in"))?;

    let store: Arc<Store<R>> = StoreBuilder::new(application, path)
      .auto_save(Self::AUTO_SAVE_DEBOUNCE)
      .build()
      .map_err(|error| format!("Failed to open the preference store: {error}"))?;

    Ok(Self { store })
  }

  /// Read one preference, treating one that does not read as its type as one that was never written.
  pub fn read<T: DeserializeOwned>(&self, key: PreferenceKey) -> Option<T> {
    let value: Value = self.store.get(key.as_str())?;

    match serde_json::from_value(value) {
      Ok(value) => Some(value),
      Err(error) => {
        log::warn!(
          "Ignoring the stored '{}' preference, which does not read as one: {error}",
          key.as_str()
        );

        None
      }
    }
  }

  /// Write one preference, leaving the store to carry it to the disk.
  pub fn write<T: Serialize>(&self, key: PreferenceKey, value: &T) -> TauriResult<()> {
    let value: Value = serde_json::to_value(value)
      .map_err(|error| format!("Failed to write the '{}' preference: {error}", key.as_str()))?;

    self.store.set(key.as_str(), value);

    Ok(())
  }

  /// Put what is written on the disk now, for a caller that cannot wait for the debounce.
  pub fn flush(&self) -> TauriResult<()> {
    self
      .store
      .save()
      .map_err(|error| format!("Failed to save the preferences: {error}"))
  }
}
