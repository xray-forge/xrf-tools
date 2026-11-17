//! A Level Editor scene's helpers: one for each object of a helped tool, named by its tool and index both ways,
//! switched by its tool, in its tool's look, and the extent and counts the page frames and lists by, and where each
//! helped object stands for a camera to frame it.

use std::sync::Arc;

use glam::Vec3;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::SessionSnapshot;
use xrf_editable_level::{
  EditableCustomObject, EditableFogVolumeKind, EditableFogVolumeObject, EditableGlowObject, EditableLightObject,
  EditableObjectTransform, EditableParticleObject, EditablePortalObject, EditableSceneObject, EditableSectorObject,
  EditableShape, EditableShapeElement, EditableShapeObject, EditableSoundEnvironmentObject, EditableSoundSourceObject,
  EditableSpawnPointKind, EditableSpawnPointObject, EditableWayLink, EditableWayObject, EditableWayPoint,
};
use xrf_ini::IniSection;
use xrf_math::{Aabb, Sphere, Vector2d, Vector3d};
use xrf_renderer::{RenderHelperIcon, RenderHelperObject, RenderHelperShape, RenderLevelHelpers};
use xrf_world::WorldLevelSource;

use crate::scene::{
  SCENE_HELPER_TOOLS, SceneDrawPart, SceneHelperToolCount, SceneHelpers, SceneHelpersPart, SceneObjectId,
  SceneObjectSphere, SceneRenderId, SceneRenderSource, SceneSource, SceneSurfaceKind, SceneToolKind, SelectedScene,
  ShownScenes,
};
use crate::tests::fixtures::{new_scenes, open_scene, scene_custom_object, scene_part, write_scene, write_scene_part};

/// An entity's spawn point named `name` of `section`, standing at `x` along the scene's first axis, its volume
/// `attached` where it has one.
fn entity(name: &str, section: &str, x: f32, attached: Option<EditableShapeObject>) -> EditableSpawnPointObject {
  EditableSpawnPointObject {
    object: scene_custom_object(name, x),
    kind: EditableSpawnPointKind::Entity {
      section: String::from(section),
      flags: 0,
      data: IniSection::new("spawndata"),
    },
    attached,
  }
}

/// A respawn point and an environment modifier of `radius`, each standing at `x`.
fn spawn_points(x: f32, radius: f32) -> [EditableSpawnPointObject; 2] {
  [
    EditableSpawnPointObject {
      object: scene_custom_object("rpoint_0000", x),
      kind: EditableSpawnPointKind::RespawnPoint {
        team: 0,
        kind: 0,
        game_types: 1,
        profile: String::new(),
      },
      attached: None,
    },
    EditableSpawnPointObject {
      object: scene_custom_object("envmod_0000", x),
      kind: EditableSpawnPointKind::EnvironmentModifier {
        radius,
        power: 1.0,
        view_distance: 300.0,
        fog_color: 0,
        fog_density: 1.0,
        ambient_color: 0,
        sky_color: 0,
        hemi_color: 0,
        flags: EditableSpawnPointObject::DEFAULT_ENVIRONMENT_FLAGS,
      },
      attached: None,
    },
  ]
}

/// A way of three points along the scene's first axis, the first leading to the second and the second to the third.
fn way() -> EditableWayObject {
  let point = |name: &str, x: f32, to: Option<usize>| EditableWayPoint {
    name: String::from(name),
    position: Vector3d::new(x, 0.0, 1.0),
    flags: 0,
    selected: false,
    links: to
      .map(|to| EditableWayLink { to, probability: 1.0 })
      .into_iter()
      .collect(),
  };

  EditableWayObject {
    object: scene_custom_object("walk", 0.0),
    points: vec![
      point("wp00", 0.0, Some(1)),
      point("wp01", 4.0, Some(2)),
      point("wp02", 8.0, None),
    ],
    kind: EditableWayObject::PATROL_PATH,
  }
}

