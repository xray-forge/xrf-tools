/// Files of a level's directory a step reads by what their names hold rather than by a name it knows: those of
/// `directory` whose name `takes` accepts, such as the parts the Build cuts out of the geometry, numbered as it found
/// them.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LevelSceneListing {
  /// The directory under the level's, by its name.
  pub(crate) directory: &'static str,
  pub(crate) takes: fn(&str) -> bool,
}
