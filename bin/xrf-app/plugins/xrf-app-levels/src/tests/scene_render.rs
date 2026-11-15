//! A Level Editor scene as the native renderer draws it: each scene object placing a library object found in the
//! objects roots stood as a model compiled once from its `.object`, what is not drawn reported by scene object, where
//! each drawn one stands for a camera to frame it, and the game data its textures are read from.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};
use tauri::{App, Manager};
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionScopes, SessionSnapshot};
use xrf_app_render::plugin::RenderPlugin;
use xrf_chunk::XRayByteOrder;
use xrf_editable_level::EditableSceneObject;
use xrf_editable_object::EditableObjectFile;
use xrf_job::ExecutionRequest;
use xrf_math::{Aabb, Vector3d};
use xrf_renderer::{
  RenderAssetSource, RenderLevelSource, RenderLevelSpawn, RenderSpawnCategory, RenderSpawnModels, RenderViewportId,
};
use xrf_vfs::{XrayMountMode, XrayRoot, XrayRoots, XraySearch, XrayTransaction};
use xrf_visual::VisualBounds;
use xrf_world::WorldLevelSource;

use crate::commands::scene::show_scene::levels_show_scene;
use crate::plugin::LevelsPlugin;
use crate::scene::{
  SceneDrawDescription, SceneDrawPart, SceneDrawProblemKind, SceneObjectId, SceneObjectSphere, SceneRenderId,
  SceneRenderSource, SceneSource, SceneToolKind, SelectedScene, SelectedScenes, ShownScenes,
};
use crate::tests::fixtures::{
  new_scenes, open_scene, scene_custom_object, scene_library_object, scene_part, write_scene,
  write_scene_library_object, write_scene_object_file, write_scene_part,
};

/// The library object two scene objects place.
const TABLE: &str = "statics\\table";

/// A library object no objects root holds.
const LOST: &str = "statics\\lost";

/// A scene object named `name` placing `reference`, standing at `x` along the scene's first axis.
fn placed(name: &str, reference: &str, x: f32) -> EditableSceneObject {
  EditableSceneObject {
    object: scene_custom_object(name, x),
    reference: String::from(reference),
    flags: 0,
  }
}

/// A unit square on the ground, as two triangles facing up.
fn square() -> Vec<[Vector3d; 3]> {
  let [a, b, c, d] = [
    Vector3d::new(0.0, 0.0, 0.0),
    Vector3d::new(1.0, 0.0, 0.0),
    Vector3d::new(1.0, 0.0, 1.0),
    Vector3d::new(0.0, 0.0, 1.0),
  ];

  vec![[a, c, b], [a, d, c]]
}

/// A scene placing the table at 0 and 10 along `x`, and between them a library object it does not find, `extra` scene
/// objects after them; the table written into its objects root.
fn write_tables(resource: &str, extra: Vec<EditableSceneObject>) -> SceneSource {
  let source: SceneSource = write_scene(resource);
  let mut objects: Vec<EditableSceneObject> = vec![
    placed("table_0000", TABLE, 0.0),
    placed("lost_0000", LOST, 5.0),
    placed("table_0001", TABLE, 10.0),
  ];

  objects.extend(extra);
  write_scene_part(&source, "scene_object.part", scene_part(objects));
  write_scene_object_file(&source, TABLE, &scene_library_object(&square()));

  source
}

/// The scene `source` names opened, and the source drawing it.
fn show(
  source: &SceneSource,
) -> (
  AssetMountState,
  Arc<SessionSnapshot<SelectedScene>>,
  Arc<SceneRenderSource>,
) {
  let assets: AssetMountState = AssetMountState::new();
  let opened: Arc<SessionSnapshot<SelectedScene>> = open_scene(&new_scenes(&assets), &assets, source);
  let drawn: Arc<SceneRenderSource> = ShownScenes::default().get_source(Arc::clone(&opened), &assets);

  (assets, opened, drawn)
}

/// The scene object of the scene object tool at `index`.
fn scene_object(index: u32) -> SceneObjectId {
  SceneObjectId {
    tool: SceneToolKind::SceneObject,
    index,
  }
}

/// The id the renderer knows the scene object at `index` by: the scene object tool's place, 2, above its index.
fn drawn_id(index: u32) -> u32 {
  SceneRenderId::of(&scene_object(index)).expect("an index an id holds").0
}

