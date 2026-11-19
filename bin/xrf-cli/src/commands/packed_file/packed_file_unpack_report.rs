use serde::Serialize;

use crate::core::reports::FileConversionReport;

/// What an `unpack` or `re-unpack` read and wrote, beside what its format says about the export
/// ([`PackedFileUnpacked`](crate::commands::packed_file::PackedFileUnpacked)).
#[derive(Debug, Serialize)]
pub(crate) struct PackedFileUnpackReport<U> {
  #[serde(flatten)]
  conversion: FileConversionReport,
  #[serde(flatten)]
  unpacked: U,
}

impl<U: Serialize> PackedFileUnpackReport<U> {
  pub(crate) fn new(conversion: FileConversionReport, unpacked: U) -> Self {
    Self { conversion, unpacked }
  }
}
