use serde::Serialize;
use xrf_environment::{Thunderbolt, ThunderboltCollection, ThunderboltSettings};

/// What a level's weather strikes with: the collections its cycles and effects name, their bolts, and where every
/// bolt is struck from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelThunderbolts {
  pub collections: Vec<ThunderboltCollection>,
  pub thunderbolts: Vec<Thunderbolt>,
  pub settings: Option<ThunderboltSettings>,
}