#[test]
fn lists_each_scene_object_found_by_its_place_its_reference_and_where_it_stands() {
  let (_, _, drawn) = show(&write_tables("scene_render/spawn", Vec::new()));
  let spawn: RenderLevelSpawn = drawn.read_spawn().expect("the spawn reads");

  assert_eq!(
    spawn.visuals,
    [TABLE],
    "a library object is one visual, however often placed"
  );
  assert_eq!(
    spawn
      .objects
      .iter()
      .map(|object| (object.index, object.visual, object.category, object.is_released))
      .collect::<Vec<_>>(),
    [
      (0x0200_0000, 0, RenderSpawnCategory::Props, false),
      (0x0200_0002, 0, RenderSpawnCategory::Props, false)
    ],
    "each named by its tool's place above its place among the scene's objects, the one not found left out"
  );
  // Standing at `x, 2, 3` unturned, which renderer space mirrors through `z`.
  assert_eq!(
    spawn.objects[1].transform,
    [
      1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0, 2.0, -3.0, 1.0
    ]
  );
  assert_eq!(spawn.detail, 0.0);
}

#[test]
fn compiles_a_shared_library_object_once_and_lights_every_object_standing_as_it() {
  let (_, _, drawn) = show(&write_tables("scene_render/models", Vec::new()));
  let spawn: RenderLevelSpawn = drawn.read_spawn().expect("the spawn reads");
  let models: RenderSpawnModels = drawn.read_spawn_models(&spawn.visuals).expect("the models read");

  assert!(models.failures.is_empty(), "{:?}", models.failures);
  assert_eq!(
    models
      .models
      .iter()
      .map(|model| model.name.as_str())
      .collect::<Vec<_>>(),
    [TABLE]
  );
  assert!(models.models[0].skeleton.is_none(), "a scene object is drawn as stored");
  assert!(!models.models[0].package.description.submeshes.is_empty());
  assert_eq!(
    models.models[0].surfaces.len(),
    models.models[0].package.description.submeshes.len(),
    "each submesh dressed"
  );
  assert_eq!(
    models
      .lighting
      .iter()
      .map(|lighting| (lighting.object, lighting.cube, lighting.sky))
      .collect::<Vec<_>>(),
    [(drawn_id(0), [1.0; 6], 1.0), (drawn_id(2), [1.0; 6], 1.0)],
    "both tables under the open sky"
  );
}

#[test]
fn reports_by_scene_object_what_is_missing_or_unreadable_and_leaves_out_by_reference_what_does_not_compile() {
  let source: SceneSource = write_tables(
    "scene_render/problems",
    vec![
      placed("broken_0000", "statics\\broken", 20.0),
      placed("empty_0000", "statics\\empty", 30.0),
    ],
  );
  let mut empty: EditableObjectFile = scene_library_object(&square());

  empty.meshes = Some(Vec::new());
  write_scene_library_object(&source, "statics\\broken");
  write_scene_object_file(&source, "statics\\empty", &empty);

  let (assets, opened, drawn) = show(&source);
  let spawn: RenderLevelSpawn = drawn.read_spawn().expect("the spawn reads");
  let models: RenderSpawnModels = drawn.read_spawn_models(&spawn.visuals).expect("the models read");
  let described: Arc<SceneDrawDescription> =
    Arc::clone(&SceneDrawPart::get(&opened, &assets.search(&opened.objects_roots).unwrap()).description);

  assert_eq!(
    spawn.objects.iter().map(|object| object.index).collect::<Vec<_>>(),
    [0, 2, 3, 4].map(drawn_id),
    "an object found is listed before it is read"
  );
  assert_eq!(
    models
      .models
      .iter()
      .map(|model| model.name.as_str())
      .collect::<Vec<_>>(),
    [TABLE]
  );
  assert_eq!(
    models
      .failures
      .iter()
      .map(|failure| failure.name.as_str())
      .collect::<Vec<_>>(),
    ["statics\\broken", "statics\\empty"],
    "each left out named by its reference, as the viewport's problems list it"
  );
  assert!(
    models.failures[1].reason.contains("does not compile"),
    "{}",
    models.failures[1].reason
  );
  assert_eq!(
    models
      .lighting
      .iter()
      .map(|lighting| lighting.object)
      .collect::<Vec<_>>(),
    [0, 2].map(drawn_id),
    "only what is drawn is lit"
  );
  assert_eq!(
    described
      .problems
      .iter()
      .map(|problem| (problem.object.clone(), problem.reference.as_str(), problem.kind))
      .collect::<Vec<_>>(),
    [
      (scene_object(1), LOST, SceneDrawProblemKind::Missing),
      (scene_object(3), "statics\\broken", SceneDrawProblemKind::Unreadable),
    ],
    "what is known without compiling"
  );
}

