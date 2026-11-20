/// One mesh of a library object's placement and the compiled sector naming it, since the Build sectors each mesh's
/// faces apart (`SceneBuilder::BuildObject`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelInstanceMesh {
  pub name: String,
  /// The compiled sector holding most of the mesh's taken triangles, or of the placement's where it took none, as the
  /// editor puts a scene object's meshes in sectors one by one (`CSector::AddMesh`).
  pub sector: u16,
}
