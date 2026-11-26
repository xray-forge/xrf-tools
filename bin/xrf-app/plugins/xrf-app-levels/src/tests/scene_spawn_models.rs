use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::SessionSnapshot;
use xrf_chunk::{ChunkFieldRecorder, ChunkReadWrite, ChunkWriter};
use xrf_editable_level::{EditableShape, EditableShapeObject, EditableSpawnPointKind, EditableSpawnPointObject};
use xrf_error::XrfResult;
use xrf_ini::IniSection;
use xrf_level_source::fixtures::{new_abstract, new_object};
use xrf_math::{Aabb, Sphere, Vector3d};
use xrf_ogf::{OgfGeometry, OgfHeaderChunk, OgfTextureChunk, OgfVertex, OgfVertexLink, OgfVertexLinks};
use xrf_rawdata::{LevelSpawnDataFields, ObjectCompileOptions, ObjectCompileSource, ObjectCompiler};
use xrf_renderer::{
  RenderHelperIcon, RenderHelperObject, RenderHelperShape, RenderLevelHelpers, RenderLevelSpawn, RenderSpawnCategory,
  RenderSpawnModels, RenderSpawnObject,
};
use xrf_spawn::{AlifeObject, AlifeObjectBreakable, AlifeObjectDynamicVisual, AlifeObjectInherited, ClsId};
use xrf_world::WorldLevelSource;

use crate::scene::{
  SceneHelperToolCount, SceneHelpers, SceneHelpersPart, SceneObjectId, SceneObjectSphere, SceneRenderId,
  SceneRenderSource, SceneSource, SceneSpawnModelDescription, SceneToolKind, SelectedScene, ShownScenes,
};
use crate::tests::fixtures::{
  new_scenes, open_scene, scene_custom_object, scene_library_object, scene_part, write_scene, write_scene_part,
};

const BOX: &str = "dynamics\\box\\box_wood";

// The game data does not hold it.
const GONE: &str = "dynamics\\box\\box_gone";

const BOX_MODEL: &str = "dynamics\\box\\box_wood.ogf";

fn packet(visual: &str) -> IniSection {
  let mut entity: AlifeObject = new_object(
    "box",
    "breakable_object",
    ClsId::OBrkbl,
    AlifeObjectInherited::CseAlifeObjectBreakable(Box::new(AlifeObjectBreakable {
      base: AlifeObjectDynamicVisual {
        base: new_abstract(u32::MAX),
        visual_name: String::from(visual),
        visual_flags: 0,
      },
      health: 1.0,
    })),
  );
  let mut recorder: ChunkFieldRecorder = ChunkFieldRecorder::new();
  let mut data: IniSection = IniSection::new("spawndata");

  // The flags every editor-saved packet carries, including `M_SPAWN_VERSION`.
  entity.header.script_flags = 33;
  entity.write_spawn_packet(&mut recorder).expect("the packet writes");
  LevelSpawnDataFields::write_section(&recorder.into_fields(), &mut data).expect("the packet saves");

  data
}

fn spawn_point(name: &str, x: f32, data: IniSection, is_volumed: bool) -> EditableSpawnPointObject {
  EditableSpawnPointObject {
    object: scene_custom_object(name, x),
    kind: EditableSpawnPointKind::Entity {
      section: String::from("breakable_object"),
      flags: 0,
      data,
    },
    attached: is_volumed.then(|| EditableShapeObject {
      object: scene_custom_object("shape_0000", x),
      shape: EditableShape::new_unit_box(),
    }),
  }
}

fn game_data(source: &SceneSource) -> PathBuf {
  Path::new(&source.path)
    .ancestors()
    .nth(3)
    .expect("the scene lies in a rawdata tree")
    .join(SceneSource::GAME_DATA_DIRECTORY)
}

fn write_model(source: &SceneSource, visual: &str, bytes: &[u8]) {
  let path: PathBuf = game_data(source)
    .join("meshes")
    .join(format!("{}.ogf", visual.replace('\\', "/")));

  fs::create_dir_all(path.parent().expect("a model lies in a directory")).expect("the meshes are made");
  fs::write(path, bytes).expect("the model is written");
}

