use xrf_spawn::AlifeObject;

use crate::level::build::level_build_breakable_part::LevelBuildBreakablePart;

/// The breakables a build made: each part's skeleton, the entities placing them, and the parts it could not make one
/// of.
pub(crate) struct LevelBuildBreakableExport {
  pub(crate) parts: Vec<LevelBuildBreakablePart>,
  pub(crate) entities: Vec<AlifeObject>,
  /// Each part refused, by its name, with why.
  pub(crate) refused: Vec<(String, String)>,
}
