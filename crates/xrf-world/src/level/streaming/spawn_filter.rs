use crate::level::streaming::hidden_spawn_groups::HiddenSpawnGroups;

/// What of a level's spawn a scene leaves out this frame: whole groups, and objects one by one.
#[derive(Clone, Copy, Debug)]
pub struct SpawnFilter<'a> {
  pub groups: HiddenSpawnGroups,
  /// By index among the level's spawned objects, ascending.
  pub objects: &'a [u32],
}

impl SpawnFilter<'_> {
  /// Whether the spawned object at `object` of `group` stands in the scene.
  pub fn is_shown(&self, group: u32, object: u32) -> bool {
    self.groups.is_shown(group) && self.objects.binary_search(&object).is_err()
  }
}
