use std::any::Any;
use std::sync::Arc;

use crate::XrayFileStamp;

/// One retained value, with what it weighs against the budget and when it was retained.
#[derive(Debug)]
pub(crate) struct XrayCacheEntry {
  pub(crate) value: Arc<dyn Any + Send + Sync>,
  pub(crate) bytes: u64,
  /// How the copy's file looked when it was read, for a loose copy.
  pub(crate) stamp: Option<XrayFileStamp>,
  /// When it was retained, against every other entry, so the oldest goes first.
  pub(crate) order: u64,
}
