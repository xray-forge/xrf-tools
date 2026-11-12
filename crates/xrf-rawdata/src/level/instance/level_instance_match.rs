use crate::level::instance::level_instance_fit::LevelInstanceFit;

/// A library object found placed in a level's static geometry.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LevelInstanceMatch {
  /// The object, by its place in the library.
  pub(crate) object: usize,
  pub(crate) fit: LevelInstanceFit,
  /// How much of the object's rendered area the level draws as the object's own triangles, from 0 to 1.
  pub(crate) coverage: f64,
  /// The root mean square distance, metres, between the object's placed corners and the level's they were matched to.
  pub(crate) error: f64,
  /// The level triangles the placement covers, which leave the shells, by their place in the index.
  pub(crate) claimed: Vec<u32>,
}
