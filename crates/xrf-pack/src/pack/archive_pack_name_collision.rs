use std::fmt::{Display, Formatter, Result as FormatResult};

use xrf_error::XrfError;
use xrf_utils::format_path;

use crate::pack::archive_pack_config::ArchivePackConfig;

/// Authored names of different files that one engine name would fold together.
///
/// The reading half of the registration rule in [`crate::pack::archive_pack_name_table`]: that module decides what
/// collides, this one decides how a person is told. Kept apart because the two change for different reasons — a
/// sharper diagnostic is not a change to what the engine folds.
#[derive(Debug)]
pub(crate) struct ArchivePackNameCollision {
  pub(crate) engine_name: String,
  pub(crate) spellings: Vec<String>,
}

impl ArchivePackNameCollision {
  /// Why a set whose files the engine would fold together stopped before anything was written.
  ///
  /// Every collision is named in one refusal rather than one per run, so a source tree with several is fixed in a
  /// single pass instead of a repack per name.
  pub(crate) fn describe_refusal(config: &ArchivePackConfig, collisions: &[Self]) -> XrfError {
    let described: Vec<String> = collisions.iter().map(ToString::to_string).collect();

    XrfError::new_invalid_error(format!(
      "Refusing to pack '{}': {} engine name(s) are claimed by more than one file, so only one of each could ever be \
       read. Rename or drop the extra spellings: {}",
      format_path(&config.source),
      collisions.len(),
      described.join("; ")
    ))
  }
}

impl Display for ArchivePackNameCollision {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
    let Some((last, head)) = self.spellings.split_last() else {
      return write!(formatter, "'{}' is claimed by no file", self.engine_name);
    };

    let head: Vec<String> = head.iter().map(|spelling| format!("'{spelling}'")).collect();
    let quantifier: &str = if self.spellings.len() == 2 { "both" } else { "all" };

    write!(
      formatter,
      "{} and '{last}' {quantifier} register as '{}'",
      head.join(", "),
      self.engine_name
    )
  }
}
