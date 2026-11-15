//! What several test modules build the same way: a level of shader entries, a search over a fixture tree, an empty
//! environment catalog, a spawned object, and a Level Editor scene written into a rawdata tree with its library
//! objects, and opened.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionDerived, SessionId, SessionScopes, SessionSnapshot};
use xrf_chunk::XRayByteOrder;
use xrf_editable_level::{
  EditableBuildParams, EditableCustomObject, EditableLevelCamera, EditableLevelFile, EditableLevelGuid,
  EditableLevelOptions, EditableObjectPart, EditableObjectTransform, EditableToolObject,
};
use xrf_editable_object::EditableObjectFile;
use xrf_engine_target::{XrayEngine, XrayEngineChoice};
use xrf_environment::{EnvironmentCatalog, WeatherGraphs};
use xrf_level::{LevelFile, LevelHeaderChunk, LevelShaderEntry, LevelShadersChunk, LevelVertex};
use xrf_material::fixtures::FixtureTree;
use xrf_math::{Vector2d, Vector3d};
use xrf_rawdata::{LevelVisualGeometry, ObjectDecompiler, SurfaceDress};
use xrf_spawn::{AlifeObject, AlifeObjectAbstract, AlifeObjectInherited, ClsId, SpawnPacketHeader};
use xrf_test_utils::GeneratedResources;
use xrf_vfs::{XrayLookupScope, XrayMountId, XrayRevision, XrayRoots, XraySearch, XrayVfs};

use crate::scene::{OpenedScene, SceneGameData, SceneSource, SelectedScene, SelectedScenes};

/// The GUID every test scene and its parts carry, `labx8`'s.
pub(super) const SCENE_GUID: EditableLevelGuid = EditableLevelGuid {
  g0: 5065284203709945560,
  g1: 13177767572218497455,
};

/// The name of every test scene, which names its part directory.
pub(super) const SCENE_NAME: &str = "labx8";

/// A level whose shader table holds exactly these raw entries.
pub(super) fn new_level(entries: &[&str]) -> LevelFile {
  LevelFile {
    header: LevelHeaderChunk {
      xrlc_quality: 1,
      xrlc_version: 14,
    },
    glows: None,
    lights: None,
    portals: None,
    sectors: None,
    shaders: Some(LevelShadersChunk {
      entries: entries.iter().map(|raw| LevelShaderEntry::parse(raw)).collect(),
    }),
  }
}

/// A search over `tree` alone, mounted as the game's root.
pub(super) fn mount_search(tree: &FixtureTree) -> XraySearch {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");

  XraySearch::new(vfs).with_step("tree", XrayLookupScope::only([id]))
}

/// A catalog of nothing, the vanilla engine's, for a test to fill what it reads.
pub(super) fn new_catalog() -> EnvironmentCatalog {
  EnvironmentCatalog {
    ambient_effects: Vec::new(),
    ambients: Vec::new(),
    configs: Vec::new(),
    cycles: Vec::new(),
    effects: Vec::new(),
    engine: XrayEngine::Vanilla,
    findings: Vec::new(),
    graphs: WeatherGraphs::default(),
    level_ambients: Vec::new(),
    sound_channels: Vec::new(),
    sun_table: None,
    suns: Vec::new(),
    thunderbolt_collections: Vec::new(),
    thunderbolt_settings: None,
    thunderbolts: Vec::new(),
  }
}

/// An object's abstract part with `story_id`, on no vertex.
pub(super) fn new_abstract(story_id: u32) -> AlifeObjectAbstract {
  AlifeObjectAbstract {
    game_vertex_id: 0,
    distance: 0.0,
    direct_control: 1,
    level_vertex_id: 0,
    flags: 0,
    custom_data: String::new(),
    story_id,
    spawn_story_id: AlifeObjectAbstract::INVALID_STORY_ID,
  }
}

/// A spawned object named `name` of `section`, at `1, 2, 3` facing nowhere in particular.
pub(super) fn new_object(name: &str, section: &str, clsid: ClsId, inherited: AlifeObjectInherited) -> AlifeObject {
  AlifeObject {
    header: SpawnPacketHeader {
      id: 0,
      net_action: 1,
      section: String::from(section),
      name: String::from(name),
      script_game_id: 0,
      script_rp: 0,
      position: Vector3d::new(1.0, 2.0, 3.0),
      direction: Vector3d::new(0.0, 0.0, 0.0),
      respawn_time: 0,
      parent_id: u16::MAX,
      phantom_id: u16::MAX,
      script_flags: 0,
      version: 128,
      game_type: 0,
      script_version: 12,
      client_data: Vec::new(),
      spawn_id: 0,
    },
    clsid,
    inherited,
  }
}

/// A rawdata tree made afresh under `resource`, holding the scene `labx8` in `levels` with its `.level` alone, as the
/// CoP SDK's editor saves a new one; answers where it is read from.
pub(super) fn write_scene(resource: &str) -> SceneSource {
  let levels: PathBuf = GeneratedResources::create_dir(resource)
    .expect("the tree is made")
    .join("rawdata")
    .join("levels");
  let level: EditableLevelFile = EditableLevelFile::new(
    &SCENE_GUID,
    &EditableLevelOptions::new(SCENE_NAME, EditableLevelOptions::SINGLE_PLAYER),
    &EditableBuildParams::DEFAULT,
    &EditableLevelCamera::default(),
    "owner",
    0,
  );
  let path: PathBuf = levels.join(format!("{SCENE_NAME}.{}", EditableLevelFile::EXTENSION));

  fs::create_dir_all(levels.join(SCENE_NAME)).expect("the part directory is made");
  fs::write(&path, level.write_to_bytes().expect("the .level writes")).expect("the .level is written");

  SceneSource {
    path: path.to_string_lossy().into_owned(),
  }
}