/// A scene holding one or more objects of every helped tool, a scene object and a sector, which draw no helper.
fn write_helped_scene(resource: &str) -> SceneSource {
  let source: SceneSource = write_scene(resource);
  let [respawn, modifier] = spawn_points(30.0, 5.0);

  write_scene_part(
    &source,
    "spawn.part",
    scene_part(vec![
      entity("actor", "actor", 0.0, None),
      entity("stalker_0000", "stalker", 10.0, None),
      entity(
        "restrictor_0000",
        "space_restrictor",
        20.0,
        Some(EditableShapeObject {
          object: scene_custom_object("shape_0001", 20.0),
          shape: EditableShape::new_unit_box(),
        }),
      ),
      respawn,
      modifier,
    ]),
  );
  write_scene_part(&source, "way.part", scene_part(vec![way()]));
  write_scene_part(
    &source,
    "light.part",
    scene_part(vec![EditableLightObject::new(scene_custom_object("light_0000", 1.0))]),
  );
  write_scene_part(
    &source,
    "glow.part",
    scene_part(vec![EditableGlowObject {
      object: scene_custom_object("glow_0000", 2.0),
      radius: 1.5,
      shader: String::from("effects\\glow"),
      texture: String::from("glow\\glow_white"),
      flags: 0,
    }]),
  );
  write_scene_part(
    &source,
    "shape.part",
    scene_part(vec![EditableShapeObject {
      object: scene_custom_object("shape_0000", 3.0),
      shape: EditableShape {
        usage: 0,
        elements: vec![
          EditableShapeElement::Sphere {
            center: Vector3d::ZERO,
            radius: 2.0,
          },
          EditableShape::new_unit_box().elements[0].clone(),
        ],
      },
    }]),
  );
  write_scene_part(
    &source,
    "sound_src.part",
    scene_part(vec![EditableSoundSourceObject {
      object: scene_custom_object("drip_0000", 0.0),
      sound: String::from("ambient\\drip"),
      flags: EditableSoundSourceObject::LOOPED,
      position: Vector3d::new(4.0, 5.0, 6.0),
      volume: 1.0,
      frequency: 1.0,
      min_distance: EditableSoundSourceObject::DEFAULT_MIN_DISTANCE,
      max_distance: EditableSoundSourceObject::DEFAULT_MAX_DISTANCE,
      max_ai_distance: EditableSoundSourceObject::DEFAULT_MAX_AI_DISTANCE,
      random_pause: Vector2d::new(0.0, 0.0),
      active_time: Vector2d::new(0.0, 0.0),
      play_time: Vector2d::new(0.0, 0.0),
    }]),
  );
  write_scene_part(
    &source,
    "sound_env.part",
    scene_part(vec![EditableSoundEnvironmentObject {
      object: scene_custom_object("sound_env_0000", 5.0),
      shape: EditableShape::new_unit_box(),
      inner: String::from("cave"),
      outer: String::from("default"),
    }]),
  );
  write_scene_part(
    &source,
    "ps.part",
    scene_part(vec![EditableParticleObject {
      object: scene_custom_object("steam_0000", 6.0),
      reference: String::from("industrial_particles\\steam"),
      game_types: EditableParticleObject::ALL_GAME_TYPES,
    }]),
  );
  write_scene_part(
    &source,
    "fog_volume.part",
    scene_part(vec![EditableFogVolumeObject {
      object: scene_custom_object("fog_volume_0000", 7.0),
      shape: EditableShape::new_unit_box(),
      kind: EditableFogVolumeKind::Emitter {
        profile: String::from(EditableFogVolumeObject::DEFAULT_PROFILE),
      },
      group: 1,
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
        Vector3d::new(0.0, 0.0, 0.0),
        Vector3d::new(2.0, 0.0, 0.0),
        Vector3d::new(2.0, 2.0, 0.0),
        Vector3d::new(0.0, 2.0, 0.0),
      ],
    }]),
  );
  write_scene_part(
    &source,
    "scene_object.part",
    scene_part(vec![EditableSceneObject {
      object: scene_custom_object("table_0000", 0.0),
      reference: String::from("statics\\table"),
      flags: 0,
    }]),
  );
  write_scene_part(
    &source,
    "sector.part",
    scene_part(vec![EditableSectorObject {
      object: scene_custom_object("sector_0000", 0.0),
      color: [0, 255, 0, 255],
      is_default: true,
      items: Vec::new(),
      map_index: EditableSectorObject::NO_MAP,
    }]),
  );

  source
}

