use xrf_spawn::AlifeObject;

/// The ladders a build made of the climbable faces, and the parts it could not make one of.
pub(crate) struct LevelBuildClimbableExport {
  pub(crate) entities: Vec<AlifeObject>,
  /// Each part refused, by its name, with why.
  pub(crate) refused: Vec<(String, String)>,
}
