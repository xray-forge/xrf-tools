//! Shared test builders: levels, searches, catalogs, spawned objects, Level Editor scenes on disk, and a silent audio
//! server.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionDerived, SessionId, SessionScopes, SessionSnapshot};
use xrf_audio::{AudioEngineParts, AudioEngineSettings, AudioEvent, AudioServer};
use xrf_editable_level::{
  EditableBuildParams, EditableCustomObject, EditableLevelCamera, EditableLevelFile, EditableLevelGuid,
  EditableLevelOptions, EditableObjectPart, EditableObjectTransform, EditableToolObject,
};
use xrf_editable_object::{EditableObjectFile, EditableSurface, EditableSurfaceFaces};
use xrf_engine_target::{XrayEngine, XrayEngineChoice};
use xrf_environment::{EnvironmentCatalog, WeatherGraphs};
use xrf_level::{LevelFile, LevelHeaderChunk, LevelShaderEntry, LevelShadersChunk, LevelVertex};
use xrf_material::fixtures::FixtureTree;
use xrf_math::{Vector2d, Vector3d};
use xrf_rawdata::{LevelVisualGeometry, ObjectDecompiler, SurfaceDress};
use xrf_shaders::{ShaderCompilerFile, ShaderCompilerShader};
use xrf_spawn::{AlifeObject, AlifeObjectAbstract, AlifeObjectInherited, ClsId, SpawnPacketHeader};
use xrf_test_utils::GeneratedResources;
use xrf_vfs::{XrayLookupScope, XrayMountId, XrayRevision, XrayRoots, XraySearch, XrayVfs};

use crate::scene::{OpenedScene, SceneGameData, SceneSource, SelectedScene, SelectedScenes};

/// The GUID of every test scene and its parts (`labx8`'s).
pub(super) const SCENE_GUID: EditableLevelGuid = EditableLevelGuid {
  g0: 5065284203709945560,
  g1: 13177767572218497455,
};

/// The name of every test scene and its part directory.
pub(super) const SCENE_NAME: &str = "labx8";

/// An audio server with no device, as the audio plugin starts on a machine without one.
pub(super) fn silent_server() -> AudioServer {
  let AudioEngineParts { engine, .. } = AudioEngineParts::new(AudioEngineSettings::default());

  AudioServer::start_on(engine, None, |_: AudioEvent| {}).expect("the audio control starts")
}

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

/// A fresh rawdata tree under `resource` holding scene `labx8` with only its `.level`, as the CoP SDK editor saves a
/// new scene.
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

/// Write `bytes` as the scene's part file `file`.
pub(super) fn write_scene_part(source: &SceneSource, file: &str, bytes: impl AsRef<[u8]>) {
  let parts: PathBuf = Path::new(&source.path).with_extension("");

  fs::write(parts.join(file), bytes).expect("the part is written");
}

/// Write a placeholder library object at `reference` into the tree's `objects` directory.
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

/// Write `object` as the library object at `reference` into the tree's `objects` directory.
pub(super) fn write_scene_object_file(source: &SceneSource, reference: &str, object: &EditableObjectFile) {
  let path: PathBuf = Path::new(&source.path)
    .parent()
    .and_then(Path::parent)
    .expect("the scene lies in a rawdata tree")
    .join(SceneSource::OBJECTS_DIRECTORY)
    .join(format!("{reference}.object"));

  fs::create_dir_all(path.parent().expect("an object lies in a directory")).expect("the object's directory is made");
  object.write_to_path(&path).expect("the object is written");
}

/// A static library object of one `models\model` wood surface of `triangles` and game material
/// [`SCENE_GAME_MATERIAL`], as level visuals decompile.
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

pub(super) const SCENE_GAME_MATERIAL: &str = "materials\\stone";

/// A tree collision hull's compiler shader: collides, not drawn.
pub(super) const COLLISION_COMPILER_SHADER: &str = "flora\\flora_collision";

