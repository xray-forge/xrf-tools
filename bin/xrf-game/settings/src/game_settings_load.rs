use std::path::{Path, PathBuf};
use std::str::FromStr;

use xrf_ltx::Ltx;

use crate::game_preset::GamePreset;
use crate::game_setting_override::GameSettingOverride;
use crate::game_settings::GameSettings;

/// What a settings file holds: the preset it starts from, and the settings it changes over it.
#[derive(Debug, Default)]
pub(crate) struct GameSettingsFile {
  pub preset: Option<GamePreset>,
  pub overrides: Vec<GameSettingOverride>,
}

impl GameSettings {
  /// The section naming the file's own choices rather than a group of settings.
  pub(crate) const FILE_SECTION: &'static str = "settings";

  /// The key in [`Self::FILE_SECTION`] naming the preset the file starts from.
  pub(crate) const PRESET_KEY: &'static str = "preset";

  /// The settings this run draws with: `preset` where the run names one, else the settings file's, else the default;
  /// the settings file's values over it, then each of `sets`, `<path>=<value>`. The file is the first of `files` that
  /// exists; none is the preset alone, and one that cannot be read is said so and left out.
  pub fn load(files: &[PathBuf], preset: Option<GamePreset>, sets: &[String]) -> Self {
    let file: GameSettingsFile = files
      .iter()
      .find(|file| file.is_file())
      .map(|file| Self::read_settings_file(file))
      .unwrap_or_default();
    let preset: GamePreset = preset.or(file.preset).unwrap_or_default();
    let overrides: Vec<GameSettingOverride> = file
      .overrides
      .into_iter()
      .chain(sets.iter().filter_map(|argument| {
        let over: Option<GameSettingOverride> = GameSettingOverride::parse_argument(argument);

        if over.is_none() {
          log::warn!("Argument '--set {argument}' left out: write it as <path>=<value>");
        }

        over
      }))
      .collect();

    log::info!(
      "Settings start from the {preset:?} preset, {} changed over it",
      overrides.len()
    );

    preset.get_settings().merge(&overrides)
  }

  /// What the settings file at `path` holds; nothing, said so, where it cannot be read.
  pub(crate) fn read_settings_file(path: &Path) -> GameSettingsFile {
    match Ltx::read_from_file_standard(path) {
      Ok(ltx) => {
        log::info!("Settings read from '{}'", path.display());

        Self::read_settings_ltx(&ltx, &path.display().to_string())
      }
      Err(error) => {
        log::warn!(
          "Settings file '{}' left out, it cannot be read: {error}",
          path.display()
        );

        GameSettingsFile::default()
      }
    }
  }

  /// What a settings file read as `ltx` holds. A section names a group's dotted path (`[view.shadows.contact]`) and
  /// each of its keys a setting of that group; `[settings]` names the file's own choices, its preset among them.
  pub(crate) fn read_settings_ltx(ltx: &Ltx, origin: &str) -> GameSettingsFile {
    let mut file: GameSettingsFile = GameSettingsFile::default();

    for (section, fields) in ltx {
      for (key, value) in fields {
        if section.eq_ignore_ascii_case(Self::FILE_SECTION) {
          Self::read_file_choice(&mut file, key, value, origin);
        } else {
          file.overrides.push(GameSettingOverride {
            path: format!("{}.{}", section.trim(), key.trim()),
            value: value.trim().to_owned(),
            origin: origin.to_owned(),
          });
        }
      }
    }

    file
  }

  fn read_file_choice(file: &mut GameSettingsFile, key: &str, value: &str, origin: &str) {
    if !key.trim().eq_ignore_ascii_case(Self::PRESET_KEY) {
      log::warn!(
        "Settings '[{}] {key}' ({origin}) left out: the file names no such choice",
        Self::FILE_SECTION
      );

      return;
    }

    match GamePreset::from_str(value) {
      Ok(preset) => file.preset = Some(preset),
      Err(reason) => log::warn!("Preset '{value}' ({origin}) left out: {reason}"),
    }
  }
}
