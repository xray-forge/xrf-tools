use std::sync::Arc;

use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::SessionSnapshot;
use xrf_editable_level::{
  EditableFogVolumeKind, EditableFogVolumeObject, EditableGlowObject, EditableGroupTool, EditableLightObject,
  EditableParticleObject, EditablePortalObject, EditableSceneObject, EditableShape, EditableShapeObject,
  EditableSoundEnvironmentObject, EditableSoundSourceObject, EditableSpawnPointKind, EditableSpawnPointObject,
  EditableTextPart, EditableToolObject, EditableWayLink, EditableWayObject, EditableWayPoint,
};
use xrf_ini::{IniFile, IniFormat, IniSection};
use xrf_math::{Vector2d, Vector3d};
use xrf_vfs::XraySearch;

use crate::scene::{
  SceneLightType, SceneObjectDescription, SceneObjectProperties, SceneObjectToolDescription, SceneObjectsDescription,
  SceneObjectsPart, SceneShapeUsage, SceneSource, SceneSpawnKind, SceneToolKind, SceneWayType, SelectedScene,
  SelectedScenes,
};
use crate::tests::fixtures::{
  SCENE_GUID, new_scenes, open_scene, scene_custom_object, scene_part, write_scene, write_scene_library_object,
  write_scene_part,
};

const GROUP_SECTION: &str = "object_0";

fn scene_object(name: &str, reference: &str) -> EditableSceneObject {
  EditableSceneObject {
    object: scene_custom_object(name, 0.0),
    reference: String::from(reference),
    flags: 0,
  }
}

fn spawn_point() -> EditableSpawnPointObject {
  let mut data: IniSection = IniSection::new("spawndata");

  data.set("000001", "1");
  data.set("000002", "\"space_restrictor\"");

  EditableSpawnPointObject {
    object: scene_custom_object("lx8_restrictor", 1.0),
    kind: EditableSpawnPointKind::Entity {
      section: String::from("space_restrictor"),
      flags: 0,
      data,
    },
    attached: Some(EditableShapeObject {
      object: scene_custom_object("lx8_restrictor_shape", 0.0),
      shape: EditableShape::new_unit_box(),
    }),
  }
}

fn way() -> EditableWayObject {
  let point = |name: &str, x: f32, links: Vec<EditableWayLink>| EditableWayPoint {
    name: String::from(name),
    position: Vector3d::new(x, 0.0, 0.0),
    flags: 1,
    selected: false,
    links,
  };

  EditableWayObject {
    object: scene_custom_object("lx8_patrol_walk", 0.0),
    points: vec![
      point(
        "wp00",
        0.0,
        vec![EditableWayLink {
          to: 1,
          probability: 1.0,
        }],
      ),
      point("wp01", 2.0, Vec::new()),
    ],
    kind: EditableWayObject::PATROL_PATH,
  }
}

fn sound_source() -> EditableSoundSourceObject {
  EditableSoundSourceObject {
    object: scene_custom_object("lamp_squeaks_0000", 0.0),
    sound: String::from("ambient\\lamp_squeaks_1"),
    flags: EditableSoundSourceObject::LOOPED,
    position: Vector3d::new(4.0, 5.0, 6.0),
    volume: 1.0,
    frequency: 1.0,
    min_distance: 1.0,
    max_distance: 50.0,
    max_ai_distance: 20.0,
    random_pause: Vector2d::new(0.0, 0.0),
    active_time: Vector2d::new(0.0, 0.0),
    play_time: Vector2d::new(0.0, 0.0),
  }
}

// Laid out as `CGroupObject::SaveLTX` saves a group.
fn group_part(name: &str, write_members: impl FnOnce(&mut IniFile)) -> Vec<u8> {
  let mut ini: IniFile = IniFile::default();

  SCENE_GUID.write_to_section(ini.section_mut("guid"));
  ini.section_mut("main").set("objects_count", "1");
  ini.section_mut("main").set("version", "0");
  ini.section_mut("modif").set("name", "");
  ini.section_mut("modif").set("time", "0");

  let group: &mut IniSection = ini.section_mut(GROUP_SECTION);

  group.set("clsid", EditableGroupTool::CLASS_ID.to_string());
  scene_custom_object(name, 7.0).write_to_section(group);
  write_members(&mut ini);

  ini
    .write_to_string_as(&IniFormat::XRAY_SAVE_AS)
    .expect("the group writes")
    .into_bytes()
}

