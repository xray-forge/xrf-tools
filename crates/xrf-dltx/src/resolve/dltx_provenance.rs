use std::collections::BTreeMap;
use std::sync::Arc;

use crate::resolve::dltx_field_origin::DltxFieldOrigin;

/// Which file won each resolved field.
#[derive(Debug, Default)]
pub struct DltxProvenance {
  origins: BTreeMap<Arc<str>, BTreeMap<Arc<str>, DltxFieldOrigin>>,
}

impl DltxProvenance {
  pub(crate) fn record(&mut self, section: &Arc<str>, key: &Arc<str>, origin: DltxFieldOrigin) {
    self
      .origins
      .entry(Arc::clone(section))
      .or_default()
      .insert(Arc::clone(key), origin);
  }

  /// Drops a whole section, for one deleted after everything resolved.
  pub(crate) fn forget_section(&mut self, section: &str) {
    self.origins.remove(section);
  }

  /// Where one field came from.
  pub fn get(&self, section: &str, key: &str) -> Option<&DltxFieldOrigin> {
    self.origins.get(section).and_then(|fields| fields.get(key))
  }

  /// Every field a mod file is responsible for, as `(section, key)`.
  pub fn list_patched_fields(&self) -> Vec<(&str, &str)> {
    self
      .origins
      .iter()
      .flat_map(|(section, fields)| {
        fields
          .iter()
          .filter(|(_, origin)| origin.is_from_mod_file())
          .map(|(key, _)| (&**section, &**key))
      })
      .collect()
  }
}