fn open(source: &SceneSource) -> (AssetMountState, Arc<SessionSnapshot<SelectedScene>>) {
  let assets: AssetMountState = AssetMountState::new();
  let opened: Arc<SessionSnapshot<SelectedScene>> = open_scene(&new_scenes(&assets), &assets, source);

  (assets, opened)
}

/// The open scene's helpers, its spawn points' models looked for in its game data.
fn read_helpers(assets: &AssetMountState, opened: &SessionSnapshot<SelectedScene>) -> Arc<SceneHelpers> {
  SceneHelpersPart::get(opened, &assets.search(&opened.game.roots).unwrap())
}

/// The helpers of `tool`'s objects, in their order.
fn list_tool(helpers: &RenderLevelHelpers, tool: SceneToolKind) -> Vec<&RenderHelperObject> {
  helpers
    .objects
    .iter()
    .filter(|object| SceneRenderId(object.id).get_object().is_some_and(|id| id.tool == tool))
    .collect()
}

/// The icons a helper draws.
fn list_icons(object: &RenderHelperObject) -> Vec<RenderHelperIcon> {
  object
    .shapes
    .iter()
    .filter_map(|shape| match shape {
      RenderHelperShape::Icon { icon, .. } => Some(*icon),
      _ => None,
    })
    .collect()
}

#[test]
fn names_an_object_to_the_renderer_by_its_tool_and_index_both_ways() {
  let way: SceneObjectId = SceneObjectId {
    tool: SceneToolKind::Way,
    index: 5,
  };

  // The page decodes the same constant (`scene-render-id.test.ts`): the way's place, 7, above 24 bits of index.
  assert_eq!(SceneRenderId::of(&way), Some(SceneRenderId(0x0700_0005)));
  assert_eq!(SceneRenderId(0x0700_0005).get_object(), Some(way));
  // A scene object's model carries its tool's place, 2, as a helper does.
  assert_eq!(
    SceneRenderId::of(&SceneObjectId {
      tool: SceneToolKind::SceneObject,
      index: 3,
    }),
    Some(SceneRenderId(0x0200_0003))
  );
  // Its other models set a bit above the tools' places each, and name the object as its own model does.
  assert_eq!(
    SceneRenderId(0x0200_0003).get_part(SceneSurfaceKind::Collision),
    SceneRenderId(0x8200_0003)
  );
  assert_eq!(
    SceneRenderId(0x0200_0003).get_part(SceneSurfaceKind::EditorOnly),
    SceneRenderId(0x4200_0003)
  );

  for id in [0x8200_0003, 0x4200_0003] {
    assert_eq!(SceneRenderId(id).get_object(), SceneRenderId(0x0200_0003).get_object());
  }
  assert_eq!(
    SceneRenderId::of(&SceneObjectId {
      tool: SceneToolKind::Way,
      index: SceneRenderId::MAX_INDEX + 1,
    }),
    None,
    "an index past what an id holds is not drawn"
  );
  assert_eq!(
    SceneRenderId(16 << 24).get_object(),
    None,
    "no tool stands past the last"
  );

  for tool in SceneToolKind::ALL {
    let object: SceneObjectId = SceneObjectId {
      tool,
      index: SceneRenderId::MAX_INDEX,
    };

    assert_eq!(
      SceneRenderId::of(&object).and_then(SceneRenderId::get_object),
      Some(object)
    );
  }
}

#[test]
fn switches_each_helped_tool_by_its_place_among_the_tools() {
  for entry in SCENE_HELPER_TOOLS {
    let place: usize = SceneToolKind::ALL
      .iter()
      .position(|tool| *tool == entry.tool)
      .expect("a listed tool");

    assert_eq!(entry.kind as usize, place, "{:?}", entry.tool);
    assert!(entry.kind < u32::BITS, "a view switches at most 32 kinds");
    assert!(
      entry.get_rgb().iter().any(|channel| *channel > 0.0),
      "{} reads",
      entry.color
    );
  }

  assert!(
    SCENE_HELPER_TOOLS.windows(2).all(|pair| pair[0].kind < pair[1].kind),
    "in the editor's order of its tools"
  );
}

