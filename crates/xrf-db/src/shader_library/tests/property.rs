//! What one property reads as, and where a field's width is the contract.

use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::shader_library::shader_blender_property::ShaderBlenderProperty;
use crate::shader_library::shader_blender_property_kind::ShaderBlenderPropertyKind;
use crate::shader_library::shader_blender_property_value::ShaderBlenderPropertyValue;
use crate::shader_library::shader_blender_token::ShaderBlenderToken;
use crate::shader_library::tests::fixtures;

fn read(bytes: &[u8]) -> XrfResult<ShaderBlenderProperty> {
  ShaderBlenderProperty::read::<XRayByteOrder, _>(&mut ChunkReader::from_bytes(bytes)?)
}

fn write(property: &ShaderBlenderProperty) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  property.write::<XRayByteOrder>(&mut writer)?;

  writer.flush_raw_into_buffer()
}

#[test]
fn reads_each_type_as_the_value_its_payload_holds() -> XrfResult {
  assert_eq!(
    read(&fixtures::marker("General"))?.value,
    ShaderBlenderPropertyValue::Marker
  );
  assert_eq!(
    read(&fixtures::boolean("Alpha-blend", true))?.value,
    ShaderBlenderPropertyValue::Bool(true)
  );
  assert_eq!(
    read(&fixtures::integer("Alpha ref", 200, 0, 255))?.value,
    ShaderBlenderPropertyValue::Integer {
      value: 200,
      minimum: 0,
      maximum: 255
    }
  );
  assert_eq!(
    read(&fixtures::float("Height", 0.05, 0.0, 1.0))?.value,
    ShaderBlenderPropertyValue::Float {
      value: 0.05,
      minimum: 0.0,
      maximum: 1.0
    }
  );
  assert_eq!(
    read(&fixtures::text(ShaderBlenderPropertyKind::Texture, "Name", "$base0"))?.value,
    ShaderBlenderPropertyValue::Texture(String::from("$base0"))
  );
  assert_eq!(
    read(&fixtures::token("Tessellation", 1, &[(0, "NO_TESS")]))?.value,
    ShaderBlenderPropertyValue::Token {
      selected: 1,
      items: vec![ShaderBlenderToken {
        id: 0,
        label: String::from("NO_TESS")
      }]
    }
  );
  assert_eq!(
    read(&fixtures::class_id("Class", 7, &[9]))?.value,
    ShaderBlenderPropertyValue::ClassId {
      selected: 7,
      items: vec![9]
    }
  );

  Ok(())
}

#[test]
fn keeps_each_type_apart_so_a_writer_can_put_it_back() -> XrfResult {
  // The five name-carrying types share a payload and differ only in their discriminant, which is what a writer has to
  // reproduce; folding them into one string variant would lose it.
  for kind in [
    ShaderBlenderPropertyKind::Matrix,
    ShaderBlenderPropertyKind::Constant,
    ShaderBlenderPropertyKind::Texture,
    ShaderBlenderPropertyKind::Object,
    ShaderBlenderPropertyKind::Text,
  ] {
    let original: Vec<u8> = fixtures::text(kind, "Name", "$null");
    let property: ShaderBlenderProperty = read(&original)?;

    assert_eq!(property.value.kind(), kind);
    assert_eq!(write(&property)?, original);
  }

  Ok(())
}

#[test]
fn reads_a_fixed_field_up_to_its_terminator() -> XrfResult {
  // Whatever an authoring tool left in the rest of the buffer is not part of the value.
  let mut bytes: Vec<u8> = fixtures::raw_property(ShaderBlenderPropertyKind::Texture.raw(), "Name", b"lmap\0");

  bytes.extend_from_slice(&[b'x'; ShaderBlenderPropertyKind::TEXT_SIZE - 5]);

  assert_eq!(
    read(&bytes)?.value,
    ShaderBlenderPropertyValue::Texture(String::from("lmap"))
  );

  Ok(())
}

#[test]
fn refuses_a_fixed_field_that_does_not_terminate() -> XrfResult {
  let bytes: Vec<u8> = fixtures::raw_property(
    ShaderBlenderPropertyKind::Texture.raw(),
    "Name",
    &[b'x'; ShaderBlenderPropertyKind::TEXT_SIZE],
  );

  assert_eq!(
    read(&bytes).unwrap_err().to_string(),
    "Missing terminator error: Shader blender property value is not null terminated"
  );

  Ok(())
}

#[test]
fn refuses_a_value_too_long_for_its_fixed_field() {
  let property: ShaderBlenderProperty = ShaderBlenderProperty {
    name: String::from("Name"),
    value: ShaderBlenderPropertyValue::Texture("a".repeat(ShaderBlenderPropertyKind::TEXT_SIZE)),
  };

  assert!(write(&property).is_err());
}

#[test]
fn refuses_a_token_list_longer_than_its_chunk() -> XrfResult {
  // A file-derived count, so it is bounded by the bytes behind it before anything is reserved.
  let bytes: Vec<u8> = fixtures::raw_property(
    ShaderBlenderPropertyKind::Token.raw(),
    "Tessellation",
    &[0, 0, 0, 0, 0xff, 0xff, 0xff, 0x0f],
  );

  assert!(read(&bytes).is_err());

  Ok(())
}
