//! A Level Editor scene opened as a document: what its open lists of its tools' part files, where it looks for its
//! library objects, and how a tool restores and closes it.

use std::path::Path;
use std::sync::Arc;

use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionScope, SessionSnapshot};
use xrf_chunk::XRayByteOrder;
use xrf_editable_level::{
  EditableAiMapNode, EditableAiMapParams, EditableAiMapTool, EditableSceneObject, EditableSectorItem,
  EditableSectorObject, EditableWallmarkSlot, EditableWallmarkTool,
};
use xrf_math::{Aabb, AiNodePosition, Vector3d};
use xrf_vfs::{XrayMountMode, XrayRoot, XrayRoots};

use crate::scene::{
  SceneBinaryToolDescription, SceneBinaryToolsDescription, SceneBinaryToolsPart, SceneSource, SceneToolDescription,
  SceneToolFormat, SceneToolKind, SelectedScene, SelectedSceneDescription, SelectedScenes,
};
use crate::tests::fixtures::{
  SCENE_GUID, SCENE_NAME, new_scenes, open_scene, scene_custom_object, scene_part, write_scene, write_scene_part,
};

fn ai_map_part() -> Vec<u8> {
  let node: EditableAiMapNode = EditableAiMapNode {
    links: [EditableAiMapNode::NO_LINK; 4],
    plane: 0,
    position: AiNodePosition { x: 1, y: 0, z: 2 },
    flags: 0,
  };
  let bounds: Aabb = Aabb {
    min: Vector3d::ZERO,
    max: Vector3d::ONE,
  };

  EditableAiMapTool::new(bounds, EditableAiMapParams::default(), vec![node, node])
    .to_part::<XRayByteOrder>(SCENE_GUID)
    .and_then(|part| part.write_to_bytes::<XRayByteOrder>())
    .expect("the AI map writes")
}

fn wallmark_part() -> Vec<u8> {
  let slot: EditableWallmarkSlot = EditableWallmarkSlot {
    shader: String::from("effects\\wallmark"),
    texture: String::from("wm\\wm_blood_1"),
    marks: Vec::new(),
  };

  EditableWallmarkTool::new(vec![slot])
    .to_part::<XRayByteOrder>(SCENE_GUID)
    .and_then(|part| part.write_to_bytes::<XRayByteOrder>())
    .expect("the wall marks write")
}

fn sector() -> EditableSectorObject {
  EditableSectorObject {
    object: scene_custom_object("sector_0000", 0.0),
    color: [255, 0, 0, 255],
    is_default: true,
    items: vec![EditableSectorItem {
      object: String::from("stol_0000"),
      mesh: String::from("stol"),
    }],
    map_index: EditableSectorObject::NO_MAP,
  }
}

#[test]
fn opens_a_scene_listing_its_tools_part_files_and_what_its_level_holds() {
  let source: SceneSource = write_scene("scene_open/listing");
  let scene_objects: Vec<u8> = scene_part(vec![EditableSceneObject {
    object: scene_custom_object("stol_0000", 0.0),
    reference: String::from("statics\\stol\\stol_4"),
    flags: 0,
  }]);
  let sectors: Vec<u8> = scene_part(vec![sector()]);

  write_scene_part(&source, "scene_object.part", &scene_objects);
  write_scene_part(&source, "sector.part", &sectors);
  write_scene_part(&source, "ai_map.part", ai_map_part());
  write_scene_part(&source, "wallmark.part", wallmark_part());
  // A tool the editor does not save, whose name sorts before every tool it does.
  write_scene_part(&source, "a_custom.part", &sectors);
  // The editor's backup of an earlier save, which is no part of the scene.
  write_scene_part(&source, "sector.~part", "an earlier save");

  let assets: AssetMountState = AssetMountState::new();
  let scenes: SelectedScenes = new_scenes(&assets);
  let opened: Arc<SessionSnapshot<SelectedScene>> = open_scene(&scenes, &assets, &source);
  let described: SelectedSceneDescription = opened.describe();

  assert_eq!(described.name, SCENE_NAME);
  assert_eq!(described.guid, "5065284203709945560:13177767572218497455");
  assert!(described.problems.is_empty(), "{:?}", described.problems);
  assert!(described.build_params.is_some() && described.camera.is_some());
  assert_eq!(
    described
      .tools
      .iter()
      .map(|tool| (tool.name.as_str(), tool.kind, tool.format, tool.files.len()))
      .collect::<Vec<_>>(),
    [
      (
        "scene_object",
        Some(SceneToolKind::SceneObject),
        Some(SceneToolFormat::Object),
        1
      ),
      ("sector", Some(SceneToolKind::Sector), Some(SceneToolFormat::Object), 1),
      ("ai_map", Some(SceneToolKind::AiMap), Some(SceneToolFormat::Binary), 1),
      (
        "wallmark",
        Some(SceneToolKind::Wallmark),
        Some(SceneToolFormat::Binary),
        1
      ),
      ("a_custom", None, Some(SceneToolFormat::Object), 1),
    ],
    "the editor's tools in its order, any other after them"
  );

  let sector_tool: &SceneToolDescription = described.tools.iter().find(|tool| tool.name == "sector").unwrap();

  assert_eq!(sector_tool.size, sectors.len() as u64);
  assert_eq!(sector_tool.files[0].name, "sector.part");
  assert_eq!(sector_tool.files[0].guid.as_deref(), Some(described.guid.as_str()));
  assert_eq!(
    opened.scene.tools.len(),
    5,
    "every part the editor would load is kept to read"
  );
}