#[test]
fn helps_each_object_of_a_helped_tool_in_its_tools_look() {
  let (assets, opened) = open(&write_helped_scene("scene_helpers/tools"));
  let helpers: Arc<SceneHelpers> = read_helpers(&assets, &opened);

  assert_eq!(
    helpers.description.tools,
    [
      (SceneToolKind::Glow, 1),
      (SceneToolKind::Light, 1),
      (SceneToolKind::Shape, 1),
      (SceneToolKind::SoundSource, 1),
      (SceneToolKind::Spawn, 5),
      (SceneToolKind::Way, 1),
      (SceneToolKind::Portal, 1),
      (SceneToolKind::SoundEnvironment, 1),
      (SceneToolKind::Particle, 1),
      (SceneToolKind::FogVolume, 1),
    ]
    .map(|(tool, count)| SceneHelperToolCount { tool, count }),
    "the scene object and the sector draw none"
  );

  for object in &helpers.helpers.objects {
    let id: SceneObjectId = SceneRenderId(object.id).get_object().expect("a scene object's id");
    let entry = SCENE_HELPER_TOOLS
      .iter()
      .find(|entry| entry.tool == id.tool)
      .expect("a helped tool");

    assert_eq!(object.kind, entry.kind);
    assert_eq!(object.color, entry.get_rgb());
    assert_eq!(object.draw_distance, entry.draw_distance);
    assert!(!object.shapes.is_empty());
  }

  let spawns: Vec<&RenderHelperObject> = list_tool(&helpers.helpers, SceneToolKind::Spawn);

  assert_eq!(
    spawns
      .iter()
      .map(|object| SceneRenderId(object.id).get_object().map(|id| id.index))
      .collect::<Vec<_>>(),
    [0, 1, 2, 3, 4].map(Some),
    "each by its place among the tool's objects"
  );
  assert_eq!(list_icons(spawns[0]), [RenderHelperIcon::ActorStart]);
  assert_eq!(list_icons(spawns[1]), [RenderHelperIcon::SpawnPoint]);
  assert!(
    matches!(spawns[2].shapes.as_slice(), [RenderHelperShape::Box { .. }]),
    "a restrictor by the volume attached to it"
  );
  assert_eq!(list_icons(spawns[3]), [RenderHelperIcon::RespawnPoint]);
  assert_eq!(
    spawns[4].shapes,
    [RenderHelperShape::Sphere {
      center: [0.0; 3],
      radius: 5.0
    }],
    "an environment modifier by the sphere it reaches"
  );

  let way: &RenderHelperObject = list_tool(&helpers.helpers, SceneToolKind::Way)[0];

  assert!(
    matches!(&way.shapes[0], RenderHelperShape::Segments { ends } if ends.len() == 6),
    "two links, each a line and two barbs"
  );
  assert_eq!(list_icons(way), [RenderHelperIcon::PatrolPoint; 3], "a mark a point");
  assert!(
    matches!(
      list_tool(&helpers.helpers, SceneToolKind::Portal)[0].shapes.as_slice(),
      [RenderHelperShape::Segments { ends }] if ends.len() == 4
    ),
    "a portal by its closed outline"
  );
  assert_eq!(
    list_icons(list_tool(&helpers.helpers, SceneToolKind::SoundSource)[0]),
    [RenderHelperIcon::Sound],
    "a sound source by its icon alone, its saved distances being no measure"
  );
  assert_eq!(
    list_tool(&helpers.helpers, SceneToolKind::Glow)[0].shapes,
    [RenderHelperShape::Sphere {
      center: [0.0; 3],
      radius: 1.5
    }]
  );
  assert_eq!(
    list_tool(&helpers.helpers, SceneToolKind::Shape)[0].shapes.len(),
    2,
    "a shape by each of its elements"
  );
}

/// Asserts `sphere` stands about `center` with `radius`, to a float's rounding.
fn assert_sphere(sphere: Option<Sphere>, center: [f32; 3], radius: f32) {
  let sphere: Sphere = sphere.expect("a sphere");

  assert!(
    (Vec3::new(sphere.center.x, sphere.center.y, sphere.center.z) - Vec3::from_array(center)).length() < 1e-5
      && (sphere.radius - radius).abs() < 1e-5,
    "{sphere:?} against {center:?} of {radius}"
  );
}

