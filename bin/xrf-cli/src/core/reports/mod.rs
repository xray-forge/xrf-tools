//! Report shapes more than one command answers with.
//!
//! A command that owns its shape keeps it beside the command, its root in `<command>/report.rs`. These are the shapes
//! several commands genuinely share: `particle pack` and `spawn pack` do the same kind of thing to different bytes, and
//! reporting that in two near-identical shapes would be the per-command drift the reporting contract exists to remove.
//!
//! Paths are portable display strings. Serializing a `PathBuf` fails outright on a host name that is
//! not valid Unicode, and a report describes a run rather than addressing one.

mod file_conversion_report;
mod file_verify_report;
mod skipped_mount_report;

pub use file_conversion_report::FileConversionReport;
pub use file_verify_report::FileVerifyReport;
pub use skipped_mount_report::SkippedMountReport;
