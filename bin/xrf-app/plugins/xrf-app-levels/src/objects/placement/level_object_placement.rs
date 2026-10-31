use serde::Serialize;

use crate::objects::{LevelObjectShape, LevelPlacementDetails};

/// What one of the level's companion files records of a placement: the shapes it covers in its object's own space,
/// renderer space, none for one placed at a point, and what its file records of it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelObjectPlacement {
  pub shapes: Vec<LevelObjectShape>,
  pub details: LevelPlacementDetails,
}
