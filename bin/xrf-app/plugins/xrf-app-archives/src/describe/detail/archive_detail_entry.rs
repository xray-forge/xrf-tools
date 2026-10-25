use serde::Serialize;

use crate::describe::detail::archive_detail_model::ArchiveDetailModel;

/// One object of a level's detail library, and how much of the level it is planted across.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveDetailEntry {
  /// Position in the library, which is what a slot's six bits address.
  pub index: u32,
  /// Slot corners planting this object across the whole grid.
  pub planted_corners: u64,
  pub model: ArchiveDetailModel,
}