/// The unit square as a skinned visual, one piece of one link a vertex, as a dynamic object's export lays it out.
fn skinned_square_model() -> Vec<u8> {
  let corner = |x: f32, z: f32| OgfVertex {
    position: Vector3d::new(x, 0.0, z),
    normal: Vector3d::new(0.0, 1.0, 0.0),
    tangent: Vector3d::new(1.0, 0.0, 0.0),
    binormal: Vector3d::new(0.0, 0.0, 1.0),
    texture_u: x,
    texture_v: z,
    links: OgfVertexLinks::from([OgfVertexLink { bone: 0, weight: 1.0 }]),
  };
  let bounding_box: Aabb = Aabb {
    min: Vector3d::new(0.0, 0.0, 0.0),
    max: Vector3d::new(1.0, 0.0, 1.0),
  };
  let mut file: ChunkWriter = ChunkWriter::new();
  let mut write_chunk = |id: u32, chunk: &dyn Fn(&mut ChunkWriter) -> XrfResult| {
    let mut writer: ChunkWriter = ChunkWriter::new();

    chunk(&mut writer).expect("the chunk writes");
    writer.flush_chunk_into(&mut file, id).expect("the chunk is framed");
  };

  write_chunk(OgfHeaderChunk::CHUNK_ID, &|writer| {
    OgfHeaderChunk {
      version: 4,
      // `MT_SKELETON_GEOMDEF_ST`, a skinned piece standing alone.
      model_type: 5,
      shader_id: 0,
      bounding_sphere: bounding_box.get_bounding_sphere(),
      bounding_box,
    }
    .write(writer)
  });
  write_chunk(OgfTextureChunk::CHUNK_ID, &|writer| {
    OgfTextureChunk {
      texture_name: String::from("wood\\wood_planks"),
      shader_name: String::from("models\\model"),
      trailing: Vec::new(),
    }
    .write(writer)
  });
  OgfGeometry::write_skinned(
    &mut file,
    &[corner(0.0, 0.0), corner(1.0, 0.0), corner(1.0, 1.0), corner(0.0, 1.0)],
    &[0, 2, 1, 0, 3, 2],
  )
  .expect("the geometry writes");

  file.into_bytes()
}

/// The unit square compiled as a rigid object is, of static vertices, which pack to no geometry.
fn square_model() -> Vec<u8> {
  let [a, b, c, d] = [
    Vector3d::new(0.0, 0.0, 0.0),
    Vector3d::new(1.0, 0.0, 0.0),
    Vector3d::new(1.0, 0.0, 1.0),
    Vector3d::new(0.0, 0.0, 1.0),
  ];

  ObjectCompiler::compile_with(
    &scene_library_object(&[[a, c, b], [a, d, c]]),
    &ObjectCompileSource {
      source_file: String::from("box_wood.object"),
      convertor: String::from("xrf-app-levels tests"),
      built_at: 0,
    },
    &ObjectCompileOptions::default(),
  )
  .expect("the square compiles")
  .bytes
}

// The last spawn point's empty packet does not read.
fn write_spawn_scene(resource: &str, model: Option<&[u8]>) -> SceneSource {
  let source: SceneSource = write_scene(resource);

  write_scene_part(
    &source,
    "spawn.part",
    scene_part(vec![
      spawn_point("box_0000", 0.0, packet(BOX), false),
      spawn_point("box_0001", 10.0, packet(BOX), true),
      spawn_point("gone_0000", 20.0, packet(GONE), false),
      spawn_point("stalker_0000", 30.0, IniSection::new("spawndata"), false),
    ]),
  );

  if let Some(model) = model {
    write_model(&source, BOX, model);
  }

  source
}

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

fn spawn_point_id(index: u32) -> SceneObjectId {
  SceneObjectId {
    tool: SceneToolKind::Spawn,
    index,
  }
}

fn drawn_id(index: u32) -> u32 {
  SceneRenderId::of(&spawn_point_id(index))
    .expect("an index an id holds")
    .0
}

fn list_spawn_helpers(helpers: &RenderLevelHelpers) -> Vec<(u32, &RenderHelperObject)> {
  helpers
    .objects
    .iter()
    .filter_map(|object| {
      SceneRenderId(object.id)
        .get_object()
        .filter(|id| id.tool == SceneToolKind::Spawn)
        .map(|id| (id.index, object))
    })
    .collect()
}

