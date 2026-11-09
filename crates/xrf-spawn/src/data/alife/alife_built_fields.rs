use xrf_math::Vector3d;

/// The spawn packet fields xrAI fills in when it builds a root spawn from the levels' spawn lists
/// (`level_spawn_constructor.cpp`, `game_spawn_constructor.cpp:270`), whatever the Level Editor wrote there. Each is
/// `None` for a class that has no such field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlifeBuiltFields {
  pub spawn_id: u16,
  pub game_vertex_id: Option<u16>,
  /// The distance to the game vertex.
  pub distance: Option<f32>,
  pub level_vertex_id: Option<u32>,
  /// A level changer's destination, which xrAI takes from its destination graph point.
  pub destination: Option<AlifeBuiltDestination>,
  /// An anomalous zone's offset into the root spawn's artefact spawn points.
  pub artefact_position_offset: Option<u32>,
}

/// A level changer's destination as xrAI fills it in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlifeBuiltDestination {
  pub game_vertex_id: u16,
  pub level_vertex_id: u32,
  pub position: Vector3d,
  pub direction: Vector3d,
}

impl AlifeBuiltFields {
  /// What a server entity's constructor holds before xrAI fills the fields in (`xrServer_Objects_ALife.cpp:247`,
  /// `:758`), set on every class that has them.
  pub const UNBUILT: Self = Self {
    spawn_id: u16::MAX,
    game_vertex_id: Some(u16::MAX),
    distance: Some(0.0),
    level_vertex_id: Some(u32::MAX),
    destination: Some(AlifeBuiltDestination {
      game_vertex_id: u16::MAX,
      level_vertex_id: u32::MAX,
      position: Vector3d::ZERO,
      direction: Vector3d::ZERO,
    }),
    artefact_position_offset: Some(0),
  };
}
