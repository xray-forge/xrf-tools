//! Where the renderer stands a scene object's model, against where the level build places the same object's points.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use glam::{Mat4, Vec3};
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::SessionSnapshot;
use xrf_chunk::XRayByteOrder;
use xrf_editable_level::{EditableCustomObject, EditableObjectTransform, EditableSceneObject};
use xrf_editable_object::EditableObjectFile;
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_math::Vector3d;
use xrf_rawdata::{LevelBuildExport, LevelBuildExporter, LevelBuildImpostorLimit, LevelBuildSources};
use xrf_renderer::RenderLevelSpawn;
use xrf_visual::convert_vector;
use xrf_world::WorldLevelSource;

use crate::scene::{SceneRenderSource, SceneSource, SelectedScene, ShownScenes};
use crate::tests::fixtures::{
  SCENE_GAME_MATERIAL, new_scenes, open_scene, scene_library_object, scene_part, write_scene, write_scene_object_file,
  write_scene_part,
};

/// How far a drawn point may lie from where the build placed it, the two composed in different orders in `f32`.
const TOLERANCE: f32 = 1e-4;

#[test]
fn a_scene_object_stands_where_the_level_build_places_its_points() {
  let source: SceneSource = write_scene("scene_render/placement");
  // Turned about every axis and scaled unevenly, so a basis taken by rows rather than columns, an angle's sign or a
  // mirror applied on one side only would each show.
  let transform: EditableObjectTransform = EditableObjectTransform {
    position: Vector3d::new(12.5, -3.0, 7.25),
    rotation: Vector3d::new(0.4, -1.1, 0.7),
    scale: Vector3d::new(2.0, 0.5, 1.5),
  };
  let object: EditableObjectFile = scene_library_object(&[
    [
      Vector3d::new(0.0, 0.0, 0.0),
      Vector3d::new(1.0, 0.2, 3.0),
      Vector3d::new(2.0, 0.0, 0.5),
    ],
    [
      Vector3d::new(-1.0, 1.5, 0.25),
      Vector3d::new(0.5, 2.0, -2.0),
      Vector3d::new(0.0, 1.0, 1.0),
    ],
  ]);

  write_scene_part(
    &source,
    "scene_object.part",
    scene_part(vec![EditableSceneObject {
      object: EditableCustomObject::new("shell_0000", transform),
      reference: String::from("levels\\placement\\shell"),
      flags: 0,
    }]),
  );
  write_scene_object_file(&source, "levels\\placement\\shell", &object);

  let assets: AssetMountState = AssetMountState::new();
  let opened: Arc<SessionSnapshot<SelectedScene>> = open_scene(&new_scenes(&assets), &assets, &source);
  let objects: PathBuf = Path::new(&source.path)
    .ancestors()
    .nth(2)
    .expect("the scene lies in a rawdata tree")
    .join(SceneSource::OBJECTS_DIRECTORY);
  let built: LevelBuildExport = LevelBuildExporter::export::<XRayByteOrder>(
    &opened.scene,
    &LevelBuildSources {
      objects: &[objects],
      game_materials: &GameMtlFile::new(vec![GameMtlMaterial::new(1, SCENE_GAME_MATERIAL)], Vec::new()),
      textures: None,
      impostor_limit: LevelBuildImpostorLimit::default(),
      held: None,
    },
  )
  .expect("the scene builds");
  let drawn: Arc<SceneRenderSource> = ShownScenes::default().get_source(Arc::clone(&opened), &assets);
  let spawn: RenderLevelSpawn = drawn.read_spawn().expect("the spawn reads");
  let placement: Mat4 = Mat4::from_cols_array(&spawn.objects[0].transform);
  let points: &[Vector3d] = &object.meshes.as_ref().expect("the object has a mesh")[0].points;

  // The build places every point of the object's one mesh, in order, in engine space.
  assert_eq!(built.project.vertices.len(), points.len());

  for (point, placed) in points.iter().zip(&built.project.vertices) {
    // A model's vertex is the point in renderer space, which the drawn transform stands where it lands.
    let model: Vector3d = convert_vector(point);
    let drawn: Vec3 = placement.transform_point3(Vec3::new(model.x, model.y, model.z));
    let expected: Vector3d = convert_vector(placed);

    assert!(
      Vector3d::new(drawn.x, drawn.y, drawn.z).distance(&expected) <= TOLERANCE,
      "point {point:?} is drawn at {drawn:?}, built at {placed:?}"
    );
  }
}
