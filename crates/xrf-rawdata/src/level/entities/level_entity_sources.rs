use xrf_spawn::{AlifeObject, LevelSpawnFile};

/// What a level's entities are decompiled from.
#[derive(Clone, Copy, Debug, Default)]
pub struct LevelEntitySources<'a> {
  pub spawn: Option<&'a LevelSpawnFile>,
  pub respawn: Option<&'a LevelSpawnFile>,
  /// The root spawn's objects standing on the level. Where it holds any, they stand in for `spawn`'s
  /// entities but its graph points, which a root spawn keeps as graph vertices instead.
  pub root: &'a [AlifeObject],
  /// The names of entities the Build exported from the scene's geometry, which the decompile folded back into objects:
  /// left out of every list, as the Build writes them again.
  pub folded: &'a [String],
}
