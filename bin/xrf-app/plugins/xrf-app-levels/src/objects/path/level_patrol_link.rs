use serde::Serialize;

/// One way a patrol goes on from a point: the point it leads to, by index along the path, and how likely it is taken
/// among the point's others.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelPatrolLink {
  pub from: u32,
  pub to: u32,
  pub weight: f32,
}