#[test]
fn bounds_the_objects_drawn_by_their_placed_boxes_in_renderer_space() {
  let source: SceneSource = write_tables("scene_render/bounds", Vec::new());
  let (assets, opened, _) = show(&source);
  let described: Arc<SceneDrawDescription> =
    Arc::clone(&SceneDrawPart::get(&opened, &assets.search(&opened.objects_roots).unwrap()).description);
  let bounds: &VisualBounds = described.bounds.as_ref().expect("the tables are drawn");

  // The unit square at `0, 2, 3` and `10, 2, 3`, `z` mirrored.
  assert_eq!(
    bounds.bounding_box,
    Aabb {
      min: Vector3d::new(0.0, 2.0, -4.0),
      max: Vector3d::new(11.0, 2.0, -3.0),
    }
  );
}

#[test]
fn locates_a_drawn_object_by_its_placed_box_in_renderer_space_and_nothing_for_one_not_drawn() {
  let source: SceneSource = write_tables("scene_render/locate", Vec::new());
  let (assets, opened, _) = show(&source);
  let locate = |index: u32| SceneObjectSphere::locate(&opened, &assets, &scene_object(index)).expect("it searches");

  // The unit square at `10, 2, 3`, `z` mirrored.
  assert_eq!(
    locate(2),
    Some(
      Aabb {
        min: Vector3d::new(10.0, 2.0, -4.0),
        max: Vector3d::new(11.0, 2.0, -3.0),
      }
      .get_bounding_sphere()
    )
  );
  assert_eq!(locate(1), None, "its library object is not found");
  assert_eq!(locate(3), None, "no such scene object");
  assert!(
    SceneDrawPart::get(&opened, &assets.search(&opened.objects_roots).unwrap())
      .find_sphere(0)
      .is_some(),
    "kept with the drawing, read once"
  );
}

