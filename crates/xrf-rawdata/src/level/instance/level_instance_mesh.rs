/// One mesh of a library object's placement and the compiled sector that names it: a sector names each mesh of a scene
/// object apart, and the Build gives each mesh's faces its sector (`SceneBuilder::BuildObject`, `BuilderRemote.cpp`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelInstanceMesh {
  pub name: String,
  /// The compiled sector holding most of the mesh's triangles the placement took; where it took none, the sector
  /// holding most of the placement's.
  pub sector: u16,
}
