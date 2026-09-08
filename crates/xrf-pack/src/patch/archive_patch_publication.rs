use serde::Serialize;

use crate::pack::ArchivePackResult;

/// What became of the difference a comparison found.
///
/// One value in place of the two booleans and eight copied fields this replaced. A patch result used to restate
/// `volumes`, `filesStored`, `sizeWritten` and the rest field by field, which meant every field added to
/// [`ArchivePackResult`] was silently absent here until someone noticed; and it carried an `isDryRun` flag beside an
/// empty volume list, leaving a reader to work out whether nothing was written because nothing was asked for or
/// because nothing differed. Those are three distinct outcomes, so they are three variants.
///
/// Tagged the way [`xrf_vfs::XrayAssetContainer`] is, so a consumer switches on `kind` rather than on the emptiness of
/// a list.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ArchivePatchPublication {
  /// Nothing was written, and the difference stands as reported.
  ///
  /// Ordinarily because the run was a comparison. Also where a publishing run was cancelled before it reached the
  /// write, which is why this does not claim nothing was *going* to be written — `outcome` is the field that says a
  /// run was stopped, and this one says only what is on disk.
  Compared,
  /// The run would have written, and the two worlds turned out to agree. No volume is a patch of nothing.
  Unnecessary,
  /// The difference was written, as this set.
  Published(ArchivePackResult),
}

impl ArchivePatchPublication {
  /// The volume set that was written, if one was.
  pub fn get_published(&self) -> Option<&ArchivePackResult> {
    match self {
      Self::Published(result) => Some(result),
      Self::Compared | Self::Unnecessary => None,
    }
  }

  /// Whether the run left anything on disk.
  pub fn is_published(&self) -> bool {
    matches!(self, Self::Published(_))
  }
}
