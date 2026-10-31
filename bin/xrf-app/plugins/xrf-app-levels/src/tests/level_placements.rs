use std::collections::HashMap;

use xrf_level::{
  LevelDynamicLight, LevelEnvModFile, LevelEnvModifier, LevelFogVolFile, LevelFogVolume, LevelGameFile,
  LevelGameRPoint, LevelLightColor, LevelPsStaticFile, LevelPsStaticPlacement, LevelSndEnvFace, LevelSndEnvFile,
  LevelSndStaticFile, LevelSndStaticSound, LevelSndStaticWindow,
};
use xrf_math::{Matrix4x4, Vector3d};
use xrf_renderer::{RenderHelperIcon, RenderHelperObject, RenderHelperShape, RenderLevelHelpers};

use crate::objects::{
  LevelObject, LevelObjectId, LevelObjectKind, LevelObjectShape, LevelObjectSources, LevelObjectsDescription,
  LevelPlacementDetails, LevelPlacementFile,
};
use crate::parts::{LevelPlacements, LevelSpawn};

/// An `Fmatrix` scaling by `scale` and standing at `at`, engine space.
fn new_matrix(scale: f32, at: [f32; 3]) -> Matrix4x4 {
  Matrix4x4 {
    values: [
      scale, 0.0, 0.0, 0.0, 0.0, scale, 0.0, 0.0, 0.0, 0.0, scale, 0.0, at[0], at[1], at[2], 1.0,
    ],
  }
}

fn new_light(kind: u32, at: Vector3d) -> LevelDynamicLight {
  let color = |r: f32| LevelLightColor {
    r,
    g: 0.5,
    b: 0.25,
    a: 1.0,
  };

  LevelDynamicLight {
    controller: 0,
    kind,
    diffuse: color(1.0),
    specular: color(0.2),
    ambient: color(0.0),
    position: at,
    direction: Vector3d::new(0.0, -1.0, 0.0),
    range: 12.0,
    falloff: 0.0,
    attenuation_0: 1.0,
    attenuation_1: 0.0,
    attenuation_2: 0.0,
    theta: 0.0,
    phi: 0.0,
  }
}

/// One of each placement: a system planted for every game and one for multiplayer alone, a sound, a box of reverb
/// 2 by 4 by 6 metres about `(10, 0, 0)`, a fog volume three metres across with an obstacle a metre off its centre, an
/// environment modifier and a respawn point.
fn new_placements() -> LevelPlacements {
  let window = |from: u32, to: u32| LevelSndStaticWindow { from, to };
  let corners: Vec<Vector3d> = (0..8)
    .map(|corner: u32| {
      Vector3d::new(
        10.0 + if corner & 1 == 0 { -1.0 } else { 1.0 },
        if corner & 2 == 0 { -2.0 } else { 2.0 },
        if corner & 4 == 0 { -3.0 } else { 3.0 },
      )
    })
    .collect();

  LevelPlacements {
    particles: Some(LevelPsStaticFile {
      version: 1,
      placements: vec![
        LevelPsStaticPlacement {
          game_types: Some(u16::MAX),
          effect: String::from("static\\smoke"),
          transform: new_matrix(1.0, [1.0, 2.0, 3.0]),
        },
        LevelPsStaticPlacement {
          game_types: Some(2),
          effect: String::from("static\\deathmatch"),
          transform: new_matrix(1.0, [0.0; 3]),
        },
      ],
    }),
    sounds: Some(LevelSndStaticFile {
      sounds: vec![LevelSndStaticSound {
        sound: String::from("ambient\\wind"),
        position: Vector3d::new(4.0, 5.0, 6.0),
        volume: 0.8,
        frequency: 1.0,
        active: window(0, 0),
        play: window(1000, 2000),
        pause: window(0, 0),
      }],
    }),
    sound_environments: Some(LevelSndEnvFile {
      environments: vec![String::from("open"), String::from("room")],
      version: 4,
      aabb_min: Vector3d::new(9.0, -2.0, -3.0),
      aabb_max: Vector3d::new(11.0, 2.0, 3.0),
      vertices: corners,
      faces: (0..12)
        .map(|_| LevelSndEnvFace {
          vertices: [0, 1, 2],
          front: 0,
          back: 1,
        })
        .collect(),
    }),
    fog_volumes: Some(LevelFogVolFile {
      version: 3,
      volumes: vec![LevelFogVolume {
        profile: String::from("environment\\fog\\cave"),
        terminator: String::from("\r\n"),
        transform: new_matrix(3.0, [0.0, 10.0, 0.0]),
        obstacles: vec![new_matrix(1.0, [1.0, 10.0, 0.0])],
      }],
    }),
    env_modifiers: Some(LevelEnvModFile {
      version: 0x0017,
      modifiers: vec![LevelEnvModifier {
        position: Vector3d::new(0.0, 0.0, 20.0),
        radius: 5.0,
        power: 0.5,
        far_plane: 100.0,
        fog_color: Vector3d::new(0.1, 0.2, 0.3),
        fog_density: 0.4,
        ambient: Vector3d::new(0.0, 0.0, 0.0),
        sky_color: Vector3d::new(1.0, 1.0, 1.0),
        hemi_color: Vector3d::new(0.5, 0.5, 0.5),
        use_flags: Some(1 | 4),
      }],
    }),
    game: Some(LevelGameFile {
      rpoints: vec![LevelGameRPoint {
        position: Vector3d::new(7.0, 0.0, 7.0),
        rotation: Vector3d::new(0.0, 0.0, 0.0),
        team: 1,
        kind: 0,
        game_types: u16::MAX,
        profile: String::new(),
      }],
      ways: Vec::new(),
    }),
  }
}