fn write_member<T: EditableToolObject>(ini: &mut IniFile, index: usize, object: &T) {
  let section: String = format!("{GROUP_SECTION}_ingroup_{index}");

  ini.section_mut(&section).set("clsid", T::CLASS_ID.to_string());
  object.write_to_ini(ini, &section);
  ini
    .section_mut(GROUP_SECTION)
    .set("ingroup_count", (index + 1).to_string());
}

fn describe(source: &SceneSource) -> Arc<SceneObjectsDescription> {
  let assets: AssetMountState = AssetMountState::new();
  let scenes: SelectedScenes = new_scenes(&assets);
  let opened: Arc<SessionSnapshot<SelectedScene>> = open_scene(&scenes, &assets, source);
  let objects: XraySearch = assets.search(&opened.objects_roots).expect("the objects roots mount");

  SceneObjectsPart::get(&opened, &objects)
}

fn find_tool(described: &SceneObjectsDescription, tool: SceneToolKind) -> &SceneObjectToolDescription {
  described
    .tools
    .iter()
    .find(|described| described.tool == tool)
    .unwrap_or_else(|| panic!("tool {tool:?} is described"))
}

fn list_tools(described: &SceneObjectsDescription) -> Vec<SceneToolKind> {
  described.tools.iter().map(|tool| tool.tool).collect()
}

