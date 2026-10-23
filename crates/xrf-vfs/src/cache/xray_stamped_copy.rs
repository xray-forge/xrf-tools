use crate::{XrayFileStamp, XrayRevision};

/// A loose copy a retained value was parsed from, as the read saw its file.
#[derive(Debug)]
pub(crate) struct XrayStampedCopy {
  pub(crate) opening: u64,
  pub(crate) path: String,
  pub(crate) revision: XrayRevision,
  pub(crate) stamp: XrayFileStamp,
}
