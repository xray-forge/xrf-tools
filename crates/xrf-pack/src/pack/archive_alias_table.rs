use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;

/// Payloads already written to the current volume, so an identical file costs a descriptor row and nothing else.
#[derive(Default)]
pub(crate) struct ArchiveAliasTable {
  /// Keyed by size and checksum, which several distinct payloads may share, so each key keeps every candidate.
  candidates: HashMap<(u32, u32), Vec<ArchiveAliasCandidate>>,
}

/// Where a payload an entry can share already sits in the current volume.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ArchiveAlias {
  pub(crate) offset: u32,
  pub(crate) size_compressed: u32,
}

/// One recorded payload, kept with the file it came from so a match can be proven rather than assumed.
struct ArchiveAliasCandidate {
  path: PathBuf,
  alias: ArchiveAlias,
}

impl ArchiveAliasTable {
  /// Find an identical payload already in this volume, confirming the match byte for byte.
  ///
  /// Equal size and checksum is strong evidence and not proof; xrCompress re-reads the candidate too. Trusting the key
  /// alone would point an entry at another file's payload, which reads back as that other file rather than as an
  /// error, so the cost of the re-read buys the one guarantee worth paying for here.
  pub(crate) fn find(&self, contents: &[u8], size_real: u32, crc: u32) -> XrfResult<Option<ArchiveAlias>> {
    let Some(candidates) = self.candidates.get(&(size_real, crc)) else {
      return Ok(None);
    };

    for candidate in candidates {
      if fs::read(&candidate.path)? == contents {
        return Ok(Some(candidate.alias));
      }
    }

    Ok(None)
  }

  /// Record a payload just written, so a later identical file can point at it.
  pub(crate) fn record(&mut self, path: &Path, size_real: u32, crc: u32, alias: ArchiveAlias) {
    self
      .candidates
      .entry((size_real, crc))
      .or_default()
      .push(ArchiveAliasCandidate {
        path: path.to_path_buf(),
        alias,
      });
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

  const FIRST: &[u8] = b"the first payload";
  const SECOND: &[u8] = b"a different one!!";

  fn write(scope: &str, name: &str, contents: &[u8]) -> PathBuf {
    let path: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/{name}"));

    fs::create_dir_all(path.parent().expect("scope directory")).expect("scope directory");
    fs::write(&path, contents).expect("candidate file");

    path
  }

  fn alias(offset: u32) -> ArchiveAlias {
    ArchiveAlias {
      offset,
      size_compressed: 17,
    }
  }

  #[test]
  fn an_identical_payload_points_at_the_copy_already_written() {
    let scope: &str = "an_identical_payload_points_at_the_copy_already_written";
    let mut table: ArchiveAliasTable = ArchiveAliasTable::default();

    table.record(&write(scope, "first.bin", FIRST), 17, 42, alias(64));

    assert_eq!(table.find(FIRST, 17, 42).expect("lookup"), Some(alias(64)));
    assert_eq!(table.find(FIRST, 17, 43).expect("lookup"), None, "a different checksum");
    assert_eq!(table.find(FIRST, 18, 42).expect("lookup"), None, "a different size");
  }

  #[test]
  fn a_key_two_payloads_share_is_resolved_by_the_bytes() {
    // The one case where a wrong answer is silent: an entry aliased onto another file's payload reads back as that
    // other file. Both are recorded under one key here, which a real checksum collision would also produce.
    let scope: &str = "a_key_two_payloads_share_is_resolved_by_the_bytes";
    let mut table: ArchiveAliasTable = ArchiveAliasTable::default();

    table.record(&write(scope, "first.bin", FIRST), 17, 42, alias(64));

    assert_eq!(
      table.find(SECOND, 17, 42).expect("lookup"),
      None,
      "the key matches and the bytes do not, so nothing is shared"
    );

    table.record(&write(scope, "second.bin", SECOND), 17, 42, alias(128));

    assert_eq!(table.find(FIRST, 17, 42).expect("lookup"), Some(alias(64)));
    assert_eq!(
      table.find(SECOND, 17, 42).expect("lookup"),
      Some(alias(128)),
      "a candidate behind the first is still reachable"
    );
  }

  #[test]
  fn a_new_volume_shares_nothing_with_the_one_before_it() {
    let scope: &str = "a_new_volume_shares_nothing_with_the_one_before_it";
    let mut table: ArchiveAliasTable = ArchiveAliasTable::default();

    table.record(&write(scope, "first.bin", FIRST), 17, 42, alias(64));
    table.reset();

    // An offset addresses one volume, so a surviving entry would point into the wrong file.
    assert_eq!(table.find(FIRST, 17, 42).expect("lookup"), None);
  }
}