/// A compiler shader library of `default` (drawn and colliding) and [`COLLISION_COMPILER_SHADER`].
pub(super) fn compiler_library() -> ShaderCompilerFile {
  let shader = |name: &str, flags: u32| ShaderCompilerShader {
    name: String::from(name),
    name_trailing: vec![0; ShaderCompilerShader::NAME_SIZE - name.len()],
    flags,
    vertex_translucency: 0.0,
    vertex_ambient: 0.0,
    lightmap_density: 1.0,
  };

  ShaderCompilerFile {
    shaders: vec![
      shader(
        SurfaceDress::DEFAULT_SETTING,
        ShaderCompilerShader::COLLISION | ShaderCompilerShader::RENDERING,
      ),
      shader(COLLISION_COMPILER_SHADER, ShaderCompilerShader::COLLISION),
    ],
  }
}

/// A ground unit square from [`scene_library_object`] with its second triangle on a `hull` surface of
/// `compiler_shader`.
pub(super) fn scene_parted_object(compiler_shader: &str) -> EditableObjectFile {
  let [a, b, c, d] = [
    Vector3d::new(0.0, 0.0, 0.0),
    Vector3d::new(1.0, 0.0, 0.0),
    Vector3d::new(1.0, 0.0, 1.0),
    Vector3d::new(0.0, 0.0, 1.0),
  ];
  let mut object: EditableObjectFile = scene_library_object(&[[a, c, b], [a, d, c]]);
  let surfaces: &mut Vec<EditableSurface> = object.surfaces.as_mut().expect("the object has surfaces");
  let hull: EditableSurface = EditableSurface {
    name: String::from("hull"),
    compiler_shader: String::from(compiler_shader),
    ..surfaces[0].clone()
  };
  let drawn: String = surfaces[0].name.clone();

  surfaces.push(hull);

  for mesh in object.meshes.iter_mut().flatten() {
    let moved: Vec<u32> = mesh.surface_faces[0].faces.split_off(1);

    mesh.surface_faces.push(EditableSurfaceFaces {
      surface: String::from("hull"),
      faces: moved,
    });
    assert_eq!(mesh.surface_faces[0].surface, drawn);
  }

  object
}

/// Tool `T`'s text part holding `objects`, with the test scene GUID.
pub(super) fn scene_part<T: EditableToolObject>(objects: Vec<T>) -> Vec<u8> {
  EditableObjectPart::new(SCENE_GUID, objects)
    .to_text_part()
    .and_then(|part| part.write_to_bytes())
    .expect("the part writes")
}

/// An object named `name` at `x` along the first axis.
pub(super) fn scene_custom_object(name: &str, x: f32) -> EditableCustomObject {
  EditableCustomObject::new(
    name,
    EditableObjectTransform {
      position: Vector3d::new(x, 2.0, 3.0),
      ..EditableObjectTransform::default()
    },
  )
}

/// The plugin's scenes, derived again inside the commit itself, so a test sees a commit's effect as soon as it returns.
pub(super) fn new_scenes(assets: &AssetMountState) -> SelectedScenes {
  SelectedScenes::new("scene", &SessionScopes::default()).with_derivation(assets, OpenedScene::derive)
}

/// Read the scene at `source` as an open does without opening it: what [`open_scene`] commits, or why it refuses.
pub(super) fn read_scene(assets: &AssetMountState, source: &SceneSource) -> TauriResult<SessionDerived<SelectedScene>> {
  let roots: XrayRoots = source.get_roots().expect("the scene lies in a directory");
  let search: XraySearch = assets.search(&roots).expect("the scene's directory mounts");

  OpenedScene {
    source: source.clone(),
    objects_roots: source.get_objects_roots(None),
    game: SceneGameData {
      roots: source.get_game_roots(None),
      is_dltx: false,
      engine: XrayEngineChoice::Auto,
    },
  }
  .read(search)
}

/// Open the scene at `source` as one more handle of `scenes`, as `levels|open_scene` does, with the default objects and
/// game data roots.
pub(super) fn open_scene(
  scenes: &SelectedScenes,
  assets: &AssetMountState,
  source: &SceneSource,
) -> Arc<SessionSnapshot<SelectedScene>> {
  let roots: XrayRoots = source.get_roots().expect("the scene lies in a directory");
  let revision: XrayRevision = assets
    .search(&roots)
    .expect("the scene's directory mounts")
    .get_vfs()
    .get_revision();
  let opened: SessionDerived<SelectedScene> = read_scene(assets, source).expect("the scene opens");

  scenes
    .begin_fixture_open(SessionId::generate())
    .unwrap()
    .commit_fixture(opened, revision)
    .unwrap()
}