#[test]
fn reads_a_tools_numbered_part_files_after_its_first_until_one_is_missing() {
  let source: SceneSource = write_scene("scene_open/numbered");
  let sectors: Vec<u8> = scene_part(vec![sector()]);

  write_scene_part(&source, "sector.part", &sectors);
  write_scene_part(&source, "sector.part1", &sectors);
  // Past the missing `sector.part2`, which the editor stops reading at.
  write_scene_part(&source, "sector.part3", &sectors);

  let assets: AssetMountState = AssetMountState::new();
  let opened: Arc<SessionSnapshot<SelectedScene>> = open_scene(&new_scenes(&assets), &assets, &source);
  let described: SelectedSceneDescription = opened.describe();
  let sector_tool: &SceneToolDescription = described.tools.iter().find(|tool| tool.name == "sector").unwrap();

  assert!(described.problems.is_empty(), "{:?}", described.problems);
  assert_eq!(
    sector_tool
      .files
      .iter()
      .map(|file| file.name.as_str())
      .collect::<Vec<_>>(),
    ["sector.part", "sector.part1"]
  );
  assert_eq!(sector_tool.size, 2 * sectors.len() as u64);
  assert_eq!(
    opened.scene.get_tool("sector").map(|tool| tool.parts.len()),
    Some(2),
    "each file read is kept as one of the tool's parts"
  );
}

#[test]
fn counts_the_binary_tools_without_their_payload() {
  let source: SceneSource = write_scene("scene_open/binary");

  write_scene_part(&source, "ai_map.part", ai_map_part());
  write_scene_part(&source, "wallmark.part", wallmark_part());

  let assets: AssetMountState = AssetMountState::new();
  let scenes: SelectedScenes = new_scenes(&assets);
  let opened: Arc<SessionSnapshot<SelectedScene>> = open_scene(&scenes, &assets, &source);
  let counted: Arc<SceneBinaryToolsDescription> = SceneBinaryToolsPart::get(&opened);

  assert!(counted.problems.is_empty(), "{:?}", counted.problems);
  assert_eq!(
    counted.tools,
    [
      SceneBinaryToolDescription::AiMap { nodes: 2 },
      SceneBinaryToolDescription::Wallmark { slots: 1, marks: 0 },
    ]
  );
}

#[test]
fn a_scene_with_no_part_directory_opens_holding_no_tool() {
  let source: SceneSource = write_scene("scene_open/no_parts");

  std::fs::remove_dir(Path::new(&source.path).with_extension("")).unwrap();

  let assets: AssetMountState = AssetMountState::new();
  let opened: Arc<SessionSnapshot<SelectedScene>> = open_scene(&new_scenes(&assets), &assets, &source);

  assert!(opened.tools.is_empty());
  assert!(opened.problems.is_empty());
}

#[test]
fn a_scene_is_searched_from_its_levels_directory_and_its_library_objects_beside_it() {
  let source: SceneSource = SceneSource {
    path: String::from("C:\\rawdata\\levels\\labx8.level"),
  };

  assert_eq!(source.require_name().unwrap(), SCENE_NAME);
  assert_eq!(
    source.get_roots().unwrap(),
    XrayRoots::new([XrayRoot::new("C:\\rawdata\\levels".into(), XrayMountMode::Directory)])
  );
  assert_eq!(
    source.get_objects_roots(None),
    XrayRoots::new([XrayRoot::new(
      Path::new("C:\\rawdata").join(SceneSource::OBJECTS_DIRECTORY),
      XrayMountMode::Directory
    )])
  );
  assert_eq!(
    source.get_objects_roots(Some(vec![String::from("D:\\library")])),
    XrayRoots::new([XrayRoot::new("D:\\library".into(), XrayMountMode::Directory)]),
    "the roots the page names win"
  );
}

#[test]
fn refuses_a_path_naming_no_scene() {
  for path in [
    "C:\\rawdata\\levels\\labx8.part",
    "C:\\rawdata\\levels\\.level",
    "labx8",
  ] {
    assert!(
      SceneSource {
        path: String::from(path)
      }
      .require_name()
      .is_err(),
      "{path}"
    );
  }
}

#[test]
fn a_tool_restores_the_scene_it_holds_until_it_closes_it() {
  let source: SceneSource = write_scene("scene_open/restore");
  let assets: AssetMountState = AssetMountState::new();
  let scenes: SelectedScenes = new_scenes(&assets);
  let opened: Arc<SessionSnapshot<SelectedScene>> = open_scene(&scenes, &assets, &source);
  // What a fixture's open binds its handle to, as a tool's scope.
  let scope: SessionScope = SessionScope::new("fixture", opened.session_id.to_string());
  let restored: SessionSnapshot<SelectedSceneDescription> = scenes
    .restore_as(&scope, SelectedScene::describe)
    .expect("the tool holds the scene");

  assert_eq!(restored.session_id, opened.session_id);
  assert_eq!(restored.revision, opened.revision);
  assert_eq!(*restored.value, opened.describe());

  scenes.close(&[opened.session_id]);

  assert!(scenes.restore(&scope).is_none(), "a closed scene restores nothing");
  assert!(scenes.require(opened.session_id).is_err());
}
