use std::sync::Arc;

use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::SessionSnapshot;
use xrf_chunk::ChunkLayout;
use xrf_editable_level::{
  EditableAiMapParams, EditableAiMapTool, EditableBinaryPart, EditableDetailObjectTool, EditableGlowObject,
  EditableLevelGuid, EditableLightObject, EditableObjectPart, EditableSectorItem, EditableSectorObject,
  EditableTextPart, EditableWayObject,
};
use xrf_ini::IniFile;
use xrf_math::{Aabb, Vector3d};
use xrf_vfs::XraySearch;

use crate::scene::{
  SceneBinaryToolsDescription, SceneBinaryToolsPart, SceneObjectsDescription, SceneObjectsPart, SceneProblem,
  SceneProblemKind, SceneSource, SceneToolKind, SelectedScene, SelectedSceneDescription, SelectedScenes,
};
use crate::tests::fixtures::{
  SCENE_GUID, new_scenes, open_scene, read_scene, scene_custom_object, scene_part, write_scene, write_scene_part,
};

fn edited_part(part: Vec<u8>, edit: impl FnOnce(&mut IniFile)) -> Vec<u8> {
  let mut part: EditableTextPart = EditableTextPart::read_from_bytes(&part).expect("the part reads");

  edit(&mut part.ini);
  part.write_to_bytes().expect("the part writes")
}

fn light_part() -> Vec<u8> {
  scene_part(vec![EditableLightObject::new(scene_custom_object("light_0000", 0.0))])
}

fn glow() -> EditableGlowObject {
  EditableGlowObject {
    object: scene_custom_object("glow_0000", 0.0),
    radius: 1.0,
    shader: String::from("effects\\glow"),
    texture: String::from("glow\\glow_white"),
    flags: 0,
  }
}

fn sector_part() -> Vec<u8> {
  scene_part(vec![EditableSectorObject {
    object: scene_custom_object("sector_0000", 0.0),
    color: [0, 255, 0, 255],
    is_default: false,
    items: vec![EditableSectorItem {
      object: String::from("stol_0000"),
      mesh: String::from("stol"),
    }],
    map_index: EditableSectorObject::NO_MAP,
  }])
}

fn ai_map_part() -> Vec<u8> {
  let bounds: Aabb = Aabb {
    min: Vector3d::ZERO,
    max: Vector3d::ONE,
  };

  EditableAiMapTool::new(bounds, EditableAiMapParams::default(), Vec::new())
    .to_part(SCENE_GUID)
    .and_then(|part| part.write_to_bytes())
    .expect("the AI map writes")
}

fn find_problem<'a>(problems: &'a [SceneProblem], tool: &str) -> &'a SceneProblem {
  problems
    .iter()
    .find(|problem| problem.tool == tool)
    .unwrap_or_else(|| panic!("a problem on {tool} among {problems:?}"))
}

fn open(assets: &AssetMountState, source: &SceneSource) -> Arc<SessionSnapshot<SelectedScene>> {
  let scenes: SelectedScenes = new_scenes(assets);

  open_scene(&scenes, assets, source)
}

// The editor skips such a stale copy, but opening without that tool's objects would show a different scene, so the
// open refuses, as the build does.
#[test]
fn refuses_a_scene_holding_a_part_saved_for_another_scene() {
  let source: SceneSource = write_scene("scene_problems/stale");
  let stale: EditableLevelGuid = EditableLevelGuid { g0: 1, g1: 2 };

  write_scene_part(&source, "ai_map.part", ai_map_part());
  write_scene_part(
    &source,
    "glow.part",
    EditableObjectPart::new(stale, vec![glow()])
      .to_text_part()
      .and_then(|part| part.write_to_bytes())
      .unwrap(),
  );

  let error: String = read_scene(&AssetMountState::new(), &source)
    .err()
    .expect("a stale part refuses the scene");

  assert!(error.contains("glow.part") && error.contains("GUID 1:2"), "{error}");
}

