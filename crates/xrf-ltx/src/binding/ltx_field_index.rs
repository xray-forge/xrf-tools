use std::hash::BuildHasher;

use hashbrown::HashTable;
use rustc_hash::FxBuildHasher;

/// Finds a section's field by key among fields its owner keeps in written order and names by position.
///
/// A spawn's graph point has three fields, an ALife object dozens: a scan beats hashing a few, and past that a table
/// of positions saves a scan per field.
#[derive(Default)]
pub(crate) struct LtxFieldIndex {
  /// The fields' positions by key, filled past [`Self::SCANNED_FIELDS`].
  positions: HashTable<usize>,
  count: usize,
}

impl LtxFieldIndex {
  /// Fields found by scanning before the index hashes them.
  const SCANNED_FIELDS: usize = 8;

  /// The position of `key`, `key_of` naming the key at each position.
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

  /// Indexes the field just added at the end.
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

  /// Forgets every field, for the next section.
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
