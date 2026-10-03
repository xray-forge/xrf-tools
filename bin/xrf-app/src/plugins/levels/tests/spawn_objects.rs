use std::collections::HashMap;

use xrf_math::Vector3d;
use xrf_spawn::{
  AlifeGraphPoint, AlifeObject, AlifeObjectAbstract, AlifeObjectDynamicVisual, AlifeObjectHangingLamp,
  AlifeObjectInherited, AlifeObjectItem, AlifeObjectPhysic, AlifeObjectSkeleton, ClsId,
};

use crate::plugins::levels::spawn_objects::describe_spawn_objects;
use crate::plugins::levels::state::{
  LevelSpawn, LevelSpawnCategory, LevelSpawnObject, LevelSpawnObjectsDescription, LevelSpawnRelease,
};

fn new_abstract(story_id: u32) -> AlifeObjectAbstract {
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

fn new_visual(visual: &str, story_id: u32) -> AlifeObjectDynamicVisual {
  AlifeObjectDynamicVisual {
    base: new_abstract(story_id),
    visual_name: String::from(visual),
    visual_flags: 0,
  }
}

fn new_skeleton() -> AlifeObjectSkeleton {
  AlifeObjectSkeleton {
    name: String::from("$editor"),
    flags: 0,
    source_id: u16::MAX,
  }
}

fn new_physic(visual: &str, story_id: u32) -> AlifeObjectInherited {
  AlifeObjectInherited::CseAlifeObjectPhysic(Box::new(AlifeObjectPhysic {
    base: new_visual(visual, story_id),
    skeleton: new_skeleton(),
    physic_type: 0,
    mass: 10.0,
    fixed_bones: String::new(),
  }))
}

fn new_item(visual: &str) -> AlifeObjectInherited {
  AlifeObjectInherited::CseAlifeItem(Box::new(AlifeObjectItem {
    base: new_visual(visual, AlifeObjectAbstract::INVALID_STORY_ID),
    condition: 1.0,
    upgrades_count: 0,
  }))
}

fn new_lamp(light_flags: u16) -> AlifeObjectInherited {
  AlifeObjectInherited::CseAlifeObjectHangingLamp(Box::new(AlifeObjectHangingLamp {
    base: new_visual("dynamics\\light\\light_lamp", AlifeObjectAbstract::INVALID_STORY_ID),
    skeleton: new_skeleton(),
    main_color: 0,
    main_brightness: 1.0,
    color_animator: String::new(),
    main_range: 8.0,
    light_flags,
    startup_animation: String::new(),
    fixed_bones: String::new(),
    health: 1.0,
    virtual_size: 0.1,
    ambient_radius: 0.0,
    ambient_power: 0.0,
    ambient_texture: String::new(),
    light_texture: String::new(),
    light_bone: String::new(),
    spot_cone_angle: 1.0,
    glow_texture: String::new(),
    glow_radius: 0.0,
    light_ambient_bone: String::new(),
    volumetric_quality: 1.0,
    volumetric_intensity: 1.0,
    volumetric_distance: 1.0,
  }))
}

fn new_graph_point() -> AlifeObjectInherited {
  AlifeObjectInherited::CseAlifeGraphPoint(Box::new(AlifeGraphPoint {
    connection_point_name: String::new(),
    connection_level_name: String::new(),
    location0: 0,
    location1: 0,
    location2: 0,
    location3: 0,
  }))
}

fn new_object(name: &str, section: &str, clsid: ClsId, inherited: AlifeObjectInherited) -> AlifeObject {
  AlifeObject {
    id: 0,
    net_action: 1,
    section: String::from(section),
    clsid,
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
    client_data_size: 0,
    spawn_id: 0,
    inherited,
    update_data: Vec::new(),
  }
}

fn describe(objects: Vec<AlifeObject>) -> LevelSpawnObjectsDescription {
  describe_spawn_objects(&LevelSpawn {
    arrivals: Vec::new(),
    objects,
    releases: HashMap::new(),
  })
}

#[test]
fn draws_every_object_with_a_visual_in_the_category_of_its_class() {
  let described: LevelSpawnObjectsDescription = describe(vec![
    new_object("crate", "physic_object", ClsId::OPhysS, new_physic("physics\\box", 7)),
    new_object("patrol", "graph_point", ClsId::AiGraph, new_graph_point()),
    new_object("medkit", "medkit", ClsId::SFood, new_item("dynamics\\medkit")),
    new_object(
      "lamp",
      "hanging_lamp",
      ClsId::SoHLamp,
      new_lamp(AlifeObjectHangingLamp::FLAG_R2),
    ),
  ]);
  let summary: Vec<(u32, &str, LevelSpawnCategory, u32, Option<u32>)> = described
    .objects
    .iter()
    .map(|it: &LevelSpawnObject| (it.index, it.name.as_str(), it.category, it.visual, it.story_id))
    .collect();

  assert_eq!(
    summary,
    vec![
      (0, "crate", LevelSpawnCategory::Props, 0, Some(7)),
      (2, "medkit", LevelSpawnCategory::Items, 1, None),
      (3, "lamp", LevelSpawnCategory::Lamps, 2, None),
    ]
  );
  assert_eq!(
    described.visuals,
    vec!["physics\\box", "dynamics\\medkit", "dynamics\\light\\light_lamp"]
  );
}

#[test]
fn names_a_shared_visual_once() {
  let described: LevelSpawnObjectsDescription = describe(vec![
    new_object("first", "medkit", ClsId::SFood, new_item("dynamics\\medkit")),
    new_object("second", "medkit", ClsId::SFood, new_item("dynamics\\medkit")),
  ]);

  assert_eq!(described.visuals, vec!["dynamics\\medkit"]);
  assert!(described.objects.iter().all(|it: &LevelSpawnObject| it.visual == 0));
}

// The engine spawns a lamp without the R2 flag on R1 alone, and an object naming no visual draws nothing.
#[test]
fn passes_over_what_the_game_draws_nothing_of() {
  let described: LevelSpawnObjectsDescription = describe(vec![
    new_object("lamp", "hanging_lamp", ClsId::SoHLamp, new_lamp(0)),
    new_object("empty", "physic_object", ClsId::OPhysS, new_physic("", 1)),
  ]);

  assert!(described.objects.is_empty());
  assert!(described.visuals.is_empty());
}

// A new game's releases stay in the description, so a panel can say what the level loses and a view can draw them.
#[test]
fn marks_what_a_new_game_releases_and_why() {
  let described: LevelSpawnObjectsDescription = describe_spawn_objects(&LevelSpawn {
    arrivals: Vec::new(),
    objects: vec![
      new_object("kept", "medkit", ClsId::SFood, new_item("dynamics\\medkit")),
      new_object("removed", "medkit", ClsId::SFood, new_item("dynamics\\medkit")),
      new_object("replaced", "medkit", ClsId::SFood, new_item("dynamics\\medkit")),
    ],
    releases: HashMap::from([
      (1, LevelSpawnRelease::RemoveObjects),
      (2, LevelSpawnRelease::ReplaceItems),
    ]),
  });
  let releases: Vec<(u32, Option<LevelSpawnRelease>)> = described
    .objects
    .iter()
    .map(|it: &LevelSpawnObject| (it.index, it.release))
    .collect();

  assert_eq!(
    releases,
    vec![
      (0, None),
      (1, Some(LevelSpawnRelease::RemoveObjects)),
      (2, Some(LevelSpawnRelease::ReplaceItems)),
    ]
  );
}
