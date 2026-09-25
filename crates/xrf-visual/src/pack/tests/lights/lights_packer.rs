use std::f32::consts::FRAC_PI_2;
use std::sync::Arc;

use xrf_level::{LevelDynamicLight, LevelLightColor};
use xrf_light_anim::{LightAnimFile, LightAnimItem, LightAnimKey};
use xrf_ltx::Ltx;
use xrf_math::Vector3d;
use xrf_ogf::OgfFile;
use xrf_spawn::{
  AlifeObject, AlifeObjectAbstract, AlifeObjectDynamicVisual, AlifeObjectHangingLamp, AlifeObjectInherited,
  AlifeObjectSkeleton, ClsId,
};

use crate::data::lights::light_description::LightDescription;
use crate::data::lights::light_kind::LightKind;
use crate::data::lights::lights_description::LightsDescription;
use crate::data::visual::skeleton::visual_rest_pose::VisualRestPose;
use crate::pack::lights::lights_packer::LightsPacker;
use crate::pack::tests::visual::fixtures::{MODEL_TYPE_SKELETON_ANIM, bind, bones, vector, visual};

const FLAG_CAST_SHADOW: u16 = 1 << 1;
const FLAG_R2: u16 = 1 << 3;
const FLAG_SPOT: u16 = 1 << 4;
const FLAG_POINT_AMBIENT: u16 = 1 << 5;

fn assert_close(actual: &Vector3d, expected: Vector3d) {
  assert!(
    (actual.x - expected.x).abs() < 1e-5
      && (actual.y - expected.y).abs() < 1e-5
      && (actual.z - expected.z).abs() < 1e-5,
    "expected {expected:?}, got {actual:?}"
  );
}

fn lamp(flags: u16, game_vertex_id: u16) -> AlifeObjectHangingLamp {
  AlifeObjectHangingLamp {
    base: AlifeObjectDynamicVisual {
      base: AlifeObjectAbstract {
        game_vertex_id,
        distance: 0.0,
        direct_control: 1,
        level_vertex_id: 0,
        flags: 0,
        custom_data: String::new(),
        story_id: u32::MAX,
        spawn_story_id: u32::MAX,
      },
      visual_name: String::from("dynamics\\light\\light_lamp"),
      visual_flags: 0,
    },
    skeleton: AlifeObjectSkeleton {
      name: String::from("$editor"),
      flags: 0,
      source_id: u16::MAX,
    },
    main_color: 0xFF80_4020,
    main_brightness: 2.0,
    color_animator: String::new(),
    main_range: 8.0,
    light_flags: flags,
    startup_animation: String::from("$editor"),
    fixed_bones: String::new(),
    health: 1.0,
    virtual_size: 0.1,
    ambient_radius: 3.0,
    ambient_power: 0.25,
    ambient_texture: String::new(),
    light_texture: String::new(),
    light_bone: String::from("bone_lamp"),
    spot_cone_angle: 2.0,
    glow_texture: String::new(),
    glow_radius: 0.7,
    light_ambient_bone: String::from("bone_omni"),
    volumetric_quality: 1.0,
    volumetric_intensity: 1.0,
    volumetric_distance: 1.0,
  }
}

fn object(name: &str, lamp: AlifeObjectHangingLamp) -> AlifeObject {
  AlifeObject {
    id: 0,
    net_action: 1,
    section: String::from("lights_hanging_lamp"),
    clsid: ClsId::SoHLamp,
    name: String::from(name),
    script_game_id: 0,
    script_rp: 0,
    // Turned a quarter about y, which `setXYZ` enters as a heading of pi/2 and turns +z to -x.
    position: vector(10.0, 2.0, 5.0),
    direction: vector(0.0, FRAC_PI_2, 0.0),
    respawn_time: 0,
    parent_id: u16::MAX,
    phantom_id: u16::MAX,
    script_flags: 0,
    version: 128,
    game_type: 0,
    script_version: 12,
    client_data_size: 0,
    spawn_id: 0,
    inherited: AlifeObjectInherited::CseAlifeObjectHangingLamp(Box::new(lamp)),
    update_data: Vec::new(),
  }
}

