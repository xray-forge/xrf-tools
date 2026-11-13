/// An edge of a level's game graph: the vertex it leads to, numbered on the level, and how far it runs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpawnGameEdge {
  pub target: u16,
  pub distance: f32,
}
