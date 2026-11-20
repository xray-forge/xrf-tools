use crate::level::geometry::level_decompiled_object::LevelDecompiledObject;

/// A ladder decompiled: its object, and whether its game material was inferred.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelDecompiledLadder {
  pub object: LevelDecompiledObject,
  pub is_material_inferred: bool,
}
