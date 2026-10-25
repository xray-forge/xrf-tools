use serde::{Deserialize, Serialize};
use xrf_vfs::XrayRoots;

use crate::browse::TextureCatalogMode;

/// What a tree is listing: the roots and how it addresses them, restored after a frontend reload.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureBrowse {
  pub roots: XrayRoots,
  pub mode: TextureCatalogMode,
}
