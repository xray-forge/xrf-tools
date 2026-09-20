use serde::{Deserialize, Serialize};

/// Where a texture coordinate sits and how wide the element holding it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelVertexCoordinate {
  pub(crate) offset: u16,
  pub(crate) is_tree: bool,
}
