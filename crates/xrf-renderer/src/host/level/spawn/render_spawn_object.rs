use crate::host::level::spawn::render_spawn_group::RenderSpawnGroup;

/// One object of a level's spawn the viewer draws: the visual it stands as, where, and the group showing it.
#[derive(Clone, Copy, Debug)]
pub struct RenderSpawnObject {
  /// Its index among the level's spawned objects, which a pick names it by.
  pub index: u32,
  /// Its category, whether a new game releases it, and whether a detector must reveal it, which a view shows it by.
  pub group: RenderSpawnGroup,
  /// Its visual, by its index among [`crate::RenderLevelSpawn::visuals`].
  pub visual: u32,
  /// Its `XFORM` in renderer space, sixteen floats column by column.
  pub transform: [f32; 16],
}
