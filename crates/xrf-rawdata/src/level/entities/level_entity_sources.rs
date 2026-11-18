use xrf_spawn::{AlifeObject, LevelSpawnFile};

/// What a level's entities are decompiled from.
#[derive(Clone, Copy, Debug, Default)]
pub struct LevelEntitySources<'a> {
  pub spawn: Option<&'a LevelSpawnFile>,
  pub respawn: Option<&'a LevelSpawnFile>,
  /// The root spawn's objects on the level; where any are, they stand in for `spawn`'s entities but its graph points,
  /// which a root spawn keeps as graph vertices.
  pub root: &'a [AlifeObject],
  /// Names of the entities the Build cut from the geometry, which the decompile folded back into objects: left out of
  /// every list, as the Build writes them again.
  pub folded: &'a [String],
}
