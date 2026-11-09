use std::hash::BuildHasher;

use hashbrown::HashTable;
use rustc_hash::FxBuildHasher;

/// Finds a section's field by key among fields its owner keeps in written order, so a key set twice can take the place
/// it was first written in.
///
/// A record has a few fields or many: a spawn's graph point three, an ALife object dozens. A scan finds one of a few
/// faster than hashing it; past that a table of the fields' positions keeps a section of many from costing a scan per
/// field. The owner keeps the keys and names them by position.
#[derive(Default)]
pub(crate) struct LtxFieldIndex {
  /// The fields' positions by key, filled once there are more than a scan covers.
  positions: HashTable<usize>,
  count: usize,
}

impl LtxFieldIndex {
  /// Fields a section holds before they are found by key instead of by scanning them all.
  const SCANNED_FIELDS: usize = 8;

  /// Where `key` is among the fields, `key_of` naming the key at each position.
  pub(crate) fn find<'k>(&self, key: &str, key_of: impl Fn(usize) -> &'k str) -> Option<usize> {
    if self.count <= Self::SCANNED_FIELDS {
      (0..self.count).position(|index| key_of(index) == key)
    } else {
      self
        .positions
        .find(Self::hash(key), |&index| key_of(index) == key)
        .copied()
    }
  }

  /// Take in the field just added at the end, `key_of` naming the key at each position.
  pub(crate) fn push<'k>(&mut self, key_of: impl Fn(usize) -> &'k str) {
    self.count += 1;

    if self.count <= Self::SCANNED_FIELDS {
      return;
    }

    // Past the scanned few, every field so far is indexed once, then each as it comes.
    let first: usize = if self.count == Self::SCANNED_FIELDS + 1 {
      0
    } else {
      self.count - 1
    };

    for index in first..self.count {
      self
        .positions
        .insert_unique(Self::hash(key_of(index)), index, |&held| Self::hash(key_of(held)));
    }
  }

  /// Forget every field, for the next section.
  pub(crate) fn clear(&mut self) {
    if self.count > Self::SCANNED_FIELDS {
      self.positions.clear();
    }

    self.count = 0;
  }

  fn hash(text: &str) -> u64 {
    FxBuildHasher.hash_one(text)
  }
}

#[cfg(test)]
mod test {
  use crate::binding::LtxFieldIndex;

  #[test]
  fn finds_every_field_by_scan_and_by_hash() {
    let keys: Vec<String> = (0..30).map(|index| format!("key_{index}")).collect();
    let mut index: LtxFieldIndex = LtxFieldIndex::default();

    for count in 1..=keys.len() {
      index.push(|at| keys[at].as_str());

      for (position, key) in keys[..count].iter().enumerate() {
        assert_eq!(
          index.find(key, |at| keys[at].as_str()),
          Some(position),
          "{count} fields"
        );
      }

      assert_eq!(index.find("absent", |at| keys[at].as_str()), None);
    }

    index.clear();

    assert_eq!(
      index.find("key_0", |at| keys[at].as_str()),
      None,
      "a cleared index to hold nothing"
    );
  }
}
