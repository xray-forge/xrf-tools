use xrf_editable_level::{EditableGlowObject, EditableObjectPart, EditableTextPart};
use xrf_error::XrfResult;
use xrf_level::{LevelFile, LevelGlow, LevelGlowsChunk, LevelHeaderChunk, LevelShaderEntry, LevelShadersChunk};
use xrf_math::Vector3d;

use crate::level::tests::GUID;
use crate::{LevelDecompileContext, LevelGlowConversion};

/// A compiled level of these shader table rows and glows, and nothing else.
fn level(shaders: &[&str], glows: Vec<LevelGlow>) -> LevelFile {
  LevelFile {
    header: LevelHeaderChunk {
      xrlc_version: 14,
      xrlc_quality: 1,
    },
    shaders: Some(LevelShadersChunk {
      entries: shaders.iter().map(|raw| LevelShaderEntry::parse(raw)).collect(),
    }),
    sectors: None,
    portals: None,
    lights: None,
    glows: Some(LevelGlowsChunk { glows }),
  }
}

fn glow(position: [f32; 3], radius: f32, shader: u16) -> LevelGlow {
  LevelGlow {
    position: Vector3d::from(position),
    radius,
    shader,
  }
}

#[test]
fn glows_come_back_as_the_part_the_level_compiler_was_given() -> XrfResult {
  let level: LevelFile = level(
    &[
      "",
      "def_shaders\\def_vertex/prop\\prop_fence",
      "effects\\glow/glow\\glow_04",
    ],
    vec![
      glow([-78.109_23, -12.153_18, 73.253_07], 1.0, 2),
      glow([10.0, 2.5, -4.0], 0.75, 2),
    ],
  );
  let part: EditableObjectPart<EditableGlowObject> =
    LevelGlowConversion::decompile(&level, &mut LevelDecompileContext::new(GUID))?;
  let read: EditableObjectPart<EditableGlowObject> = EditableObjectPart::read_from_text_part(
    &EditableTextPart::read_from_bytes(&part.to_text_part()?.write_to_bytes()?)?,
  )?;

  assert_eq!(read, part);
  assert_eq!(
    part
      .objects
      .iter()
      .map(|glow| (
        glow.object.name.as_str(),
        glow.shader.as_str(),
        glow.texture.as_str(),
        glow.radius
      ))
      .collect::<Vec<_>>(),
    [
      ("glow_04_0000", "effects\\glow", "glow\\glow_04", 1.0),
      ("glow_04_0001", "effects\\glow", "glow\\glow_04", 0.75),
    ]
  );
  assert_eq!(
    part.objects[1].object.transform.position,
    Vector3d::new(10.0, 2.5, -4.0)
  );
  assert_eq!(part.flags, Some(0));

  Ok(())
}

#[test]
fn refuses_a_glow_whose_shader_row_is_no_shader_with_a_texture() {
  for shader in [0, 3] {
    let error: String = LevelGlowConversion::decompile(
      &level(&["", "effects\\glow/glow\\glow_04"], vec![glow([0.0; 3], 1.0, shader)]),
      &mut LevelDecompileContext::new(GUID),
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains(&format!("shader row {shader}")), "{error}");
  }
}
