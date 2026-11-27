use serde_json::{Map, Value};

use crate::game_setting_entry::GameSettingEntry;
use crate::game_setting_override::GameSettingOverride;
use crate::game_settings::GameSettings;

impl GameSettings {
  /// Every setting these settings hold, in path order, each as the console and the settings file write it; a group
  /// that holds none, a list of groups among them, lists nothing.
  pub fn get_entries(&self) -> Vec<GameSettingEntry> {
    let mut entries: Vec<GameSettingEntry> = Vec::new();

    if let Ok(Value::Object(root)) = serde_json::to_value(self) {
      Self::collect_entries(&root, "", &mut entries);
    }

    entries.sort_by(|left, right| left.path.cmp(&right.path));

    entries
  }

  /// These settings with the one at `path` (dotted, in snake case or as the settings hold it) set to `value`, read as
  /// the settings file reads it and held to the renderer's bounds; why not where the path names no setting or the
  /// value is not one it takes.
  ///
  /// # Errors
  ///
  /// Returns why the setting cannot take the value.
  pub fn with_setting(&self, path: &str, value: &str) -> Result<Self, String> {
    let tree: Value = serde_json::to_value(self).map_err(|error| format!("the settings cannot be read: {error}"))?;
    let next: Value = Self::apply(
      &tree,
      &GameSettingOverride {
        path: path.to_owned(),
        value: value.to_owned(),
        origin: String::from("console"),
      },
    )?;

    let mut settings: Self =
      serde_json::from_value(next).map_err(|error| format!("'{value}' is not a value it takes: {error}"))?;

    settings.hold_to_bounds();

    Ok(settings)
  }

  /// Adds every value under `group` to `entries`, `prefix` its path.
  fn collect_entries(group: &Map<String, Value>, prefix: &str, entries: &mut Vec<GameSettingEntry>) {
    for (key, value) in group {
      let path: String = format!("{prefix}{}", Self::format_snake_case(key));

      match value {
        Value::Object(inner) => Self::collect_entries(inner, &format!("{path}."), entries),
        Value::Array(items) if items.iter().any(Value::is_object) => {}
        _ => entries.push(GameSettingEntry {
          path,
          value: Self::format_value(value),
        }),
      }
    }
  }

  /// `value` as the settings file writes it: a variant in snake case, other text as it is, a list comma-separated.
  fn format_value(value: &Value) -> String {
    match value {
      Value::String(text) if Self::is_variant(text) => Self::format_snake_case(text),
      Value::String(text) => text.clone(),
      Value::Array(items) => items.iter().map(Self::format_value).collect::<Vec<String>>().join(", "),
      Value::Null => String::from(Self::UNSET),
      other => other.to_string(),
    }
  }

  /// Whether `text` is a variant's name as the settings hold one, `pbrNeutral`: a lowercase letter, then letters and
  /// digits.
  fn is_variant(text: &str) -> bool {
    text.starts_with(|character: char| character.is_ascii_lowercase())
      && text.chars().all(|character| character.is_ascii_alphanumeric())
  }

  /// A name as the settings file writes it: `shadowRadius` as `shadow_radius`.
  fn format_snake_case(name: &str) -> String {
    let mut formatted: String = String::with_capacity(name.len() + 4);

    for character in name.chars() {
      if character.is_ascii_uppercase() {
        formatted.push('_');
        formatted.push(character.to_ascii_lowercase());
      } else {
        formatted.push(character);
      }
    }

    formatted
  }
}
