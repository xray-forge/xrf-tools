use xrf_editable_level::{EditableLightObject, EditableObjectPart, EditableTextPart};
use xrf_error::XrfResult;
use xrf_level::{
  LevelDynamicLight, LevelDynamicLightsChunk, LevelFile, LevelHeaderChunk, LevelLight, LevelLightsChunk,
  LevelLightsFile,
};
use xrf_math::{Color, Vector3d};

use crate::level::tests::fixtures::GUID;
use crate::{LevelDecompileContext, LevelLightConversion};

/// A static light as the level compiler stores one it was given: diffuse over its energy, the mean of its channels, and
/// range times 1.1 (IX-Ray's `xrLC/Build_Load.cpp:241`).
fn compiled_static(kind: u16, position: [f32; 3], diffuse: [f32; 3], range: f32) -> LevelLight {
  let energy: f32 = (diffuse[0] + diffuse[1] + diffuse[2]) / 3.0;

  LevelLight {
    kind,
    level: 0,
    diffuse: Vector3d::new(diffuse[0] / energy, diffuse[1] / energy, diffuse[2] / energy),
    position: Vector3d::from(position),
    direction: Vector3d::new(0.0, -1.0, 0.0),
    range: range * 1.1,
    range_squared: 0.0,
    falloff: 0.0,
    attenuation_constant: 0.5,
    attenuation_linear: 0.25,
    attenuation_quadratic: 0.125,
    energy,
    triangle: [Vector3d::ZERO; 3],
  }
}

fn compiled_dynamic(position: [f32; 3], diffuse: [f32; 3], range: f32) -> LevelDynamicLight {
  let color = |[r, g, b]: [f32; 3]| Color { r, g, b, a: 0.0 };

  LevelDynamicLight {
    controller: 0,
    kind: LevelDynamicLight::KIND_POINT,
    diffuse: color(diffuse),
    specular: color([0.0; 3]),
    ambient: color([0.0; 3]),
    position: Vector3d::from(position),
    direction: Vector3d::new(0.0, -1.0, 0.0),
    range,
    falloff: 0.0,
    attenuation_0: 1.0,
    attenuation_1: 0.0,
    attenuation_2: 0.0,
    theta: 0.0,
    phi: 0.5,
  }
}

fn decompile() -> EditableObjectPart<EditableLightObject> {
  let statics: LevelLightsFile = LevelLightsFile {
    chunks: vec![
      LevelLightsChunk::Lights {
        id: 0,
        lights: vec![compiled_static(1, [1.0, 2.0, 3.0], [0.75, 0.5, 0.25], 8.0)],
      },
      LevelLightsChunk::Lights {
        id: 1,
        lights: vec![
          compiled_static(1, [-4.0, 1.5, 6.0], [0.25, 0.25, 0.5], 2.5),
          compiled_static(3, [0.0, 100.0, 0.0], [0.5, 0.5, 0.5], 1.0),
        ],
      },
    ],
  };
  let level: LevelFile = LevelFile {
    header: LevelHeaderChunk {
      xrlc_version: 14,
      xrlc_quality: 1,
    },
    shaders: None,
    sectors: None,
    portals: None,
    lights: Some(LevelDynamicLightsChunk {
      lights: vec![
        compiled_dynamic([9.0, 9.0, 9.0], [0.5, 0.25, 0.0], 4.0),
        compiled_dynamic([1.0, 2.0, 3.0], [0.75, 0.5, 0.25], 8.0),
        compiled_dynamic([5.0, 5.0, 5.0], [-0.5, -0.5, -0.25], 3.0),
      ],
    }),
    glows: None,
  };

  LevelLightConversion::decompile(Some(&statics), &level, &mut LevelDecompileContext::new(GUID))
}

#[test]
fn lights_come_back_one_per_authored_light_with_their_layer_and_lighting() -> XrfResult {
  let part: EditableObjectPart<EditableLightObject> = decompile();
  let summary: Vec<(&str, u32, u32, f32, f32)> = part
    .objects
    .iter()
    .map(|light| {
      (
        light.object.name.as_str(),
        light.flags,
        light.control,
        light.brightness,
        light.range,
      )
    })
    .collect();

  assert_eq!(
    summary,
    [
      (
        "light_0000",
        EditableLightObject::AFFECT_STATIC | EditableLightObject::AFFECT_DYNAMIC,
        EditableLightObject::STATIC_CONTROL,
        0.75,
        8.0
      ),
      (
        "light_0001",
        EditableLightObject::AFFECT_STATIC,
        EditableLightObject::HEMI_CONTROL,
        0.5,
        2.5
      ),
      (
        "light_0002",
        EditableLightObject::AFFECT_DYNAMIC,
        EditableLightObject::STATIC_CONTROL,
        0.5,
        4.0
      ),
      (
        "light_0003",
        EditableLightObject::AFFECT_DYNAMIC,
        EditableLightObject::STATIC_CONTROL,
        -0.5,
        3.0
      ),
    ],
    "the generated sky's directional light is not an authored one"
  );
  assert_eq!(
    part.objects[3].color,
    [1.0, 1.0, 0.5, 0.0],
    "a darkening light keeps its sign in its brightness"
  );
  assert_eq!(part.objects[0].color, [1.0, 0.5 / 0.75, 0.25 / 0.75, 0.0]);
  assert_eq!(part.objects[0].cone, 0.5, "a dynamic light's cone");
  assert_eq!(part.objects[1].attenuation, [0.5, 0.25, 0.125]);
  assert_eq!(part.tool_data.get("lcontrols", "$hemi"), Some("1"));

  let read: EditableObjectPart<EditableLightObject> = EditableObjectPart::read_from_text_part(
    &EditableTextPart::read_from_bytes(&part.to_text_part()?.write_to_bytes()?)?,
  )?;

  assert_eq!(read.objects.len(), 4);
  assert_eq!(
    read.to_text_part()?.write_to_bytes()?,
    part.to_text_part()?.write_to_bytes()?,
    "built in code and read back save the same, keys sorting on save"
  );

  Ok(())
}
