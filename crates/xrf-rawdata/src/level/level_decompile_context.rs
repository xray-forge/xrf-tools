use xrf_editable_level::EditableLevelGuid;

use crate::level::level_object_names::LevelObjectNames;

/// What every conversion of one level shares as it decompiles: the scene's GUID, which each part carries, and the names
/// its objects take, unique across the scene as the editor keeps them.
#[derive(Clone, Debug)]
pub struct LevelDecompileContext {
  pub guid: EditableLevelGuid,
  pub names: LevelObjectNames,
}

impl LevelDecompileContext {
  /// The context of a scene of `guid` whose objects are not named yet.
  pub fn new(guid: EditableLevelGuid) -> Self {
    Self {
      guid,
      names: LevelObjectNames::default(),
    }
  }
}