#[test]
fn describes_every_object_tools_objects_typed() {
  let source: SceneSource = write_scene("scene_objects/typed");
  let light: EditableLightObject = EditableLightObject::new(scene_custom_object("light_0000", 0.0));

  write_scene_part(&source, "light.part", scene_part(vec![light]));
  write_scene_part(
    &source,
    "glow.part",
    scene_part(vec![EditableGlowObject {
      object: scene_custom_object("glow_0000", 0.0),
      radius: 1.5,
      shader: String::from("effects\\glow"),
      texture: String::from("glow\\glow_white"),
      flags: EditableGlowObject::FIXED_SIZE,
    }]),
  );
  write_scene_part(
    &source,
    "portal.part",
    scene_part(vec![EditablePortalObject {
      object: scene_custom_object("portal_0000", 0.0),
      sector_front: String::from("sector_0000"),
      sector_back: String::from("sector_0001"),
      vertices: vec![
        Vector3d::ZERO,
        Vector3d::new(1.0, 0.0, 0.0),
        Vector3d::new(0.0, 1.0, 0.0),
      ],
    }]),
  );
  write_scene_part(&source, "spawn.part", scene_part(vec![spawn_point()]));
  write_scene_part(
    &source,
    "shape.part",
    scene_part(vec![EditableShapeObject {
      object: scene_custom_object("shape_0000", 0.0),
      shape: EditableShape::new_unit_box(),
    }]),
  );
  write_scene_part(&source, "way.part", scene_part(vec![way()]));
  write_scene_part(
    &source,
    "ps.part",
    scene_part(vec![EditableParticleObject {
      object: scene_custom_object("exhaust_0000", 0.0),
      reference: String::from("industrial_particles\\exhaust"),
      game_types: EditableParticleObject::ALL_GAME_TYPES,
    }]),
  );
  write_scene_part(&source, "sound_src.part", scene_part(vec![sound_source()]));
  write_scene_part(
    &source,
    "sound_env.part",
    scene_part(vec![EditableSoundEnvironmentObject {
      object: scene_custom_object("sound_env_0000", 0.0),
      shape: EditableShape::new_unit_box(),
      inner: String::from("cave"),
      outer: String::from("default"),
    }]),
  );
  write_scene_part(
    &source,
    "fog_volume.part",
    scene_part(vec![EditableFogVolumeObject {
      object: scene_custom_object("fog_volume_0000", 0.0),
      shape: EditableShape::new_unit_box(),
      kind: EditableFogVolumeKind::Occlusion,
      group: EditableFogVolumeObject::UNGROUPED,
    }]),
  );

  let described: Arc<SceneObjectsDescription> = describe(&source);

  assert!(described.problems.is_empty(), "{:?}", described.problems);
  assert_eq!(
    list_tools(&described),
    [
      SceneToolKind::Glow,
      SceneToolKind::Light,
      SceneToolKind::Shape,
      SceneToolKind::SoundSource,
      SceneToolKind::Spawn,
      SceneToolKind::Way,
      SceneToolKind::Portal,
      SceneToolKind::SoundEnvironment,
      SceneToolKind::Particle,
      SceneToolKind::FogVolume,
    ],
    "the tools in the editor's order"
  );

  let light: &SceneObjectDescription = &find_tool(&described, SceneToolKind::Light).objects[0];

  assert_eq!(light.id.tool, SceneToolKind::Light);
  assert_eq!(light.id.index, 0);
  assert_eq!(light.name, "light_0000");
  assert_eq!(light.transform.position, Vector3d::new(0.0, 2.0, 3.0));
  assert!(matches!(
    light.properties,
    SceneObjectProperties::Light { range, light_type: SceneLightType::Point, .. } if range == 8.0
  ));

  let SceneObjectProperties::Spawn { spawn, attached } =
    &find_tool(&described, SceneToolKind::Spawn).objects[0].properties
  else {
    panic!("a spawn point's properties");
  };
  let SceneSpawnKind::Entity { section, data, .. } = spawn else {
    panic!("an entity");
  };

  assert_eq!(section, "space_restrictor");
  assert_eq!(
    data
      .iter()
      .map(|field| (field.key.as_str(), field.value.as_str()))
      .collect::<Vec<_>>(),
    [("000001", "1"), ("000002", "\"space_restrictor\"")],
    "the entity's data as ordered fields"
  );
  assert_eq!(attached.as_ref().map(|shape| shape.elements.len()), Some(1));

  let SceneObjectProperties::Way { points, way_type } =
    &find_tool(&described, SceneToolKind::Way).objects[0].properties
  else {
    panic!("a way's properties");
  };

  assert_eq!(*way_type, SceneWayType::PatrolPath);
  assert_eq!(points.len(), 2);
  assert_eq!(points[0].links[0].to, 1);

  assert!(matches!(
    &find_tool(&described, SceneToolKind::SoundSource).objects[0].properties,
    SceneObjectProperties::SoundSource { sound, max_distance, .. }
      if sound == "ambient\\lamp_squeaks_1" && *max_distance == 50.0
  ));
  assert!(matches!(
    &find_tool(&described, SceneToolKind::FogVolume).objects[0].properties,
    SceneObjectProperties::FogVolume {
      profile: None,
      group: None,
      ..
    }
  ));
  assert!(matches!(
    &find_tool(&described, SceneToolKind::Portal).objects[0].properties,
    SceneObjectProperties::Portal { vertices, .. } if vertices.len() == 3
  ));
  assert!(matches!(
    &find_tool(&described, SceneToolKind::Glow).objects[0].properties,
    SceneObjectProperties::Glow { radius, .. } if *radius == 1.5
  ));
  assert!(matches!(
    &find_tool(&described, SceneToolKind::Particle).objects[0].properties,
    SceneObjectProperties::Particle { reference, .. } if reference == "industrial_particles\\exhaust"
  ));
  assert!(matches!(
    &find_tool(&described, SceneToolKind::SoundEnvironment).objects[0].properties,
    SceneObjectProperties::SoundEnvironment { inner, .. } if inner == "cave"
  ));
  assert!(matches!(
    &find_tool(&described, SceneToolKind::Shape).objects[0].properties,
    SceneObjectProperties::Shape { shape } if shape.elements.len() == 1 && shape.usage == SceneShapeUsage::Common
  ));
}

