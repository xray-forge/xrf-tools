use std::sync::{Arc, Mutex};

use xrf_utils::StagedFile;

/// What a transaction writes to one path: bytes held whole, or a file its writer already streamed beside the target.
#[derive(Clone, Debug)]
pub(crate) enum XrayWriteContent {
  Bytes(Vec<u8>),
  /// Shared by every clone of the transaction and taken by the first commit that tries it, which publishes it or, when
  /// the commit fails, removes it: a staged write is spent by one commit.
  Staged(Arc<Mutex<Option<StagedFile>>>),
}
