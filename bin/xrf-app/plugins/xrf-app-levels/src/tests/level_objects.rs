use std::collections::HashMap;

use xrf_math::Vector3d;
use xrf_spawn::{
  AlifeGraphPoint, AlifeObject, AlifeObjectAbstract, AlifeObjectDynamicVisual, AlifeObjectHangingLamp,
  AlifeObjectInherited, AlifeObjectItem, AlifeObjectPhysic, AlifeObjectSkeleton, AlifeObjectSpaceRestrictor, ClsId,
  Shape,
};

use crate::objects::{
  LEVEL_OBJECT_KINDS, LevelObject, LevelObjectId, LevelObjectKind, LevelObjectKindEntry, LevelObjectShape,
  LevelObjectsDescription, LevelRestriction,
};
use crate::parts::LevelSpawn;
use crate::spawn::LevelSpawnRelease;
use crate::tests::fixtures::{new_abstract, new_object};

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
    update_state: None,
  }))
}

fn new_item(visual: &str) -> AlifeObjectInherited {
  AlifeObjectInherited::CseAlifeItem(Box::new(AlifeObjectItem {
    base: new_visual(visual, AlifeObjectAbstract::INVALID_STORY_ID),
    condition: 1.0,
    upgrades: Vec::new(),
    update_state: None,
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

fn new_restrictor(shape: Vec<Shape>, restrictor_type: u8) -> AlifeObjectInherited {
  AlifeObjectInherited::CseAlifeSpaceRestrictor(Box::new(AlifeObjectSpaceRestrictor {
    base: new_abstract(AlifeObjectAbstract::INVALID_STORY_ID),
    shape,
    restrictor_type,
  }))
}

fn describe(objects: Vec<AlifeObject>) -> LevelObjectsDescription {
  LevelObjectsDescription::describe(&LevelSpawn {
    arrivals: Vec::new(),
    objects,
    releases: HashMap::new(),
  })
}

/// An object as the listing tests read it: its spawn index, name, kind, visual and story id.
type ListedObject<'a> = (u32, &'a str, LevelObjectKind, Option<u32>, Option<u32>);

fn get_index(object: &LevelObject) -> u32 {
  match object.id {
    LevelObjectId::Spawn { index } => index,
  }
}

#[test]
fn lists_every_kind_in_the_order_it_is_declared() {
  for (index, entry) in LEVEL_OBJECT_KINDS.iter().enumerate() {
    assert_eq!(entry.kind as usize, index, "{} stands out of order", entry.label);
    assert_eq!(entry.kind.get_entry(), entry);
  }
}

#[test]
fn reads_each_kind_colour_as_srgb_floats() {
  let entry: &LevelObjectKindEntry = LevelObjectKind::SmartTerrains.get_entry();

  assert_eq!(entry.get_rgb(), [1.0, 152.0 / 255.0, 0.0]);
}

#[test]
fn lists_every_object_in_the_kind_of_its_class_and_names_the_drawn_ones_visuals() {
  let described: LevelObjectsDescription = describe(vec![
    new_object("crate", "physic_object", ClsId::OPhysS, new_physic("physics\\box", 7)),
    new_object("vertex", "graph_point", ClsId::AiGraph, new_graph_point()),
    new_object("medkit", "medkit", ClsId::SFood, new_item("dynamics\\medkit")),
    new_object(
      "lamp",
      "hanging_lamp",
      ClsId::SoHLamp,
      new_lamp(AlifeObjectHangingLamp::FLAG_R2),
    ),
    new_object(
      "fence",
      "space_restrictor",
      ClsId::SpcRsS,
      new_restrictor(Vec::new(), 3),
    ),
  ]);
  let summary: Vec<ListedObject<'_>> = described
    .objects
    .iter()
    .map(|it: &LevelObject| {
      let spawn = it.spawn.as_ref().expect("a spawned object keeps its record");

      (get_index(it), it.name.as_str(), it.kind, spawn.visual, spawn.story_id)
    })
    .collect();

  assert_eq!(
    summary,
    vec![
      (0, "crate", LevelObjectKind::Props, Some(0), Some(7)),
      (2, "medkit", LevelObjectKind::Items, Some(1), None),
      (3, "lamp", LevelObjectKind::Lamps, Some(2), None),
      (4, "fence", LevelObjectKind::Restrictors, None, None),
    ]
  );
  assert_eq!(
    described.visuals,
    vec!["physics\\box", "dynamics\\medkit", "dynamics\\light\\light_lamp"]
  );
}

#[test]
fn names_a_shared_visual_once() {
  let described: LevelObjectsDescription = describe(vec![
    new_object("first", "medkit", ClsId::SFood, new_item("dynamics\\medkit")),
    new_object("second", "medkit", ClsId::SFood, new_item("dynamics\\medkit")),
  ]);

  assert_eq!(described.visuals, vec!["dynamics\\medkit"]);
  assert!(
    described
      .objects
      .iter()
      .all(|it: &LevelObject| it.spawn.as_ref().and_then(|spawn| spawn.visual) == Some(0))
  );
}

// The engine spawns a lamp without the R2 flag on R1 alone, and an object naming no visual draws nothing.
#[test]
fn passes_over_what_the_level_holds_nothing_of() {
  let described: LevelObjectsDescription = describe(vec![
    new_object("lamp", "hanging_lamp", ClsId::SoHLamp, new_lamp(0)),
    new_object("empty", "physic_object", ClsId::OPhysS, new_physic("", 1)),
  ]);

  assert!(described.objects.is_empty());
  assert!(described.visuals.is_empty());
}

// A new game's releases stay in the description, so a panel can say what the level loses and a view can draw them.
#[test]
fn marks_what_a_new_game_releases_and_why() {
  let described: LevelObjectsDescription = LevelObjectsDescription::describe(&LevelSpawn {
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
    .map(|it: &LevelObject| (get_index(it), it.spawn.as_ref().and_then(|spawn| spawn.release)))
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

// The shapes stand in the object's own space, mirrored through `z` into renderer space as its transform is.
#[test]
fn keeps_a_volumes_shapes_in_its_own_renderer_space_and_whom_it_restricts() {
  let described: LevelObjectsDescription = describe(vec![
    new_object(
      "keep_in",
      "space_restrictor",
      ClsId::SpcRsS,
      new_restrictor(vec![Shape::Sphere((Vector3d::new(1.0, 2.0, 3.0), 4.0))], 4),
    ),
    new_object(
      "keep_out",
      "space_restrictor",
      ClsId::SpcRsS,
      new_restrictor(
        vec![Shape::Box((
          Vector3d::new(2.0, 0.0, 0.0),
          Vector3d::new(0.0, 2.0, 0.0),
          Vector3d::new(0.0, 0.0, 2.0),
          Vector3d::new(0.0, 0.0, 5.0),
        ))],
        1,
      ),
    ),
  ]);
  let volumes: Vec<(&str, Vec<LevelObjectShape>, LevelRestriction)> = described
    .objects
    .iter()
    .map(|it: &LevelObject| {
      let volume = it.volume.as_ref().expect("a restrictor is a volume");

      (it.name.as_str(), volume.shapes.clone(), volume.restriction)
    })
    .collect();

  assert_eq!(
    volumes,
    vec![
      (
        "keep_in",
        vec![LevelObjectShape::Sphere {
          center: Vector3d::new(1.0, 2.0, -3.0),
          radius: 4.0
        }],
        LevelRestriction::In
      ),
      (
        "keep_out",
        vec![LevelObjectShape::Box {
          axes: [
            Vector3d::new(2.0, 0.0, 0.0),
            Vector3d::new(0.0, 2.0, 0.0),
            Vector3d::new(0.0, 0.0, -2.0)
          ],
          center: Vector3d::new(0.0, 0.0, -5.0),
        }],
        LevelRestriction::Out
      ),
    ]
  );
}