#[test]
fn a_library_object_come_into_the_objects_roots_is_read_again_and_bounded() {
  let source: SceneSource = write_tables("scene_render/follow", Vec::new());
  let assets: AssetMountState = AssetMountState::new();
  let scenes: SelectedScenes = new_scenes(&assets);
  let opened: Arc<SessionSnapshot<SelectedScene>> = open_scene(&scenes, &assets, &source);
  let objects: XraySearch = assets.search(&opened.objects_roots).unwrap();
  let before: Arc<SceneDrawDescription> = Arc::clone(&SceneDrawPart::get(&opened, &objects).description);

  assets
    .commit(
      &XrayTransaction::new(&objects)
        .with_write(
          &format!("{LOST}.object"),
          scene_library_object(&square())
            .write_to_bytes::<XRayByteOrder>()
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

  let next: Arc<SessionSnapshot<SelectedScene>> = scenes.require(opened.session_id).unwrap();
  let after: Arc<SceneDrawDescription> =
    Arc::clone(&SceneDrawPart::get(&next, &assets.search(&next.objects_roots).unwrap()).description);

  assert_eq!(before.problems.len(), 1);
  assert!(Arc::ptr_eq(&next.value, &opened.value), "the scene itself was kept");
  assert!(after.problems.is_empty(), "read again, the library object is found");
  assert_eq!(
    after.bounds.as_ref().map(|bounds| bounds.bounding_box),
    Some(Aabb {
      min: Vector3d::new(0.0, 2.0, -4.0),
      max: Vector3d::new(11.0, 2.0, -3.0),
    }),
    "the object found stands within what was bounded"
  );
}

#[test]
fn a_scene_draws_nothing_where_its_scene_object_tool_holds_no_object() {
  let (assets, opened, drawn) = show(&write_scene("scene_render/empty"));
  let described: Arc<SceneDrawDescription> =
    Arc::clone(&SceneDrawPart::get(&opened, &assets.search(&opened.objects_roots).unwrap()).description);

  assert!(drawn.read_spawn().expect("the spawn reads").objects.is_empty());
  assert!(described.bounds.is_none());
  assert!(described.problems.is_empty());
}

#[test]
fn viewports_of_one_scene_session_share_its_scene_and_its_source() {
  let source: SceneSource = write_tables("scene_render/key", Vec::new());
  let assets: AssetMountState = AssetMountState::new();
  let opened: Arc<SessionSnapshot<SelectedScene>> = open_scene(&new_scenes(&assets), &assets, &source);
  let shown: ShownScenes = ShownScenes::default();
  let first: Arc<SceneRenderSource> = shown.get_source(Arc::clone(&opened), &assets);
  let second: Arc<SceneRenderSource> = shown.get_source(Arc::clone(&opened), &assets);

  assert!(Arc::ptr_eq(&first, &second), "one source a snapshot");
  assert_eq!(first.get_scene_key(), Some(format!("scene {}", opened.session_id)));
  assert_eq!(first.get_sector_count(), 0, "a scene has no sectors");
  assert!(
    shown
      .find_source(opened.session_id)
      .is_some_and(|found| Arc::ptr_eq(&found, &first))
  );
}

#[test]
fn reads_textures_from_the_game_data_beside_the_rawdata_and_none_without_it() {
  let source: SceneSource = write_tables("scene_render/textures", Vec::new());
  let (_, opened, drawn) = show(&source);

  assert_eq!(
    opened.game.roots,
    XrayRoots::default(),
    "no game data beside the rawdata"
  );
  assert_eq!(
    drawn
      .read_texture("wood\\wood_planks")
      .expect("a missing texture is no error"),
    None,
    "drawn with the renderer's placeholder"
  );

  let game_data: PathBuf = Path::new(&source.path)
    .ancestors()
    .nth(3)
    .expect("the scene lies in a rawdata tree")
    .join(SceneSource::GAME_DATA_DIRECTORY);
  let texture: PathBuf = game_data.join("textures").join("wood").join("wood_planks.dds");

  fs::create_dir_all(texture.parent().unwrap()).unwrap();
  fs::write(&texture, b"planks").unwrap();

  let (_, opened, drawn) = show(&source);

  assert_eq!(
    opened.game.roots,
    XrayRoots::new([XrayRoot::new(game_data, XrayMountMode::Directory)])
  );
  assert_eq!(
    drawn.read_texture("wood\\wood_planks").expect("the texture reads"),
    Some(b"planks".to_vec())
  );
}

#[test]
fn the_game_roots_given_win_over_the_game_data_beside_the_rawdata() {
  let source: SceneSource = SceneSource {
    path: String::from("C:\\sdk\\rawdata\\levels\\labx8.level"),
  };
  let given: XrayRoots = XrayRoots::new([XrayRoot::new("D:\\game".into(), XrayMountMode::Installation)]);

  assert_eq!(source.get_game_roots(Some(given.clone())), given);
  assert_eq!(
    source.get_game_roots(None),
    XrayRoots::default(),
    "no game data where none lies beside the rawdata"
  );
}

#[test]
fn refuses_to_show_a_session_that_holds_no_scene() {
  let application: App<MockRuntime> = mock_builder()
    .manage(SessionScopes::default())
    .manage(AssetMountState::new())
    .manage(ExecutionState::new(ExecutionRequest::Auto).expect("the pool starts"))
    .plugin(RenderPlugin::init())
    .plugin(LevelsPlugin::init())
    .build(mock_context(noop_assets()))
    .expect("the plugins set up");
  let session_id: SessionId = SessionId::generate();

  let shown = tauri::async_runtime::block_on(levels_show_scene(
    application.state(),
    application.state(),
    application.state(),
    application.state(),
    application.state(),
    application.state(),
    RenderViewportId(1),
    Some(session_id),
  ));

  assert!(shown.is_err(), "no scene is held under the session");
  assert!(
    application.state::<ShownScenes>().find_source(session_id).is_none(),
    "nothing is drawn from it"
  );
}