/// Writes `bytes` as the scene's part file `file`, beside its `.level`.
pub(super) fn write_scene_part(source: &SceneSource, file: &str, bytes: impl AsRef<[u8]>) {
  let parts: PathBuf = Path::new(&source.path).with_extension("");

  fs::write(parts.join(file), bytes).expect("the part is written");
}

/// Writes a library object at `reference` into the `objects` directory of the scene's rawdata tree.
pub(super) fn write_scene_library_object(source: &SceneSource, reference: &str) {
  let path: PathBuf = Path::new(&source.path)
    .parent()
    .and_then(Path::parent)
    .expect("the scene lies in a rawdata tree")
    .join(SceneSource::OBJECTS_DIRECTORY)
    .join(format!("{reference}.object"));

  fs::create_dir_all(path.parent().expect("an object lies in a directory")).expect("the object's directory is made");
  fs::write(path, "object").expect("the object is written");
}

/// Writes `object` as the library object at `reference` into the `objects` directory of the scene's rawdata tree.
pub(super) fn write_scene_object_file(source: &SceneSource, reference: &str, object: &EditableObjectFile) {
  let path: PathBuf = Path::new(&source.path)
    .parent()
    .and_then(Path::parent)
    .expect("the scene lies in a rawdata tree")
    .join(SceneSource::OBJECTS_DIRECTORY)
    .join(format!("{reference}.object"));

  fs::create_dir_all(path.parent().expect("an object lies in a directory")).expect("the object's directory is made");
  object
    .write_to_path::<XRayByteOrder, _>(&path)
    .expect("the object is written");
}

/// A static library object of one surface of `triangles`, drawn with `models\model` and a wood texture and of the game
/// material [`SCENE_GAME_MATERIAL`], as a level's visuals decompile into one.
pub(super) fn scene_library_object(triangles: &[[Vector3d; 3]]) -> EditableObjectFile {
  let vertices: Vec<LevelVertex> = triangles
    .iter()
    .flat_map(|[a, b, c]| {
      let cross: Vector3d = (b - a).cross(&(c - a));
      let normal: Vector3d = &cross / cross.length();

      [(a, 0.0, 0.0), (b, 1.0, 0.0), (c, 0.0, 1.0)].map(|(position, u, v)| LevelVertex {
        position: *position,
        normal,
        tangent: None,
        binormal: None,
        texture_coordinate: Vector2d::new(u, v),
        lightmap_coordinate: None,
        color: None,
        hemi: 0,
      })
    })
    .collect();

  ObjectDecompiler::decompile_level_geometry(&[LevelVisualGeometry {
    texture: String::from("wood\\wood_planks"),
    shader: String::from("models\\model"),
    dress: SurfaceDress {
      compiler_shader: String::from(SurfaceDress::DEFAULT_SETTING),
      game_material: String::from(SCENE_GAME_MATERIAL),
    },
    indices: (0..vertices.len() as u16).collect(),
    vertices,
  }])
  .expect("the triangles decompile into an object")
  .object
}

/// The game material every test library object's surface is of.
pub(super) const SCENE_GAME_MATERIAL: &str = "materials\\stone";

/// The text part of tool `T` holding `objects`, carrying the test scenes' GUID.
pub(super) fn scene_part<T: EditableToolObject>(objects: Vec<T>) -> Vec<u8> {
  EditableObjectPart::new(SCENE_GUID, objects)
    .to_text_part()
    .and_then(|part| part.write_to_bytes())
    .expect("the part writes")
}

/// An object named `name` standing at `x` along the scene's first axis.
pub(super) fn scene_custom_object(name: &str, x: f32) -> EditableCustomObject {
  EditableCustomObject::new(
    name,
    EditableObjectTransform {
      position: Vector3d::new(x, 2.0, 3.0),
      ..EditableObjectTransform::default()
    },
  )
}

/// The scenes as the plugin holds them, read again inside the commit that moved what one was read from, so a test reads
/// what a commit moved as soon as it returns.
pub(super) fn new_scenes(assets: &AssetMountState) -> SelectedScenes {
  SelectedScenes::new("scene", &SessionScopes::default()).with_derivation(assets, OpenedScene::derive)
}

/// Opens the scene `source` names as one more handle of `scenes`, as `levels|open_scene` does, its library objects
/// looked for in the `objects` directory beside its `levels` and its game data in `gamedata` beside its `rawdata` where
/// that directory exists.
pub(super) fn open_scene(
  scenes: &SelectedScenes,
  assets: &AssetMountState,
  source: &SceneSource,
) -> Arc<SessionSnapshot<SelectedScene>> {
  let roots: XrayRoots = source.get_roots().expect("the scene lies in a directory");
  let search: XraySearch = assets.search(&roots).expect("the scene's directory mounts");
  let revision: XrayRevision = search.get_vfs().get_revision();
  let opened: SessionDerived<SelectedScene> = OpenedScene {
    source: source.clone(),
    objects_roots: source.get_objects_roots(None),
    game: SceneGameData {
      roots: source.get_game_roots(None),
      is_dltx: false,
      engine: XrayEngineChoice::Auto,
    },
  }
  .read(search)
  .expect("the scene opens");

  scenes
    .begin_fixture_open(SessionId::generate())
    .unwrap()
    .commit_fixture(opened, revision)
    .unwrap()
}
