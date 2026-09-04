use std::collections::BTreeMap;

use xrf_ltx::LtxKeyOperation;

use crate::dltx_item::DltxItem;

/// Where one resolved field came from.
///
/// The engine keeps this per item and exposes it to Lua, because in a patched install the only way to explain a value
/// is to name the file that won it (`xr_ini.h`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DltxFieldOrigin {
  /// Lowercased base name of the winning file.
  pub file: String,
  /// Load rank of the winning statement. Negative means a mod file.
  pub depth: i32,
  /// Which operation produced the value.
  pub operation: LtxKeyOperation,
}

impl DltxFieldOrigin {
  pub fn of(item: &DltxItem) -> Self {
    Self {
      depth: item.depth,
      file: item.filename.clone(),
      operation: item.operation,
    }
  }

  /// Whether a mod file rather than the base tree set this value.
  ///
  /// The engine answers the same question by testing the file name for a `mod_` prefix.
  pub fn is_from_mod_file(&self) -> bool {
    self.file.starts_with("mod_")
  }
}

/// Which file won each resolved field.
#[derive(Debug, Default)]
pub struct DltxProvenance {
  origins: BTreeMap<(String, String), DltxFieldOrigin>,
}

impl DltxProvenance {
  pub(crate) fn record(&mut self, section: &str, key: &str, origin: DltxFieldOrigin) {
    self.origins.insert((String::from(section), String::from(key)), origin);
  }

  /// Drops a whole section, for one deleted after everything resolved.
  pub(crate) fn forget_section(&mut self, section: &str) {
    self.origins.retain(|(held, _), _| held != section);
  }

  /// Where one field came from.
  pub fn get(&self, section: &str, key: &str) -> Option<&DltxFieldOrigin> {
    self.origins.get(&(String::from(section), String::from(key)))
  }

  /// Every field a mod file is responsible for, as `(section, key)`.
  pub fn list_patched_fields(&self) -> Vec<(&str, &str)> {
    self
      .origins
      .iter()
      .filter(|(_, origin)| origin.is_from_mod_file())
      .map(|((section, key), _)| (section.as_str(), key.as_str()))
      .collect()
  }

  pub fn len(&self) -> usize {
    self.origins.len()
  }

  pub fn is_empty(&self) -> bool {
    self.origins.is_empty()
  }
}
