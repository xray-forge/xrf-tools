use serde::{Deserialize, Serialize};
use xrf_vfs::XrayRoots;

/// The texture a Save as copies, and the name the copy takes.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TextureSaveAsTarget {
  /// The roots the texture was opened from; the copy is written into them.
  pub roots: XrayRoots,
  /// The original's texture name, such as `ston\ston_beton05`.
  pub reference: String,
  /// The copy's texture name, in the same form. Never a path on disk.
  pub destination: String,
}
