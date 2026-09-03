use std::collections::HashMap;
use std::fs;

use xrf_error::XrfResult;

use crate::pack::source::ArchivePackEntry;

/// Payloads already written to the current volume, so an identical file costs a descriptor row and nothing else.
///
/// Borrows the entries it records rather than copying anything out of them: the source table holds every entry for
/// the whole write, so a candidate needs neither its own path to prove a match nor its own name to say where an alias
/// points.
#[derive(Default)]
pub(crate) struct ArchiveAliasTable<'e> {
  /// Keyed by size and checksum, which several distinct payloads may share, so each key keeps every candidate.
  candidates: HashMap<(u32, u32), Vec<ArchiveAliasCandidate<'e>>>,
}

/// Where a payload an entry can share already sits in the current volume.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ArchiveAlias {
  pub(crate) offset: u32,
  pub(crate) size_compressed: u32,
}

/// One recorded payload and the entry it was written for, so a match can be proven rather than assumed and its source
/// named by the logical entry rather than by a host path.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ArchiveAliasCandidate<'e> {
  pub(crate) source: &'e ArchivePackEntry,
  pub(crate) alias: ArchiveAlias,
}

impl<'e> ArchiveAliasTable<'e> {
  /// Find an identical payload already in this volume, confirming the match byte for byte.
  ///
  /// Equal size and checksum is strong evidence and not proof; xrCompress re-reads the candidate too. Trusting the key
  /// alone would point an entry at another file's payload, which reads back as that other file rather than as an
  /// error, so the cost of the re-read buys the one guarantee worth paying for here.
  ///
  /// Answered by value rather than by reference: a candidate is two words that already borrow the source table, so
  /// handing back a borrow of this table would only stop the caller recording the entry it just proved a match for.
  pub(crate) fn find(&self, contents: &[u8], size_real: u32, crc: u32) -> XrfResult<Option<ArchiveAliasCandidate<'e>>> {
    let Some(candidates) = self.candidates.get(&(size_real, crc)) else {
      return Ok(None);
    };

    for candidate in candidates {
      if fs::read(&candidate.source.path)? == contents {
        return Ok(Some(*candidate));
      }
    }

    Ok(None)
  }

  /// Record a payload just written for `source`, so a later identical file can point at it.
  pub(crate) fn record(&mut self, source: &'e ArchivePackEntry, size_real: u32, crc: u32, alias: ArchiveAlias) {
    self
      .candidates
      .entry((size_real, crc))
      .or_default()
      .push(ArchiveAliasCandidate { source, alias });
  }

  /// Forget everything, because the volume the offsets addressed is closed.
  pub(crate) fn reset(&mut self) {
    self.candidates.clear();
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use super::{ArchiveAlias, ArchiveAliasTable};
  use crate::pack::source::ArchivePackEntry;

  const FIRST: &[u8] = b"the first payload";
  const SECOND: &[u8] = b"a different one!!";

  fn write(scope: &str, name: &str, contents: &[u8]) -> ArchivePackEntry {
    let path: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/{name}"));

    fs::create_dir_all(path.parent().expect("scope directory")).expect("scope directory");
    fs::write(&path, contents).expect("candidate file");

    ArchivePackEntry {
      name: name.to_string(),
      path,
    }
  }

  fn alias(offset: u32) -> ArchiveAlias {
    ArchiveAlias {
      offset,
      size_compressed: 17,
    }
  }

  /// The alias a lookup found, with the match itself proven by the name it points back to.
  fn find(table: &ArchiveAliasTable, contents: &[u8], size_real: u32, crc: u32, source: &str) -> Option<ArchiveAlias> {
    table.find(contents, size_real, crc).expect("lookup").map(|candidate| {
      assert_eq!(
        candidate.source.name, source,
        "the match names the entry whose payload it reuses"
      );

      candidate.alias
    })
  }

  #[test]
  fn an_identical_payload_points_at_the_copy_already_written() {
    let scope: &str = "an_identical_payload_points_at_the_copy_already_written";
    let first: ArchivePackEntry = write(scope, "first.bin", FIRST);
    let mut table: ArchiveAliasTable = ArchiveAliasTable::default();

    table.record(&first, 17, 42, alias(64));

    assert_eq!(find(&table, FIRST, 17, 42, "first.bin"), Some(alias(64)));
    assert_eq!(find(&table, FIRST, 17, 43, "first.bin"), None, "a different checksum");
    assert_eq!(find(&table, FIRST, 18, 42, "first.bin"), None, "a different size");
  }

  #[test]
  fn a_key_two_payloads_share_is_resolved_by_the_bytes() {
    // The one case where a wrong answer is silent: an entry aliased onto another file's payload reads back as that
    // other file. Both are recorded under one key here, which a real checksum collision would also produce.
    let scope: &str = "a_key_two_payloads_share_is_resolved_by_the_bytes";
    let first: ArchivePackEntry = write(scope, "first.bin", FIRST);
    let second: ArchivePackEntry = write(scope, "second.bin", SECOND);
    let mut table: ArchiveAliasTable = ArchiveAliasTable::default();

    table.record(&first, 17, 42, alias(64));

    assert_eq!(
      find(&table, SECOND, 17, 42, "first.bin"),
      None,
      "the key matches and the bytes do not, so nothing is shared"
    );

    table.record(&second, 17, 42, alias(128));

    assert_eq!(find(&table, FIRST, 17, 42, "first.bin"), Some(alias(64)));
    assert_eq!(
      find(&table, SECOND, 17, 42, "second.bin"),
      Some(alias(128)),
      "a candidate behind the first is still reachable"
    );
  }

  #[test]
  fn a_new_volume_shares_nothing_with_the_one_before_it() {
    let scope: &str = "a_new_volume_shares_nothing_with_the_one_before_it";
    let first: ArchivePackEntry = write(scope, "first.bin", FIRST);
    let mut table: ArchiveAliasTable = ArchiveAliasTable::default();

    table.record(&first, 17, 42, alias(64));
    table.reset();

    // An offset addresses one volume, so a surviving entry would point into the wrong file.
    assert_eq!(find(&table, FIRST, 17, 42, "first.bin"), None);
  }
}