#[test]
fn stands_each_spawn_point_whose_visual_is_found_as_its_model_in_a_group_of_its_own() {
  let (_, _, drawn) = show(&write_spawn_scene(
    "scene_spawn_models/spawn",
    Some(&skinned_square_model()),
  ));
  let spawn: RenderLevelSpawn = drawn.read_spawn().expect("the spawn reads");

  assert_eq!(spawn.visuals, [BOX_MODEL], "a visual is one model, named by its file");
  assert_eq!(
    spawn
      .objects
      .iter()
      .map(|object| (object.index, object.visual, object.group))
      .collect::<Vec<_>>(),
    [
      (drawn_id(0), 0, RenderSpawnCategory::Items.into()),
      (drawn_id(1), 0, RenderSpawnCategory::Items.into()),
    ],
    "named by the spawn tool's place above their index, the visual not found and the unread packet left out"
  );
  // At `10, 2, 3` unturned, `z` mirrored into renderer space.
  assert_eq!(
    spawn.objects[1].transform,
    [
      1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0, 2.0, -3.0, 1.0
    ]
  );
}

#[test]
fn a_spawn_point_drawn_as_its_model_keeps_its_volume_and_loses_its_icon() {
  let (assets, opened, _) = show(&write_spawn_scene(
    "scene_spawn_models/helpers",
    Some(&skinned_square_model()),
  ));
  let helpers: Arc<SceneHelpers> = SceneHelpersPart::get(&opened, &assets.search(&opened.game.roots).unwrap());
  let spawns: Vec<(u32, &RenderHelperObject)> = list_spawn_helpers(&helpers.helpers);

  assert_eq!(
    spawns.iter().map(|(index, _)| *index).collect::<Vec<_>>(),
    [1, 2, 3],
    "the first box draws its model alone"
  );
  assert!(
    matches!(spawns[0].1.shapes.as_slice(), [RenderHelperShape::Box { .. }]),
    "the second box keeps its volume, and no icon"
  );

  for (_, helper) in &spawns[1..] {
    assert!(
      matches!(
        helper.shapes.as_slice(),
        [RenderHelperShape::Icon {
          icon: RenderHelperIcon::SpawnPoint,
          ..
        }]
      ),
      "a visual not found, and a packet that does not read, keep the icon"
    );
  }

  assert_eq!(
    helpers.description.tools,
    [SceneHelperToolCount {
      tool: SceneToolKind::Spawn,
      count: 4
    }],
    "every spawn point counted"
  );
  assert_eq!(
    helpers.description.spawn_models,
    [0, 1].map(|index| SceneSpawnModelDescription {
      object: spawn_point_id(index),
      model: String::from(BOX_MODEL),
    })
  );
  assert!(
    helpers
      .description
      .bounds
      .as_ref()
      .is_some_and(|bounds| bounds.bounding_box.min.x <= 0.0),
    "the box standing as its model alone is within the extent framed"
  );
}

#[test]
fn every_spawn_point_keeps_its_icon_where_the_scene_has_no_game_data() {
  let (assets, opened, drawn) = show(&write_spawn_scene("scene_spawn_models/no_game_data", None));
  let helpers: Arc<SceneHelpers> = SceneHelpersPart::get(&opened, &assets.search(&opened.game.roots).unwrap());

  assert!(opened.game.roots.is_empty());
  assert!(drawn.read_spawn().expect("the spawn reads").objects.is_empty());
  assert_eq!(
    list_spawn_helpers(&helpers.helpers)
      .iter()
      .map(|(index, _)| *index)
      .collect::<Vec<_>>(),
    [0, 1, 2, 3]
  );
  assert!(helpers.description.spawn_models.is_empty());
}

#[test]
fn reads_a_spawn_points_model_once_and_lights_each_standing_as_it_by_the_open_sky() {
  let (_, _, drawn) = show(&write_spawn_scene(
    "scene_spawn_models/models",
    Some(&skinned_square_model()),
  ));
  let spawn: RenderLevelSpawn = drawn.read_spawn().expect("the spawn reads");
  let models: RenderSpawnModels = drawn.read_spawn_models(&spawn.visuals).expect("the models read");

  assert!(models.failures.is_empty(), "{:?}", models.failures);
  assert_eq!(
    models
      .models
      .iter()
      .map(|model| model.name.as_str())
      .collect::<Vec<_>>(),
    [BOX_MODEL]
  );
  assert_eq!(
    models
      .lighting
      .iter()
      .map(|lighting| (lighting.object, lighting.cube, lighting.sky))
      .collect::<Vec<_>>(),
    [(drawn_id(0), [1.0; 6], 1.0), (drawn_id(1), [1.0; 6], 1.0)]
  );
}

