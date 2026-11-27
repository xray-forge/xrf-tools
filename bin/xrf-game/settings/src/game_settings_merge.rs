use serde_json::{Map, Number, Value};

use crate::game_setting_override::GameSettingOverride;
use crate::game_settings::GameSettings;

impl GameSettings {
  /// How an optional setting left unset is written.
  pub(crate) const UNSET: &'static str = "none";

  /// These settings with `overrides` applied in order, each as the type these settings hold at its path: a list
  /// replaced whole, a value spelled as the file or the argument writes it. One that names nothing these settings hold,
  /// or whose value is not one the setting takes, is logged and left out, so a mistyped line never stops the game.
  pub(crate) fn merge(self, overrides: &[GameSettingOverride]) -> Self {
    let mut tree: Value = match serde_json::to_value(&self) {
      Ok(tree) => tree,
      Err(error) => {
        log::warn!("The settings cannot be changed, every override left out: {error}");

        return self;
      }
    };

    for over in overrides {
      match Self::apply(&tree, over) {
        Ok(next) => {
          log::info!("Setting '{}' = '{}' ({})", over.path, over.value, over.origin);
          tree = next;
        }
        Err(reason) => log::warn!("Setting '{}' ({}) left out: {reason}", over.path, over.origin),
      }
    }

    let mut merged: Self = serde_json::from_value(tree).unwrap_or(self);

    merged.hold_to_bounds();

    merged
  }

  /// `tree` with `over` applied, checked by reading the whole settings back from it; why not where it cannot be.
  pub(crate) fn apply(tree: &Value, over: &GameSettingOverride) -> Result<Value, String> {
    let segments: Vec<String> = over.path.split('.').map(Self::format_camel_case).collect();
    let (leaf, groups) = segments.split_last().ok_or("an empty path names nothing")?;
    let mut candidates: Vec<String> = vec![over.value.clone()];

    // A variant may be written as the file's other keys are, `pbr_neutral` for `pbrNeutral`.
    if over.value.contains('_') {
      candidates.push(Self::format_camel_case(&over.value));
    }

    let mut reason: String = String::new();

    for value in candidates {
      let mut next: Value = tree.clone();
      let slot: &mut Value = Self::find_slot(&mut next, groups, leaf)?;

      *slot = Self::parse_like(slot, &value)?;

      match serde_json::from_value::<Self>(next.clone()) {
        Ok(_) => return Ok(next),
        Err(error) => reason = format!("'{value}' is not a value it takes: {error}"),
      }
    }

    // `none` leaves an optional setting unset, where no variant or text of that name was taken above.
    if over.value.eq_ignore_ascii_case(Self::UNSET) {
      let mut next: Value = tree.clone();

      *Self::find_slot(&mut next, groups, leaf)? = Value::Null;

      if serde_json::from_value::<Self>(next.clone()).is_ok() {
        return Ok(next);
      }
    }

    Err(reason)
  }

  /// The value `leaf` names in the group `groups` names under `tree`'s root.
  fn find_slot<'a>(tree: &'a mut Value, groups: &[String], leaf: &str) -> Result<&'a mut Value, String> {
    Self::find_group(tree, groups)?
      .get_mut(leaf)
      .ok_or_else(|| format!("the settings hold no '{leaf}' there"))
  }

  /// The group `groups` names under `tree`'s root.
  fn find_group<'a>(tree: &'a mut Value, groups: &[String]) -> Result<&'a mut Map<String, Value>, String> {
    let mut node: &mut Value = tree;

    for group in groups {
      node = node
        .get_mut(group.as_str())
        .ok_or_else(|| format!("the settings hold no group '{group}'"))?;
    }

    node
      .as_object_mut()
      .ok_or_else(|| String::from("a path through a value names no group"))
  }

  /// `text` as the kind of value `like` is: a flag, a number, a word, or a list of them, comma-separated.
  fn parse_like(like: &Value, text: &str) -> Result<Value, String> {
    match like {
      Value::Bool(_) => Self::parse_flag(text).map(Value::Bool),
      Value::Number(number) if number.is_u64() => text
        .parse::<u64>()
        .map(Value::from)
        .map_err(|error| format!("'{text}' is no whole number: {error}")),
      Value::Number(number) if number.is_i64() => text
        .parse::<i64>()
        .map(Value::from)
        .map_err(|error| format!("'{text}' is no whole number: {error}")),
      Value::Number(_) => text
        .parse::<f64>()
        .ok()
        .and_then(Number::from_f64)
        .map(Value::Number)
        .ok_or_else(|| format!("'{text}' is no number")),
      Value::String(_) => Ok(Value::String(text.to_owned())),
      Value::Array(items) => {
        let element: Value = items.first().cloned().unwrap_or(Value::Null);

        text
          .split(',')
          .map(str::trim)
          .filter(|item| !item.is_empty())
          .map(|item| Self::parse_like(&element, item))
          .collect::<Result<Vec<Value>, String>>()
          .map(Value::Array)
      }
      Value::Object(_) => Err(String::from("it names a group, not one value: name one of its fields")),
      Value::Null => Ok(serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.to_owned()))),
    }
  }

  /// A flag as LTX writes one: `true`, `on`, `yes` or `1`, and their opposites.
  fn parse_flag(text: &str) -> Result<bool, String> {
    match text.to_ascii_lowercase().as_str() {
      "true" | "on" | "yes" | "1" => Ok(true),
      "false" | "off" | "no" | "0" => Ok(false),
      _ => Err(format!("'{text}' is no flag: write true or false")),
    }
  }

  /// A name as the settings hold it: `shadow_radius` as `shadowRadius`, one already so unchanged.
  pub(crate) fn format_camel_case(name: &str) -> String {
    let mut formatted: String = String::with_capacity(name.len());
    let mut is_upper: bool = false;

    for character in name.trim().chars() {
      if character == '_' {
        is_upper = true;
      } else if is_upper {
        formatted.extend(character.to_uppercase());
        is_upper = false;
      } else {
        formatted.push(character);
      }
    }

    formatted
  }
}