#[test]
fn says_which_scene_objects_library_object_is_found_and_which_is_missing() {
  let source: SceneSource = write_scene("scene_objects/references");

  write_scene_library_object(&source, "statics\\stol\\stol_4");
  write_scene_part(
    &source,
    "scene_object.part",
    scene_part(vec![
      scene_object("stol_0000", "statics\\stol\\stol_4"),
      scene_object("lost_0000", "statics\\lost\\lost_1"),
    ]),
  );

  let described: Arc<SceneObjectsDescription> = describe(&source);
  let found: Vec<(&str, bool)> = find_tool(&described, SceneToolKind::SceneObject)
    .objects
    .iter()
    .map(|object| match &object.properties {
      SceneObjectProperties::SceneObject { is_found, .. } => (object.name.as_str(), *is_found),
      properties => panic!("a scene object's properties, not {properties:?}"),
    })
    .collect();

  assert_eq!(found, [("stol_0000", true), ("lost_0000", false)]);
}

#[test]
fn describes_a_groups_members_under_their_own_tools_with_the_groups_name() {
  let source: SceneSource = write_scene("scene_objects/groups");

  write_scene_part(
    &source,
    "scene_object.part",
    scene_part(vec![scene_object("own_0000", "statics\\stol\\stol_4")]),
  );
  write_scene_part(
    &source,
    "group.part",
    group_part("lab_group", |ini| {
      write_member(ini, 0, &scene_object("grouped_0000", "statics\\stol\\stol_4"));
      write_member(
        ini,
        1,
        &EditableLightObject::new(scene_custom_object("grouped_light", 0.0)),
      );
    }),
  );

  let described: Arc<SceneObjectsDescription> = describe(&source);

  assert!(described.problems.is_empty(), "{:?}", described.problems);
  assert_eq!(
    list_tools(&described),
    [SceneToolKind::Group, SceneToolKind::SceneObject, SceneToolKind::Light],
    "the groups in the editor's order of tools, though they are read last"
  );
  assert_eq!(
    find_tool(&described, SceneToolKind::SceneObject)
      .objects
      .iter()
      .map(|object| (object.id.index, object.name.as_str(), object.group.as_deref()))
      .collect::<Vec<_>>(),
    [(0, "grouped_0000", Some("lab_group")), (1, "own_0000", None)],
    "the members first, as the editor loads the group tool first"
  );
  assert_eq!(
    find_tool(&described, SceneToolKind::Light).objects[0].group.as_deref(),
    Some("lab_group"),
    "a member of a tool the scene has no part of"
  );

  let groups: &SceneObjectToolDescription = find_tool(&described, SceneToolKind::Group);

  assert_eq!(groups.objects[0].name, "lab_group");
  assert_eq!(groups.objects[0].transform.position.x, 7.0);
  assert!(matches!(
    groups.objects[0].properties,
    SceneObjectProperties::Group { members: 2 }
  ));
}

// The editor loads the group tool first, so it holds the members of a tool whose own part it then fails.
#[test]
fn keeps_a_groups_members_of_a_tool_whose_own_part_does_not_read() {
  let source: SceneSource = write_scene("scene_objects/unread_own_part");
  let mut own: EditableTextPart = EditableTextPart::read_from_bytes(&scene_part(vec![way()])).expect("the part reads");

  // Its version and class are the tool's, so the open keeps it; its points are counted but not there.
  own.ini.section_mut("object_0").set("wp_count", "3");
  write_scene_part(&source, "way.part", own.write_to_bytes().expect("the part writes"));
  write_scene_part(
    &source,
    "group.part",
    group_part("lab_group", |ini| write_member(ini, 0, &way())),
  );

  let described: Arc<SceneObjectsDescription> = describe(&source);

  assert_eq!(
    find_tool(&described, SceneToolKind::Way)
      .objects
      .iter()
      .map(|object| (object.id.index, object.name.as_str(), object.group.as_deref()))
      .collect::<Vec<_>>(),
    [(0, "lx8_patrol_walk", Some("lab_group"))],
    "the member indexed as when the part reads"
  );
  assert_eq!(
    described
      .problems
      .iter()
      .map(|problem| (problem.tool.as_str(), problem.file.as_deref()))
      .collect::<Vec<_>>(),
    [("way", None)],
    "{:?}",
    described.problems
  );
  assert!(matches!(
    find_tool(&described, SceneToolKind::Group).objects[0].properties,
    SceneObjectProperties::Group { members: 1 }
  ));
}
