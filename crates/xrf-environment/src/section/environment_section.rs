use std::collections::BTreeMap;

use serde::Serialize;
use xrf_engine_target::XrayEngine;

use crate::key::{EnvironmentDefault, EnvironmentKey, EnvironmentValue};
use crate::section::environment_origin::EnvironmentOrigin;

/// One section of an environment config as authored: every key the engine reads that it writes, as the engine reads
/// it, and every other key as written.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", bound(serialize = "K: Serialize"))]
pub struct EnvironmentSection<K: EnvironmentKey> {
  /// The section's name as its header writes it.
  pub name: String,
  /// The config it was read from, as a logical path.
  pub file: String,
  /// Every key of the table the section writes, whether or not this engine reads it.
  pub values: BTreeMap<K, EnvironmentValue>,
  /// Keys outside the table, as written, so a writer can keep them.
  pub extras: BTreeMap<String, String>,
  /// Where each written key's value came from, for a read that was asked to record it; empty otherwise.
  pub origins: BTreeMap<String, EnvironmentOrigin>,
}

impl<K: EnvironmentKey> EnvironmentSection<K> {
  /// A section with nothing in it yet.
  pub fn new(name: impl Into<String>, file: impl Into<String>) -> Self {
    Self {
      extras: BTreeMap::new(),
      file: file.into(),
      name: name.into(),
      origins: BTreeMap::new(),
      values: BTreeMap::new(),
    }
  }

  /// Whether the section writes the key.
  pub fn has(&self, key: K) -> bool {
    self.values.contains_key(&key)
  }

  /// The value as written and read, where the section writes it.
  pub fn get(&self, key: K) -> Option<&EnvironmentValue> {
    self.values.get(&key)
  }

  /// The number the engine holds for the key: written, or its default.
  pub fn get_number(&self, key: K, engine: XrayEngine) -> f32 {
    match self.get(key).and_then(EnvironmentValue::as_number) {
      Some(number) => number,
      None => match key.get_use(engine).get_default() {
        Some(EnvironmentDefault::Number(number)) => number,
        _ => 0.0,
      },
    }
  }

  /// The integer the engine holds for the key: written, or its default.
  pub fn get_integer(&self, key: K, engine: XrayEngine) -> i32 {
    match self.get(key).and_then(EnvironmentValue::as_integer) {
      Some(integer) => integer,
      None => match key.get_use(engine).get_default() {
        Some(EnvironmentDefault::Integer(integer)) => integer,
        _ => 0,
      },
    }
  }

  /// The vector the engine holds for the key, `N` components wide: written, each component `r_fvector` did not read
  /// at zero as it starts them, or the default whole where not written.
  pub fn get_vector<const N: usize>(&self, key: K, engine: XrayEngine) -> [f32; N] {
    let components: &[f32] = match self.get(key).and_then(EnvironmentValue::as_vector) {
      Some(written) => written,
      None => match key.get_use(engine).get_default() {
        Some(EnvironmentDefault::Vector(components)) => components,
        _ => &[],
      },
    };

    std::array::from_fn(|index| components.get(index).copied().unwrap_or(0.0))
  }

  /// The flag the engine holds for the key: written, or its default.
  pub fn get_flag(&self, key: K, engine: XrayEngine) -> bool {
    match self.get(key).and_then(EnvironmentValue::as_flag) {
      Some(flag) => flag,
      None => matches!(key.get_use(engine).get_default(), Some(EnvironmentDefault::Flag(true))),
    }
  }

  /// The text the engine holds for the key: written, or its default.
  pub fn get_text(&self, key: K, engine: XrayEngine) -> &str {
    match self.get(key).and_then(EnvironmentValue::as_text) {
      Some(text) => text,
      None => match key.get_use(engine).get_default() {
        Some(EnvironmentDefault::Text(text)) => text,
        _ => "",
      },
    }
  }

  /// The items of the list the engine walks for the key; none where it is not written.
  pub fn get_list(&self, key: K) -> &[String] {
    self.get(key).and_then(EnvironmentValue::as_list).unwrap_or(&[])
  }
}
