use crate::contract::render_spawn_category::RenderSpawnCategory;

/// One object of a level's spawn the viewer draws: the visual it stands as, where, and the group showing it.
#[derive(Clone, Copy, Debug)]
pub struct RenderSpawnObject {
  /// Its index among the level's spawned objects, which a pick names it by.
  pub index: u32,
  pub category: RenderSpawnCategory,
  /// Whether a new game releases it, which a view draws only when asked to.
  pub is_released: bool,
  /// Its visual, by its index among [`crate::RenderLevelSpawn::visuals`].
  pub visual: u32,
  /// Its `XFORM` in renderer space, sixteen floats column by column.
  pub transform: [f32; 16],
}