#[test]
fn locates_a_helped_object_by_its_helpers_extent_in_renderer_space() {
  let (assets, opened) = open(&write_helped_scene("scene_helpers/locate"));
  let locate = |tool: SceneToolKind, index: u32| {
    SceneObjectSphere::locate(&opened, &assets, &SceneObjectId { tool, index }).expect("the scene is searched")
  };

  // A light by its icon at `1, 2, 3`, `z` mirrored: a point.
  assert_sphere(locate(SceneToolKind::Light, 0), [1.0, 2.0, -3.0], 0.0);
  // A glow by its sphere of 1.5 about `2, 2, 3`, bounded by the sphere's box.
  assert_sphere(locate(SceneToolKind::Glow, 0), [2.0, 2.0, -3.0], 1.5 * 3.0_f32.sqrt());
  // A fog volume by the unit box at `7, 2, 3`.
  assert_sphere(locate(SceneToolKind::FogVolume, 0), [7.0, 2.0, -3.0], 0.75_f32.sqrt());
  // A portal by its outline, the square of 2 standing on the scene's first two axes.
  assert_sphere(locate(SceneToolKind::Portal, 0), [1.0, 1.0, 0.0], 2.0_f32.sqrt());
  assert!(
    locate(SceneToolKind::Way, 0).is_some_and(|sphere| sphere.radius >= 4.0),
    "a way by its links and points, eight metres along"
  );
  assert_eq!(locate(SceneToolKind::Light, 1), None, "no second light");
}

#[test]
fn locates_nothing_for_what_draws_nothing() {
  let (assets, opened) = open(&write_helped_scene("scene_helpers/locate_undrawn"));
  let locate = |tool: SceneToolKind| {
    SceneObjectSphere::locate(&opened, &assets, &SceneObjectId { tool, index: 0 }).expect("the scene is searched")
  };

  assert_eq!(locate(SceneToolKind::Sector), None, "a sector draws no helper");
  assert_eq!(locate(SceneToolKind::Group), None, "a group draws nothing of its own");
  assert_eq!(
    locate(SceneToolKind::SceneObject),
    None,
    "a scene object whose library object is not found is not drawn"
  );
}

#[test]
fn the_renderer_reads_the_helpers_the_page_counts() {
  let source: SceneSource = write_helped_scene("scene_helpers/render");
  let (assets, opened) = open(&source);
  let drawn: Arc<SceneRenderSource> = ShownScenes::default().get_source(Arc::clone(&opened), &assets);

  assert_eq!(
    drawn.read_helpers().expect("the helpers read"),
    read_helpers(&assets, &opened).helpers
  );
}

#[test]
fn a_scene_of_helpers_alone_is_bounded_by_them_where_it_draws_no_object() {
  let source: SceneSource = write_scene("scene_helpers/alone");
  let [_, modifier] = spawn_points(10.0, 2.0);

  write_scene_part(&source, "spawn.part", scene_part(vec![modifier]));
  write_scene_part(
    &source,
    "light.part",
    scene_part(vec![EditableLightObject::new(EditableCustomObject::new(
      "light_0000",
      EditableObjectTransform {
        position: Vector3d::new(-4.0, 1.0, 2.0),
        ..EditableObjectTransform::default()
      },
    ))]),
  );

  let (assets, opened) = open(&source);
  let helpers: Arc<SceneHelpers> = read_helpers(&assets, &opened);

  assert!(
    SceneDrawPart::get(&opened, &assets.search(&opened.objects_roots).unwrap())
      .description
      .bounds
      .is_none(),
    "no scene object is drawn"
  );
  // The light's icon at `-4, 1, 2` and the modifier's sphere of 2 about `10, 2, 3`, `z` mirrored.
  assert_eq!(
    helpers.description.bounds.as_ref().map(|bounds| bounds.bounding_box),
    Some(Aabb {
      min: Vector3d::new(-4.0, 0.0, -5.0),
      max: Vector3d::new(12.0, 4.0, -1.0),
    })
  );
}

#[test]
fn a_scene_of_no_helped_object_has_no_helper_and_no_extent() {
  let (assets, opened) = open(&write_scene("scene_helpers/empty"));
  let helpers: Arc<SceneHelpers> = read_helpers(&assets, &opened);

  assert!(helpers.helpers.objects.is_empty());
  assert!(helpers.description.tools.is_empty());
  assert!(helpers.description.bounds.is_none());
}
