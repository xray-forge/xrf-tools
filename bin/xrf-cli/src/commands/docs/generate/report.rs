use std::path::Path;

use serde::Serialize;
use xrf_report::Status;

/// What `docs generate` did to a reference tree, in either of its modes.
///
/// One shape for both, because a caller asking "is the committed reference current?" and one asking
/// "what was just written?" are after the same four facts: how many pages the generator produces,
/// where they live, which mode ran, and what did not match. A report saved for later no longer has
/// the invocation to tell it, which is why `isCheck` is carried rather than inferred.
///
/// `stale` and `removed` are the same drift seen from the two modes: a check names what it found, a
/// write deletes the pages that no longer belong and names those. Each is empty in the other mode.
/// `stale` is deposited before the verdict becomes an outcome, since a failing check is exactly when
/// the drift explaining it is worth reporting.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocsGenerateReport {
  directory: String,
  is_check: bool,
  pages: usize,
  removed: Vec<String>,
  stale: Vec<String>,
  status: Status,
}

impl DocsGenerateReport {
  /// What a `--check` run compared, and the drift it found.
  ///
  /// Each finding is the line the check printed - `missing:`, `outdated:` or `unexpected:` followed
  /// by the page name - so a consumer reading `result` alone knows which pages to regenerate.
  pub fn checked(directory: &Path, pages: usize, stale: Vec<String>) -> Self {
    Self {
      directory: xrf_utils::to_portable_path_string(directory),
      is_check: true,
      pages,
      removed: Vec::new(),
      status: Status::from_is_valid(stale.is_empty()),
      stale,
    }
  }

  /// What a writing run produced, and the pages it deleted on the way.
  ///
  /// `removed` is what a group rename leaves behind, reported because the write is the only moment
  /// anyone learns those pages existed. The status is passed because the tree is current afterwards.
  pub fn written(directory: &Path, pages: usize, removed: Vec<String>) -> Self {
    Self {
      directory: xrf_utils::to_portable_path_string(directory),
      is_check: false,
      pages,
      removed,
      stale: Vec::new(),
      status: Status::Passed,
    }
  }
}
