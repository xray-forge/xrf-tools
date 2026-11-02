use std::path::PathBuf;

use crate::XrayLogicalPath;

/// What a transaction's write names: a logical path, placed by what the search answers for it, or a file on disk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum XrayTransactionTarget {
  Path(XrayLogicalPath),
  File(PathBuf),
}
