//! What a translation job registers itself as, and what a writing one holds exclusively.
//!
//! Only the two writers take a lease. A verification reads, and two readers of one project have nothing to collide
//! over.

use std::path::Path;

use crate::core::jobs::to_comparable_path;

/// What a build registers itself as.
///
/// The frontend spells the same strings in `EJobKind`, which is the wire contract this side owns.
pub const BUILD_JOB_KIND: &str = "translations.build";

/// What an import registers itself as.
pub const PARSE_JOB_KIND: &str = "translations.parse";

/// What a verification registers itself as.
pub const VERIFY_JOB_KIND: &str = "translations.verify";

/// Prefix of every lease over a translation output directory.
///
/// Named for what it protects rather than for who takes it: a build writes string tables into it and an import writes
/// JSON sources, and both would be refused by a run of the other kind already there.
const OUTPUT_DIRECTORY_LEASE: &str = "translations.output";

/// The directory a run would write into, as a lease key.
///
/// Both writers own the whole directory for the duration: a build replaces the tables it compiles and an import merges
/// into the sources it finds, so a second run there reads files the first is midway through replacing.
pub fn to_output_lease_key(output_dir: &Path) -> String {
  format!("{OUTPUT_DIRECTORY_LEASE}:{}", to_comparable_path(output_dir))
}
