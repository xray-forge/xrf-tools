/// One mesh a sector holds, `CSectorItem`: a scene object by name, and one of its object's meshes by name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableSectorItem {
  pub object: String,
  pub mesh: String,
}
