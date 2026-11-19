use xrf_math::Vector3d;

/// One `[levels]` entry of `configs/game_levels.ltx`, as `read_levels` takes it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpawnGameLevel {
  /// Its section, `level108` for vanilla's zaton.
  pub section: String,
  /// Its directory under `levels/`.
  pub name: String,
  /// The name a graph point names it by in a connection.
  pub caption: String,
  pub id: u8,
  /// Where it stands in the world the levels share.
  pub offset: Vector3d,
}
