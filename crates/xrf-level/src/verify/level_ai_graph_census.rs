use serde::Serialize;

/// What a level's AI map held, beside what its graph check found.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelAiGraphCensus {
  pub nodes: u32,
  /// Links naming a node of the map, each direction counted on its own.
  pub links: u64,
  /// Links naming the node they leave, which the engine's path search cannot take.
  pub self_links: u64,
  /// Links whose node does not link back across the same edge.
  pub one_way_links: u64,
  /// The nodes such a link reaches, as xrAI's `-verify` counts its "single linked nodes".
  pub one_way_nodes: u32,
  /// Parts of the map no link joins, a link counting either way.
  pub components: u32,
  pub largest_component: u32,
}