fn describe() -> LevelObjectsDescription {
  let spawn: LevelSpawn = LevelSpawn {
    arrivals: Vec::new(),
    objects: Vec::new(),
    releases: HashMap::new(),
  };
  let lights: Vec<LevelDynamicLight> = vec![
    new_light(LevelDynamicLight::KIND_DIRECTIONAL, Vector3d::new(0.0, 0.0, 0.0)),
    new_light(LevelDynamicLight::KIND_SPOT, Vector3d::new(2.0, 3.0, 4.0)),
  ];

  LevelObjectsDescription::describe(&LevelObjectSources {
    placements: Some(&new_placements()),
    lights: &lights,
    ..LevelObjectSources::of_spawn(&spawn)
  })
}

fn find(described: &LevelObjectsDescription, kind: LevelObjectKind) -> &LevelObject {
  described
    .objects
    .iter()
    .find(|object| object.kind == kind)
    .unwrap_or_else(|| panic!("a {kind:?} listed"))
}

// Each record stands where its file places it in renderer space, a single player game's systems alone, the sun left
// to the environment.
#[test]
fn lists_what_the_companion_files_place_then_the_static_lights() {
  let described: LevelObjectsDescription = describe();
  let listed: Vec<(LevelObjectId, LevelObjectKind, &str)> = described
    .objects
    .iter()
    .map(|it| (it.id, it.kind, it.name.as_str()))
    .collect();
  let placement = |file: LevelPlacementFile| LevelObjectId::Placement { file, index: 0 };

  assert_eq!(
    listed,
    vec![
      (
        placement(LevelPlacementFile::Particles),
        LevelObjectKind::StaticParticles,
        "static\\smoke"
      ),
      (
        placement(LevelPlacementFile::Sounds),
        LevelObjectKind::StaticSounds,
        "ambient\\wind"
      ),
      (
        placement(LevelPlacementFile::SoundEnvironments),
        LevelObjectKind::SoundEnvironments,
        "open / room"
      ),
      (
        placement(LevelPlacementFile::FogVolumes),
        LevelObjectKind::FogVolumes,
        "environment\\fog\\cave"
      ),
      (
        placement(LevelPlacementFile::EnvModifiers),
        LevelObjectKind::EnvModifiers,
        "modifier 0"
      ),
      (
        placement(LevelPlacementFile::RespawnPoints),
        LevelObjectKind::RespawnPoints,
        "actor spawn, team 1"
      ),
      (LevelObjectId::Light { index: 1 }, LevelObjectKind::Lights, "light 1"),
    ]
  );
  assert_eq!(described.objects[0].transform.c, Vector3d::new(1.0, 2.0, -3.0));
  assert_eq!(described.objects[6].transform.c, Vector3d::new(2.0, 3.0, -4.0));
  assert_eq!(
    described.objects[6].light.map(|it| (it.is_spot, it.range, it.color)),
    Some((true, 12.0, [1.0, 0.5, 0.25]))
  );
  assert!(matches!(
    &described.objects[1]
      .placement
      .as_ref()
      .expect("a sound's record")
      .details,
    LevelPlacementDetails::Sound { play: [1000, 2000], .. }
  ));
  assert!(matches!(
    &described.objects[4].placement.as_ref().expect("a modifier's record").details,
    LevelPlacementDetails::EnvModifier { affects, .. } if affects.len() == 2
  ));
}

