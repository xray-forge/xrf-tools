use std::path::Path;

use xrf_vfs::XrayLogicalPath;

/// What a verification tells its listener while it runs.
///
/// Borrowed from the run, so emitting one costs a call. It is said inside the per-field loop, so the run builds one only
/// when something listens.
#[derive(Clone, Copy, Debug)]
pub enum LtxVerifyEvent<'e> {
  /// One field checked against its section's scheme: the entry point being verified, where that entry point is found,
  /// and the section and field judged.
  FieldChecked {
    entry: &'e XrayLogicalPath,
    path: &'e Path,
    section: &'e str,
    field: &'e str,
  },
}
