//! What writing a blender back produces, and what its knob lookups answer.

use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::fixtures::ShaderBlenderFixture;
use crate::shader_library::shader_blender::ShaderBlender;
use crate::shader_library::shader_blender_property_kind::ShaderBlenderPropertyKind;
use crate::shader_library::tests::fixtures;

fn read(bytes: &[u8]) -> XrfResult<ShaderBlender> {
  ShaderBlender::read::<XRayByteOrder, _>(&mut ChunkReader::from_bytes(bytes)?)
}

fn write(blender: &ShaderBlender) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  blender.write::<XRayByteOrder>(&mut writer)?;

  writer.flush_raw_into_buffer()
}

#[test]
fn writes_a_blender_back_byte_for_byte() -> XrfResult {
  // The writer is judged against the hand laid bytes rather than against its own reading, so a field width the pair
  // agrees on wrongly still fails here.
  let original: Vec<u8> = fixtures::model_blender("models\\model_aref", true, 128);

  assert_eq!(write(&read(&original)?)?, original);

  Ok(())
}

#[test]
fn writes_every_property_type_back_byte_for_byte() -> XrfResult {
  let original: Vec<u8> = fixtures::blender(
    b"LM_AREF ",
    "def_shaders\\def_aref",
    1,
    &[
      fixtures::marker("General"),
      fixtures::integer("Alpha ref", 200, 0, 255),
      fixtures::float("Height", 0.05, 0.0, 1.0),
      fixtures::boolean("Alpha-blend", true),
      fixtures::text(ShaderBlenderPropertyKind::Constant, "Constant", "$null"),
      fixtures::token("Tessellation", 1, &[(0, "NO_TESS"), (3, "TESS_PN+HM")]),
      fixtures::class_id("Class", 7, &[7, 9]),
    ],
  );

  assert_eq!(write(&read(&original)?)?, original);

  Ok(())
}

#[test]
fn reads_the_knobs_a_class_declares_by_name() {
  let blender: ShaderBlender = ShaderBlenderFixture::model("models\\model_aref")
    .with_alpha_channel(true)
    .with_alpha_reference(128)
    .blender;

  assert_eq!(blender.boolean("Use alpha-channel"), Some(true));
  assert_eq!(blender.integer("Alpha ref"), Some(128));
  assert!(!blender.is_strict_sorting());
  // A knob read as the wrong type is absent rather than coerced, so a misread class is visible.
  assert_eq!(blender.integer("Use alpha-channel"), None);
  assert_eq!(blender.boolean("Alpha ref"), None);
  assert_eq!(blender.boolean("Nothing writes this"), None);
}

#[test]
fn refuses_a_name_too_long_for_its_fixed_field() -> XrfResult {
  let mut blender: ShaderBlender = read(&fixtures::model_blender("models\\model", false, 32))?;

  blender.name = "m".repeat(ShaderBlender::NAME_SIZE);

  assert!(
    write(&blender).is_err(),
    "expect a name with no room for its terminator"
  );

  Ok(())
}
