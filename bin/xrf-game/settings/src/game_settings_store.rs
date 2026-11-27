use std::collections::BTreeMap;
use std::path::PathBuf;

use xrf_error::XrfResult;

use crate::game_setting_entry::GameSettingEntry;
use crate::game_settings::GameSettings;
use crate::game_settings_changes::GameSettingsChanges;
use crate::game_settings_group::GameSettingsGroup;

/// The settings the game runs with, and the one place they change: the shell owns it for the run, and every client
/// (the console, and the bridge through it; later the options menu) sets a setting here, never on a copy of its own.
/// What changed is kept by group until the shell takes it and applies it to the parts that read that group; what changed
/// since the last save is kept to write to the settings file.
#[derive(Clone, Debug)]
pub struct GameSettingsStore {
  settings: GameSettings,
  /// Where the settings file may be, best first, as loading reads it.
  files: Vec<PathBuf>,
  /// Each setting set since the last save, by its path, at the value it took.
  unsaved: BTreeMap<String, String>,
  changes: GameSettingsChanges,
}

impl GameSettingsStore {
  /// The store over `settings` as the run started with them, saving its changes to the first of `files` that exists,
  /// else the first.
  pub fn new(settings: GameSettings, files: Vec<PathBuf>) -> Self {
    Self {
      settings,
      files,
      unsaved: BTreeMap::new(),
      changes: GameSettingsChanges::default(),
    }
  }

  /// The settings as they stand.
  pub fn get(&self) -> &GameSettings {
    &self.settings
  }

  /// Every setting as the console and the file write it, in path order.
  pub fn get_entries(&self) -> Vec<GameSettingEntry> {
    self.settings.get_entries()
  }

  /// Sets the setting at `path` to `value`, held to its bounds, answering it as it now reads; it is kept as a change of
  /// its group and to save. Why not where the path names no setting or the value is not one it takes.
  ///
  /// # Errors
  ///
  /// Returns why the setting does not take the value.
  pub fn set(&mut self, path: &str, value: &str) -> Result<GameSettingEntry, String> {
    let settings: GameSettings = self.settings.with_setting(path, value)?;
    let entry: GameSettingEntry = settings
      .get_entries()
      .into_iter()
      .find(|entry| entry.path == path)
      .unwrap_or_else(|| GameSettingEntry {
        path: String::from(path),
        value: String::from(value),
      });

    self.settings = settings;
    self.unsaved.insert(entry.path.clone(), entry.value.clone());

    if let Some(group) = GameSettingsGroup::of_path(path) {
      self.changes.insert(group);
    }

    Ok(entry)
  }

  /// Takes what changed since the last take.
  pub fn take_changes(&mut self) -> GameSettingsChanges {
    std::mem::take(&mut self.changes)
  }

  /// How many settings changed since the last save.
  pub fn count_unsaved(&self) -> usize {
    self.unsaved.len()
  }

  /// Saves what changed since the last save into the settings file (`GameSettings::save_changes`), answering the file
  /// written, none where nothing changed.
  ///
  /// # Errors
  ///
  /// Returns why the file cannot be written; the changes are kept to save again.
  pub fn save(&mut self) -> XrfResult<Option<PathBuf>> {
    if self.unsaved.is_empty() {
      return Ok(None);
    }

    let changes: Vec<GameSettingEntry> = self
      .unsaved
      .iter()
      .map(|(path, value)| GameSettingEntry {
        path: path.clone(),
        value: value.clone(),
      })
      .collect();
    let file: PathBuf = GameSettings::save_changes(&self.files, &changes)?;

    self.unsaved.clear();

    Ok(Some(file))
  }
}
