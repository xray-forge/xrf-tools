use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::volume::ArchiveFileDescriptor;

/// What a volume's header chunks declare: where its entries unpack and its name table.
pub(crate) struct ArchiveHeader {
  /// Root from `[header] entry_point` with its alias stripped, empty when the volume carries no metadata chunk.
  pub(crate) output_root_path: PathBuf,
  /// Directory under `$fs_root$` the engine registers the entries in, as [`crate::ArchiveDescriptor::mount_root`].
  pub(crate) mount_root: String,
  /// The name table, keyed by each entry's authored name.
  pub(crate) files: HashMap<Arc<str>, ArchiveFileDescriptor>,
}