/// A lamp's visual: its light bone a metre above the root, its ambient bone half a metre below the light.
fn lamp_visual() -> OgfFile {
  OgfFile {
    bones: Some(bones(&[
      ("root", ""),
      ("bone_lamp", "root"),
      ("bone_omni", "bone_lamp"),
    ])),
    ik_data: Some(xrf_ogf::OgfIkDataChunk {
      bones: vec![
        bind(vector(0.0, 0.0, 0.0), vector(0.0, 0.0, 0.0)),
        bind(vector(0.0, 0.0, 0.0), vector(0.0, 1.0, 0.0)),
        bind(vector(0.0, 0.0, 0.0), vector(0.0, -0.5, 0.0)),
      ],
    }),
    ..visual(MODEL_TYPE_SKELETON_ANIM)
  }
}

fn lamp_pose() -> Arc<VisualRestPose> {
  Arc::new(VisualRestPose::of_bind(&lamp_visual()).expect("the lamp's bind pose to resolve"))
}

fn pack_one(object: &AlifeObject, animations: Option<&LightAnimFile>) -> LightsDescription {
  let AlifeObjectInherited::CseAlifeObjectHangingLamp(lamp) = &object.inherited else {
    unreachable!()
  };
  let mut packer: LightsPacker = LightsPacker::new(animations);

  packer.add_lamp(object, lamp, &mut |_| Some(lamp_pose()));
  packer.pack()
}

#[test]
fn places_a_spot_on_its_bone_turned_with_the_object_into_renderer_space() {
  let description: LightsDescription = pack_one(&object("lamp", lamp(FLAG_R2 | FLAG_SPOT | FLAG_CAST_SHADOW, 0)), None);
  let light: &LightDescription = &description.lights[0];

  assert_eq!(description.lights.len(), 1);
  assert_eq!(light.kind, LightKind::Spot);
  // The bone's metre up, at the object's place, with the engine's z mirrored.
  assert_close(&light.position, vector(10.0, 3.0, -5.0));
  // The bone's third axis, turned from +z to -x by the object's heading.
  assert_close(&light.direction, vector(-1.0, 0.0, 0.0));
  assert_close(&light.right, vector(0.0, 0.0, -1.0));
  assert_eq!(
    light.color,
    [128.0 / 255.0 * 2.0, 64.0 / 255.0 * 2.0, 32.0 / 255.0 * 2.0]
  );
  assert_eq!(light.range, 8.0);
  assert_eq!(light.cone, 2.0);
  assert_eq!(light.near, 0.1);
  assert!(light.is_shadowed);
  assert!(!light.is_level);
  // No projector of its own: the engine's default.
  assert_eq!(light.projector, Some(0));
  assert_eq!(description.projectors, vec![String::from("lights\\lights_spot01")]);
}

#[test]
fn lights_nothing_for_a_lamp_the_engine_does_not_spawn_on_r2_one_already_broken_or_a_signal_rocket() {
  let mut broken: AlifeObjectHangingLamp = lamp(FLAG_R2, 0);

  broken.health = 0.0;

  assert!(pack_one(&object("r1 only", lamp(FLAG_SPOT, 0)), None).lights.is_empty());
  assert!(pack_one(&object("broken", broken), None).lights.is_empty());

  let mut rocket: AlifeObject = object("signal", lamp(FLAG_R2, 0));

  rocket.section = String::from("lights_signal_light");

  assert!(pack_one(&rocket, None).lights.is_empty());
}