#[test]
fn a_model_found_that_does_not_read_is_a_load_failure_named_by_its_file() {
  let (assets, opened, drawn) = show(&write_spawn_scene(
    "scene_spawn_models/unreadable",
    Some(b"not a model"),
  ));
  let spawn: RenderLevelSpawn = drawn.read_spawn().expect("the spawn reads");
  let models: RenderSpawnModels = drawn.read_spawn_models(&spawn.visuals).expect("the models read");

  assert_eq!(
    spawn
      .objects
      .iter()
      .map(|object: &RenderSpawnObject| object.index)
      .collect::<Vec<_>>(),
    [drawn_id(0), drawn_id(1)],
    "found, so stood as models before they are read"
  );
  assert!(models.models.is_empty());
  assert!(models.lighting.is_empty(), "nothing drawn is lit");
  assert_eq!(
    models
      .failures
      .iter()
      .map(|failure| failure.name.as_str())
      .collect::<Vec<_>>(),
    [BOX_MODEL]
  );
  assert!(
    models.failures[0].reason.contains("Failed to read visual"),
    "{}",
    models.failures[0].reason
  );
  // At `0, 2, 3`, `z` mirrored.
  assert_eq!(
    SceneObjectSphere::locate(&opened, &assets, &spawn_point_id(0)).expect("the scene is searched"),
    Some(Sphere {
      center: Vector3d::new(0.0, 2.0, -3.0),
      radius: 0.0,
    }),
    "a model that does not read leaves the spawn point its icon, which it locates by"
  );
}

// A rigid object's visual is read and found, but its pieces pack to nothing: standing as it, the spawn point would
// show nothing at all.
#[test]
fn a_model_of_static_vertices_is_a_load_failure_and_leaves_the_spawn_point_its_icon() {
  let (assets, opened, drawn) = show(&write_spawn_scene("scene_spawn_models/static", Some(&square_model())));
  let spawn: RenderLevelSpawn = drawn.read_spawn().expect("the spawn reads");
  let models: RenderSpawnModels = drawn.read_spawn_models(&spawn.visuals).expect("the models read");
  let helpers: Arc<SceneHelpers> = SceneHelpersPart::get(&opened, &assets.search(&opened.game.roots).unwrap());

  assert_eq!(
    spawn
      .objects
      .iter()
      .map(|object: &RenderSpawnObject| object.index)
      .collect::<Vec<_>>(),
    [drawn_id(0), drawn_id(1)],
    "stood, so its load reports why it draws nothing"
  );
  assert!(models.models.is_empty());
  assert_eq!(
    models
      .failures
      .iter()
      .map(|failure| failure.name.as_str())
      .collect::<Vec<_>>(),
    [BOX_MODEL]
  );
  assert!(
    models.failures[0].reason.contains("packs no geometry"),
    "{}",
    models.failures[0].reason
  );
  assert_eq!(
    list_spawn_helpers(&helpers.helpers)
      .iter()
      .map(|(index, _)| *index)
      .collect::<Vec<_>>(),
    [0, 1, 2, 3],
    "every spawn point keeps its icon"
  );
  assert_eq!(
    helpers.description.spawn_models.len(),
    2,
    "listed as standing as the model, so the failure names them"
  );
}

#[test]
fn locates_a_spawn_point_drawn_as_its_model_by_its_models_placed_bounds() {
  let (assets, opened, _) = show(&write_spawn_scene(
    "scene_spawn_models/locate",
    Some(&skinned_square_model()),
  ));
  let locate =
    |index: u32| SceneObjectSphere::locate(&opened, &assets, &spawn_point_id(index)).expect("the scene is searched");

  // The unit square at `10, 2, 3`, `z` mirrored.
  assert_eq!(
    locate(1),
    Some(
      Aabb {
        min: Vector3d::new(10.0, 2.0, -4.0),
        max: Vector3d::new(11.0, 2.0, -3.0),
      }
      .get_bounding_sphere()
    )
  );
  // An icon at `20, 2, 3`: a point.
  assert_eq!(
    locate(2),
    Some(Sphere {
      center: Vector3d::new(20.0, 2.0, -3.0),
      radius: 0.0,
    })
  );
}