// A reverb box keeps its three sides whichever corner its file lists first; a fog volume's cube and its obstacle's
// stand about its centre; a modifier is its sphere.
#[test]
fn covers_the_space_each_volume_placement_spans() {
  let described: LevelObjectsDescription = describe();
  let shapes = |kind: LevelObjectKind| -> Vec<LevelObjectShape> {
    find(&described, kind)
      .placement
      .as_ref()
      .map(|it| it.shapes.clone())
      .unwrap_or_default()
  };
  let reverb: Vec<LevelObjectShape> = shapes(LevelObjectKind::SoundEnvironments);
  let [LevelObjectShape::Box { axes, center }] = reverb.as_slice() else {
    panic!("a reverb box is one box");
  };
  let mut sides: Vec<f32> = axes.iter().map(|axis| axis.length()).collect();

  sides.sort_by(f32::total_cmp);

  assert_eq!(sides, vec![2.0, 4.0, 6.0]);
  assert_eq!(*center, Vector3d::new(0.0, 0.0, 0.0));
  assert_eq!(
    find(&described, LevelObjectKind::SoundEnvironments).transform.c,
    Vector3d::new(10.0, 0.0, 0.0)
  );
  assert_eq!(
    shapes(LevelObjectKind::FogVolumes),
    vec![
      LevelObjectShape::Box {
        axes: [
          Vector3d::new(3.0, 0.0, 0.0),
          Vector3d::new(0.0, 3.0, 0.0),
          Vector3d::new(0.0, 0.0, -3.0),
        ],
        center: Vector3d::new(0.0, 0.0, 0.0),
      },
      LevelObjectShape::Box {
        axes: [
          Vector3d::new(1.0, 0.0, 0.0),
          Vector3d::new(0.0, 1.0, 0.0),
          Vector3d::new(0.0, 0.0, -1.0),
        ],
        center: Vector3d::new(1.0, 0.0, 0.0),
      },
    ]
  );
  assert_eq!(
    shapes(LevelObjectKind::EnvModifiers),
    vec![LevelObjectShape::Sphere {
      center: Vector3d::new(0.0, 0.0, 0.0),
      radius: 5.0,
    }]
  );
}

// What stands at a point draws its kind's icon; what spans a space, its shapes.
#[test]
fn draws_a_point_placement_as_its_kinds_icon_and_a_volume_as_its_shapes() {
  let described: LevelObjectsDescription = describe();
  let helpers: RenderLevelHelpers = RenderLevelHelpers::from(&described);
  let shapes = |kind: LevelObjectKind| -> &Vec<RenderHelperShape> {
    &helpers
      .objects
      .iter()
      .find(|object: &&RenderHelperObject| object.kind == kind as u32)
      .unwrap_or_else(|| panic!("a helper for {kind:?}"))
      .shapes
  };
  let icon = |kind: LevelObjectKind| match shapes(kind)[..] {
    [RenderHelperShape::Icon { icon, .. }] => Some(icon),
    _ => None,
  };

  assert_eq!(helpers.objects.len(), described.objects.len());
  assert_eq!(
    icon(LevelObjectKind::StaticParticles),
    Some(RenderHelperIcon::Particles)
  );
  assert_eq!(icon(LevelObjectKind::StaticSounds), Some(RenderHelperIcon::Sound));
  assert_eq!(
    icon(LevelObjectKind::RespawnPoints),
    Some(RenderHelperIcon::RespawnPoint)
  );
  assert_eq!(icon(LevelObjectKind::Lights), Some(RenderHelperIcon::Light));
  assert!(matches!(
    shapes(LevelObjectKind::FogVolumes)[..],
    [RenderHelperShape::Box { .. }, RenderHelperShape::Box { .. }]
  ));
  assert!(matches!(
    shapes(LevelObjectKind::EnvModifiers)[..],
    [RenderHelperShape::Sphere { .. }]
  ));
}