#[test]
fn adds_the_ambient_point_on_its_own_bone_at_the_ambient_power_unshadowed() {
  let description: LightsDescription = pack_one(
    &object("lamp", lamp(FLAG_R2 | FLAG_CAST_SHADOW | FLAG_POINT_AMBIENT, 0)),
    None,
  );
  let [main, ambient] = description.lights.as_slice() else {
    panic!("expected two lights, got {:?}", description.lights)
  };

  assert_eq!(main.kind, LightKind::Point);
  assert_eq!(main.projector, None);
  assert_eq!(ambient.kind, LightKind::Point);
  assert_close(&ambient.position, vector(10.0, 2.5, -5.0));
  assert_eq!(ambient.range, 3.0);
  assert_eq!(ambient.color, main.color.map(|channel| channel * 0.25));
  assert!(!ambient.is_shadowed);
  assert_eq!(ambient.animator_scale, main.animator_scale * 0.25);
}

#[test]
fn stands_a_lamp_whose_visual_lacks_the_bone_at_the_object_itself() {
  let mut unboned: AlifeObjectHangingLamp = lamp(FLAG_R2, 0);

  unboned.light_bone = String::from("bone_missing");

  let description: LightsDescription = pack_one(&object("lamp", unboned), None);

  assert_close(&description.lights[0].position, vector(10.0, 2.0, -5.0));
}

#[test]
fn lights_each_lamp_of_the_objects_reading_each_visual_once() {
  let objects: Vec<AlifeObject> = vec![object("first", lamp(FLAG_R2, 0)), object("second", lamp(FLAG_R2, 1))];
  let mut packer: LightsPacker = LightsPacker::new(None);
  let mut reads: Vec<String> = Vec::new();

  packer.add_objects(&objects, &mut |name| {
    reads.push(name.to_owned());

    Some(lamp_pose())
  });

  let description: LightsDescription = packer.pack();

  assert_eq!(
    description.lights.iter().map(|it| it.name.as_str()).collect::<Vec<_>>(),
    vec!["first", "second"]
  );
  assert_eq!(reads, vec![String::from("dynamics\\light\\light_lamp")]);
}

#[test]
fn resolves_animators_once_and_takes_a_version_0_library_as_bgr() {
  let item = |name: &str| LightAnimItem {
    name: String::from(name),
    fps: 15.0,
    frame_count: 30,
    keys: vec![
      LightAnimKey {
        frame: 20,
        color: 0x0011_2233,
      },
      LightAnimKey {
        frame: 0,
        color: 0x00AA_BBCC,
      },
    ],
  };
  let animated = |animator: &str| {
    let mut animated: AlifeObjectHangingLamp = lamp(FLAG_R2, 0);

    animated.color_animator = String::from(animator);

    animated
  };

  for (version, expected) in [(0, [0x33_u8, 0x22, 0x11]), (1, [0x11, 0x22, 0x33])] {
    let animations: LightAnimFile = LightAnimFile {
      version,
      items: vec![item("light\\idle")],
    };
    let mut packer: LightsPacker = LightsPacker::new(Some(&animations));

    for (name, animator) in [("a", "light\\idle"), ("b", "light\\idle"), ("c", "light\\missing")] {
      let object: AlifeObject = object(name, animated(animator));
      let AlifeObjectInherited::CseAlifeObjectHangingLamp(lamp) = &object.inherited else {
        unreachable!()
      };

      packer.add_lamp(&object, lamp, &mut |_| Some(lamp_pose()));
    }

    let description: LightsDescription = packer.pack();

    assert_eq!(
      description.lights.iter().map(|it| it.animator).collect::<Vec<_>>(),
      vec![Some(0), Some(0), None]
    );
    assert_eq!(description.animators.len(), 1);
    assert_eq!(
      description.animators[0]
        .keys
        .iter()
        .map(|it| it.frame)
        .collect::<Vec<_>>(),
      vec![0, 20]
    );
    assert_eq!(description.animators[0].keys[1].color, expected.map(f32::from));
    assert_eq!(description.lights[0].animator_scale, 2.0 / 255.0);
  }
}

