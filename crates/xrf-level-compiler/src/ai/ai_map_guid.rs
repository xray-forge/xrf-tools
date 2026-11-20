use serde::{Deserialize, Serialize};

/// Where a compiled `level.ai`'s graph identity came from.
///
/// xrAI gives every run a new random identity, which the root spawn's game graph and cross table must carry for the
/// level to load. Ours is the replaced map's when the graph a spawn indexes is unchanged, so the existing spawn still
/// loads the level; otherwise one derived from the nodes, so the same input gives the same file.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AiMapGuid {
  /// The replaced map's: its nodes are as many, in the same order, at the same cells and with the same links.
  Kept,
  /// Hashed from the compiled nodes, there being no map to replace or its graph having changed.
  Derived,
}
