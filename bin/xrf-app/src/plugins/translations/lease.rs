//! What a translation job registers itself as, and what a writing one holds exclusively.
//!
//! Only the writers take a lease. A verification and a format check read, and two readers of one project have nothing
//! to collide over.

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

/// What a formatting run registers itself as.
pub const FORMAT_JOB_KIND: &str = "translations.format";

/// What a formatting check registers itself as.
///
/// Separate from the rewrite it reports on: one answers a question and the other changes the files, so they are
/// different work to watch, to attribute, and to decide about. Only the rewrite takes a lease.
pub const CHECK_FORMAT_JOB_KIND: &str = "translations.check-format";

/// Prefix of every lease over a translation output directory.
///
/// Named for what it protects rather than for who takes it: a build writes string tables into it, an import writes JSON
/// sources and a format rewrites them, and each would be refused by a run of another kind already there.
const OUTPUT_DIRECTORY_LEASE: &str = "translations.output";

/// The directory a run would write into, as a lease key.
///
/// Every writer owns the whole directory for the duration: a build replaces the tables it compiles, an import merges
/// into the sources it finds and a format rewrites them, so a second run there reads files the first is midway through
/// replacing.
pub fn to_output_lease_key(output_dir: &Path) -> String {
  format!("{OUTPUT_DIRECTORY_LEASE}:{}", to_comparable_path(output_dir))
}
