use serde::{Deserialize, Serialize};

/// Extension the produced volumes carry, which also decides how the engine treats a missing header.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchiveVolumeExtension {
  #[default]
  Db,
  Xdb,
}

impl ArchiveVolumeExtension {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Db => "db",
      Self::Xdb => "xdb",
    }
  }
}