#[test]
fn keeps_each_part_the_editor_would_skip_as_a_problem_on_its_tool_and_opens_the_rest() {
  let source: SceneSource = write_scene("scene_problems/open");

  write_scene_part(
    &source,
    "light.part",
    edited_part(light_part(), |ini| ini.section_mut("main").set("version", "1")),
  );
  write_scene_part(
    &source,
    "sector.part",
    edited_part(sector_part(), |ini| ini.section_mut("object_0").set("clsid", "2")),
  );
  // The AI map's chunks saved under the wall mark tool's name.
  write_scene_part(&source, "wallmark.part", ai_map_part());
  write_scene_part(&source, "ps.part", "not a part");
  write_scene_part(&source, "ai_map.part", ai_map_part());

  let opened: Arc<SessionSnapshot<SelectedScene>> = open(&AssetMountState::new(), &source);
  let problems: &[SceneProblem] = &opened.problems;

  assert_eq!(find_problem(problems, "light").kind, SceneProblemKind::WrongVersion);
  assert_eq!(find_problem(problems, "sector").kind, SceneProblemKind::ForeignClass);
  assert_eq!(find_problem(problems, "wallmark").kind, SceneProblemKind::ForeignClass);
  assert_eq!(find_problem(problems, "ps").kind, SceneProblemKind::Unreadable);
  assert_eq!(find_problem(problems, "ps").file.as_deref(), Some("ps.part"));
  assert_eq!(problems.len(), 4, "{problems:?}");
  let described: SelectedSceneDescription = opened.describe();

  assert_eq!(
    described.tools.len(),
    5,
    "every tool with a part file is listed, those skipped among them"
  );
  assert_eq!(
    opened
      .scene
      .tools
      .iter()
      .map(|tool| tool.name.as_str())
      .collect::<Vec<_>>(),
    ["ai_map"],
    "only what the editor would load is kept to read"
  );
  assert_eq!(
    described
      .tools
      .iter()
      .find(|tool| tool.name == "light")
      .and_then(|tool| tool.files[0].guid.as_deref()),
    Some(SelectedScene::format_guid(&SCENE_GUID).as_str()),
    "a part's file names the scene it was saved for"
  );
}

#[test]
fn a_tool_whose_objects_do_not_read_is_a_problem_of_its_objects() {
  let source: SceneSource = write_scene("scene_problems/objects");
  let way: EditableWayObject = EditableWayObject {
    object: scene_custom_object("lx8_patrol_walk", 0.0),
    points: Vec::new(),
    kind: EditableWayObject::PATROL_PATH,
  };

  // Its version and class are the tool's, so the open keeps it; its points are counted but not there.
  write_scene_part(
    &source,
    "way.part",
    edited_part(scene_part(vec![way]), |ini| {
      ini.section_mut("object_0").set("wp_count", "2");
    }),
  );
  write_scene_part(&source, "glow.part", scene_part(vec![glow()]));

  let assets: AssetMountState = AssetMountState::new();
  let opened: Arc<SessionSnapshot<SelectedScene>> = open(&assets, &source);
  let objects: XraySearch = assets.search(&opened.objects_roots).unwrap();
  let described: Arc<SceneObjectsDescription> = SceneObjectsPart::get(&opened, &objects);

  assert!(opened.problems.is_empty(), "{:?}", opened.problems);
  assert_eq!(
    find_problem(&described.problems, "way").kind,
    SceneProblemKind::Unreadable
  );
  assert_eq!(find_problem(&described.problems, "way").file, None);
  assert_eq!(
    described.tools.iter().map(|tool| tool.tool).collect::<Vec<_>>(),
    [SceneToolKind::Glow]
  );
}

#[test]
fn a_binary_tool_whose_stream_does_not_read_is_a_problem_of_its_counts() {
  let source: SceneSource = write_scene("scene_problems/binary");
  let details: EditableBinaryPart = EditableBinaryPart {
    guid: SCENE_GUID,
    class_id: EditableDetailObjectTool::CLASS_ID,
    data: b"not a stream".to_vec(),
    layout: ChunkLayout::default(),
  };

  write_scene_part(&source, "detail_object.part", details.write_to_bytes().unwrap());
  write_scene_part(&source, "ai_map.part", ai_map_part());

  let opened: Arc<SessionSnapshot<SelectedScene>> = open(&AssetMountState::new(), &source);
  let counted: Arc<SceneBinaryToolsDescription> = SceneBinaryToolsPart::get(&opened);

  assert!(opened.problems.is_empty(), "{:?}", opened.problems);
  assert_eq!(
    find_problem(&counted.problems, "detail_object").kind,
    SceneProblemKind::Unreadable
  );
  assert_eq!(counted.tools.len(), 1, "the AI map counted");
}
