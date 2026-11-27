use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxSectionWrite};

use crate::game_preset::GamePreset;
use crate::game_setting_entry::GameSettingEntry;
use crate::game_settings::GameSettings;
use crate::game_settings_load::GameSettingsFile;

impl GameSettings {
  /// Saves `changes` into the settings file: the first of `files` that exists, else the first, made where missing. The
  /// file's own settings are read and the changes applied over them, so what a run was started with for itself
  /// (`--preset`, `--set`) stays out of it; the file is then written whole, its preset and every setting differing from
  /// that preset. A change the settings do not take is logged and left out. Answers the file written.
  ///
  /// # Errors
  ///
  /// Returns why the file cannot be written, or that `files` names none.
  pub fn save_changes(files: &[PathBuf], changes: &[GameSettingEntry]) -> XrfResult<PathBuf> {
    let path: &Path = files
      .iter()
      .find(|file| file.is_file())
      .or_else(|| files.first())
      .ok_or_else(|| XrfError::new_not_found_error("the game has no place for its settings file"))?;
    let file: GameSettingsFile = if path.is_file() {
      Self::read_settings_file(path)
    } else {
      GameSettingsFile::default()
    };
    let preset: GamePreset = file.preset.unwrap_or_default();
    let mut settings: Self = preset.get_settings().merge(&file.overrides);

    for change in changes {
      match settings.with_setting(&change.path, &change.value) {
        Ok(changed) => settings = changed,
        Err(reason) => log::warn!("Setting '{}' not saved: {reason}", change.path),
      }
    }

    settings.create_settings_ltx(preset).write_to_path(path)?;

    Ok(path.to_path_buf())
  }

  /// These settings as their file writes them over `preset`: the preset's name in `[settings]`, then each setting that
  /// differs from it in the section of its group, `[view.shadows] resolution = 2048`.
  pub fn create_settings_ltx(&self, preset: GamePreset) -> Ltx {
    let defaults: Vec<GameSettingEntry> = preset.get_settings().get_entries();
    let mut ltx: Ltx = Ltx::new();

    ltx
      .with_section(Self::FILE_SECTION)
      .set(Self::PRESET_KEY, preset.get_name());

    for entry in self.get_entries() {
      if defaults.contains(&entry) {
        continue;
      }

      if let Some((group, key)) = entry.path.rsplit_once('.') {
        ltx.with_section(group).set(key, &entry.value);
      }
    }

    ltx
  }
}
