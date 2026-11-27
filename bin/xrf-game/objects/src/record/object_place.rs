use xrf_math::glam::Vec3;

/// Where an object stands, as the registry moves it (`ObjectRegistry::move_to`): its position in the engine's space,
/// the graph level, and its game and level vertices. The level is what the registry indexes objects by.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObjectPlace {
  pub position: Vec3,
  /// The graph level, by its id, none for a place on no level the graph holds.
  pub level: Option<u8>,
  pub game_vertex: Option<u16>,
  pub level_vertex: Option<u32>,
}
