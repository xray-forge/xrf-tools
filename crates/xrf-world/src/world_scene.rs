use xrf_renderer::RenderViewportId;

use crate::level::world_level::WorldLevel;

/// A level as the world plays it for every viewport showing it: its streaming, posing and effects, the key sources of
/// the same level share, and the viewports showing it in the order they began to, the first driving what it simulates
/// by its camera as the game's one actor would.
pub struct WorldScene {
  pub level: WorldLevel,
  /// What another source of the same level names, none for one no other shares.
  pub key: Option<String>,
  pub viewers: Vec<RenderViewportId>,
}

impl WorldScene {
  /// The viewport driving what it simulates: the first still showing it.
  pub fn get_driver(&self) -> Option<RenderViewportId> {
    self.viewers.first().copied()
  }
}