#[test]
fn keeps_the_level_files_point_lights_and_leaves_its_sun() {
  let light = |kind: u32| LevelDynamicLight {
    controller: 0,
    kind,
    diffuse: LevelLightColor {
      r: 0.5,
      g: 0.25,
      b: 1.0,
      a: 1.0,
    },
    specular: LevelLightColor {
      r: 0.0,
      g: 0.0,
      b: 0.0,
      a: 0.0,
    },
    ambient: LevelLightColor {
      r: 0.0,
      g: 0.0,
      b: 0.0,
      a: 0.0,
    },
    position: vector(1.0, 2.0, 3.0),
    direction: vector(0.0, -1.0, 0.0),
    range: 12.0,
    falloff: 0.0,
    attenuation_0: 1.0,
    attenuation_1: 0.0,
    attenuation_2: 0.0,
    theta: 0.0,
    phi: 0.0,
  };
  let mut packer: LightsPacker = LightsPacker::new(None);

  // A spot record lights as a point too: the loader turns every light that is not the sun into one.
  packer.add_level_lights(&[
    light(LevelDynamicLight::KIND_DIRECTIONAL),
    light(LevelDynamicLight::KIND_SPOT),
  ]);

  let description: LightsDescription = packer.pack();
  let [only] = description.lights.as_slice() else {
    panic!("expected one light, got {:?}", description.lights)
  };

  assert_eq!(only.kind, LightKind::Point);
  assert_close(&only.position, vector(1.0, 2.0, -3.0));
  assert_eq!(only.color, [0.5, 0.25, 1.0]);
  assert_eq!(only.range, 12.0);
  assert!(only.is_shadowed);
  assert!(only.is_level);
}

fn sections() -> Ltx {
  Ltx::read_from_str(
    r"
[campfire]
idle_light = on
idle_light_range = 8
idle_light_anim = koster_00
idle_light_height = 0.7

[zone_quiet]
idle_light = off
idle_light_range = 8
idle_light_anim = koster_00

[zone_unanimated]
idle_light = on
idle_light_range = 3
idle_light_anim = light\missing

[lights_hanging_lamp]
shadow = off
",
  )
  .expect("the sections to parse")
}

fn animations() -> LightAnimFile {
  LightAnimFile {
    version: 1,
    items: vec![LightAnimItem {
      name: String::from("koster_00"),
      fps: 15.0,
      frame_count: 30,
      keys: vec![LightAnimKey {
        frame: 0,
        color: 0x00FF_8000,
      }],
    }],
  }
}

#[test]
fn lights_a_zone_its_section_lights_over_it_by_its_animation_alone() {
  let ltx: Ltx = sections();
  let animations: LightAnimFile = animations();
  let mut packer: LightsPacker = LightsPacker::new(Some(&animations)).with_sections(&ltx);

  for section in ["campfire", "zone_quiet", "zone_unanimated"] {
    let mut zone: AlifeObject = object(section, lamp(FLAG_R2, 0));

    zone.section = String::from(section);
    packer.add_zone(&zone);
  }

  let description: LightsDescription = packer.pack();
  let [fire] = description.lights.as_slice() else {
    panic!("expected the campfire alone, got {:?}", description.lights)
  };

  assert_eq!(fire.name, "campfire");
  assert_eq!(fire.kind, LightKind::Point);
  // Its height over the zone, in renderer space.
  assert_close(&fire.position, vector(10.0, 2.7, -5.0));
  assert_eq!(fire.range, 8.0);
  assert_eq!(fire.range_jitter, 0.25);
  assert_eq!(fire.animator, Some(0));
  assert_eq!(fire.animator_scale, 1.0 / 255.0);
  // `idle_light_shadow` is on unless the section says otherwise.
  assert!(fire.is_shadowed);
}

#[test]
fn takes_a_lamps_own_section_shadow_over_its_flag() {
  let ltx: Ltx = sections();
  let object: AlifeObject = object("lamp", lamp(FLAG_R2 | FLAG_CAST_SHADOW, 0));
  let AlifeObjectInherited::CseAlifeObjectHangingLamp(lamp) = &object.inherited else {
    unreachable!()
  };
  let mut packer: LightsPacker = LightsPacker::new(None).with_sections(&ltx);

  packer.add_lamp(&object, lamp, &mut |_| Some(lamp_pose()));

  assert!(!packer.pack().lights[0].is_shadowed);
}
