/// What the per-level pass needs of the merged graph: where the level's vertices start and the first one's node.
pub(crate) struct SpawnLevelVertices {
  pub(crate) first: u16,
  pub(crate) first_node: u32,
}
