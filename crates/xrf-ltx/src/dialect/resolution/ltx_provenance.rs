use std::collections::HashMap;
use std::sync::Arc;

use fxhash::FxBuildHasher;

use crate::dialect::LtxFieldOrigin;

/// Where every resolved field came from, grouped by the section holding it.
///
/// Nested rather than keyed by a `(section, key)` pair, for two reasons. A pair key cannot be looked up from two
/// `&str` without building the owned halves first, so every read would allocate; and the question a caller actually
/// asks is "explain this section", which a flat map answers only by scanning. Both levels key on `Arc<str>`, which
/// borrows as `str`, so a lookup allocates nothing.
///
/// Storage is private: this is a record with two questions, not a map that happens to hold origins.
///
/// One known limitation, pinned by `ltx::tests::field_provenance`: the root section is the only section that merges
/// across files instead of colliding, so when two files of one tree both write root fields, every root field reports
/// the first file merged. Every named section is exact, because `Ltx::merge_sections_from` refuses a duplicate of one.
#[derive(Debug, Default)]
pub struct LtxProvenance {
  sections: HashMap<Arc<str>, HashMap<Arc<str>, LtxFieldOrigin, FxBuildHasher>, FxBuildHasher>,
}

impl LtxProvenance {
  /// Whether nothing was recorded, which is the state of any resolution that was not asked for provenance.
  pub fn is_empty(&self) -> bool {
    self.sections.is_empty()
  }

  /// How many sections carry a record.
  pub fn len(&self) -> usize {
    self.sections.len()
  }

  /// Where one field came from.
  pub fn get(&self, section: &str, key: &str) -> Option<&LtxFieldOrigin> {
    self.sections.get(section)?.get(key)
  }

  /// Every recorded field of one section, in no particular order.
  ///
  /// Answers an empty iterator both for a section nothing recorded and for a section that does not exist, because a
  /// caller holding the resolved section already knows which it is looking at.
  pub fn iter_section(&self, section: &str) -> impl Iterator<Item = (&str, &LtxFieldOrigin)> {
    self
      .sections
      .get(section)
      .into_iter()
      .flat_map(|fields| fields.iter().map(|(key, origin)| (&**key, origin)))
  }

  /// Records where one field came from, replacing any earlier answer for it.
  ///
  /// Public for the same reason [`crate::Section::set_origin`] is: a dialect lives in another crate and has to fill in
  /// what it resolved. Reading stays through [`Self::get`] and [`Self::iter_section`].
  pub fn insert(&mut self, section: Arc<str>, key: Arc<str>, origin: LtxFieldOrigin) {
    self.sections.entry(section).or_default().insert(key, origin);
  }

  /// Records a whole section's fields at once, which is how a dialect fills this in.
  pub(crate) fn insert_section(&mut self, section: Arc<str>, fields: HashMap<Arc<str>, LtxFieldOrigin, FxBuildHasher>) {
    self.sections.insert(section, fields);
  }

  /// Give back the growth room the record no longer needs.
  pub(crate) fn shrink_to_fit(&mut self) {
    self.sections.shrink_to_fit();

    for fields in self.sections.values_mut() {
      fields.shrink_to_fit();
    }
  }
}
