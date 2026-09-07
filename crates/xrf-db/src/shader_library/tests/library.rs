//! What reading a library answers, against bytes laid down by hand.

use xrf_chunk::ChunkReader;
use xrf_error::XrfResult;

use crate::shader_library::shader_blender::ShaderBlender;
use crate::shader_library::shader_blender_class::ShaderBlenderClass;
use crate::shader_library::shader_blender_property_value::ShaderBlenderPropertyValue;
use crate::shader_library::shader_library_file::ShaderLibraryFile;
use crate::shader_library::tests::fixtures;

fn read(bytes: &[u8]) -> XrfResult<ShaderLibraryFile> {
  ShaderLibraryFile::read_from_chunk(&mut ChunkReader::from_bytes(bytes)?)
}

#[test]
fn reads_every_blender_of_a_library_by_name() -> XrfResult {
  let library: ShaderLibraryFile = read(&fixtures::library(&[
    fixtures::model_blender("models\\model", false, 32),
    fixtures::model_blender("models\\model_aref", true, 128),
  ]))?;

  assert_eq!(library.blenders_count(), 2);
  assert_eq!(library.blenders().count(), 2);
  assert!(library.contains_blender("models\\model"));
  assert!(!library.contains_blender("models\\missing"));
  assert!(library.find_blender("models\\missing").is_none());

  Ok(())
}

#[test]
fn reads_the_description_of_a_blender_as_the_engine_struct_lays_it_out() -> XrfResult {
  // The one test that would fail if the description size, the class byte order or a fixed field width were wrong in
  // both the reader and the writer.
  let library: ShaderLibraryFile = read(&fixtures::library(&[fixtures::model_blender(
    "models\\model_aref",
    true,
    128,
  )]))?;
  let blender: &ShaderBlender = library.find_blender("models\\model_aref").expect("blender is read");

  assert_eq!(blender.class, ShaderBlenderClass::MODEL);
  assert_eq!(blender.name, "models\\model_aref");
  assert_eq!(blender.computer, fixtures::COMPUTER);
  assert_eq!(blender.time, 0);
  assert_eq!(blender.version, 2);
  assert_eq!(ShaderBlender::DESCRIPTION_SIZE, fixtures::DESCRIPTION_SIZE);

  Ok(())
}

#[test]
fn reads_every_property_of_a_blender_in_the_order_the_class_wrote_them() -> XrfResult {
  let library: ShaderLibraryFile = read(&fixtures::library(&[fixtures::model_blender(
    "models\\model_aref",
    true,
    128,
  )]))?;
  let blender: &ShaderBlender = library.find_blender("models\\model_aref").expect("blender is read");

  assert_eq!(
    blender.properties.iter().map(|it| it.name.as_str()).collect::<Vec<_>>(),
    [
      "General",
      "Priority",
      "Strict sorting",
      "Base Texture",
      "Name",
      "Transform",
      "Use alpha-channel",
      "Alpha ref",
      "Tessellation"
    ]
  );
  assert_eq!(blender.boolean("Use alpha-channel"), Some(true));
  assert_eq!(blender.integer("Alpha ref"), Some(128));
  assert_eq!(blender.integer(ShaderBlender::PRIORITY_PROPERTY), Some(1));
  assert!(!blender.is_strict_sorting());
  assert_eq!(
    blender.find_property("Name").map(|it| &it.value),
    Some(&ShaderBlenderPropertyValue::Texture(String::from("$base0")))
  );

  Ok(())
}

#[test]
fn refuses_a_library_defining_one_name_twice() -> XrfResult {
  // The engine refuses it with `R_ASSERT2`: the second definition is unreachable, and which of the two a surface got
  // would otherwise depend on iteration order here.
  let bytes: Vec<u8> = fixtures::library(&[
    fixtures::model_blender("models\\model", false, 32),
    fixtures::model_blender("models\\model", true, 128),
  ]);

  assert_eq!(
    read(&bytes).unwrap_err().to_string(),
    "Invalid error: Shader library contains duplicate blender 'models\\model'"
  );

  Ok(())
}

#[test]
fn refuses_a_library_with_no_blender_chunk() -> XrfResult {
  // The shader script list, chunk 3, on its own: a file this reader has nothing to fold.
  assert!(read(&fixtures::raw_chunk(3, &[0; 4])).is_err());

  Ok(())
}

#[test]
fn refuses_a_blender_carrying_a_property_type_the_engine_never_writes() -> XrfResult {
  // `xrPID_MARKER_TEMPLATE`, which the engine declares and nothing writes: its payload size has no witness, so the
  // walk cannot advance past it and the library is refused rather than half read.
  let bytes: Vec<u8> = fixtures::library(&[fixtures::blender(
    b"MODEL   ",
    "models\\model",
    2,
    &[
      fixtures::marker("General"),
      fixtures::raw_property(11, "Template", &[]),
      fixtures::boolean("Strict sorting", false),
    ],
  )]);

  assert!(
    read(&bytes).is_err(),
    "expect an unknown property type to fail the library"
  );

  Ok(())
}

#[test]
fn refuses_a_blender_whose_description_is_cut_short() -> XrfResult {
  let mut description: Vec<u8> = fixtures::description(b"MODEL   ", "models\\model", 2);

  description.truncate(fixtures::DESCRIPTION_SIZE - 1);

  assert!(read(&fixtures::library(&[description])).is_err());

  Ok(())
}

#[test]
fn refuses_a_blender_whose_name_field_does_not_terminate() -> XrfResult {
  // A chunk that is not a blender at all reads as one until a field runs to its end, which is where it is refused
  // rather than turned into a name.
  let bytes: Vec<u8> = fixtures::library(&[vec![b'a'; fixtures::DESCRIPTION_SIZE]]);

  assert_eq!(
    read(&bytes).unwrap_err().to_string(),
    "Missing terminator error: Shader blender name is not null terminated"
  );

  Ok(())
}
