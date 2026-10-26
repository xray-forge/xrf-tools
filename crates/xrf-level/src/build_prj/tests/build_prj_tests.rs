use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::build_prj::build_prj_file::BuildPrjFile;
use crate::build_prj::build_prj_name::BuildPrjName;
use crate::build_prj::build_prj_params::BuildPrjParams;
use crate::build_prj::geometry::build_prj_face::BuildPrjFace;
use crate::build_prj::geometry::build_prj_glow::BuildPrjGlow;
use crate::build_prj::geometry::build_prj_portal::BuildPrjPortal;
use crate::build_prj::lights::build_prj_dynamic_light::BuildPrjDynamicLight;
use crate::build_prj::lights::build_prj_light_control::BuildPrjLightControl;
use crate::build_prj::models::build_prj_lod::BuildPrjLod;
use crate::build_prj::models::build_prj_mu_reference::BuildPrjMuReference;
use crate::build_prj::surfaces::build_prj_material::BuildPrjMaterial;
use crate::build_prj::surfaces::build_prj_shader::BuildPrjShader;
use crate::build_prj::surfaces::build_prj_texture::BuildPrjTexture;
use crate::level::dynamic_lights::level_dynamic_light::LevelDynamicLight;
use crate::tests::fixtures::{chunk, floats, string};

/// A fixed name field as the editor blits it: the name, its terminator, then what a longer name left behind.
fn name_field(value: &str, stale: &[u8], size: usize) -> Vec<u8> {
  let mut bytes: Vec<u8> = string(value);

  bytes.extend_from_slice(stale);
  bytes.resize(size, 0);

  bytes
}

fn words(values: &[u32]) -> Vec<u8> {
  values.iter().flat_map(|value| value.to_le_bytes()).collect()
}

/// One face over the first three vertices, with the pack padding the editor leaves zeroed.
fn face_bytes(material: u16, game_material: u32) -> Vec<u8> {
  let mut bytes: Vec<u8> = words(&[0, 1, 2]);

  bytes.extend(floats(&[0.0, 0.0, 1.0, 0.0, 0.0, 1.0]));
  bytes.extend_from_slice(&material.to_le_bytes());
  bytes.extend_from_slice(&[0, 0]);
  bytes.extend_from_slice(&game_material.to_le_bytes());

  bytes
}

/// One `b_light`: a controller and a 104-byte `Flight`.
fn light_bytes(controller: u32, kind: u32) -> Vec<u8> {
  let mut bytes: Vec<u8> = words(&[controller, kind]);

  bytes.extend(floats(&[1.0, 0.5, 0.25, 1.0]));
  bytes.extend(floats(&[0.0; 8]));
  bytes.extend(floats(&[1.0, 2.0, 3.0, 0.0, -1.0, 0.0]));
  bytes.extend(floats(&[10.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0]));

  bytes
}

/// A project of `version` laid out in `SceneBuilder::SaveBuild`'s order, with one record in every chunk.
fn project_bytes(version: u32) -> XrfResult<Vec<u8>> {
  let has_smoothing_groups: bool = BuildPrjFile::has_smoothing_groups(version);
  let mut bytes: Vec<u8> = chunk(BuildPrjFile::VERSION_CHUNK_ID, &version.to_le_bytes())?;

  let mut params: Vec<u8> = floats(&[75.0, 0.005, 20.0]);

  params.extend(words(&[9, 4, 4]));
  params.extend_from_slice(&[2, 0, 0x7C, 0x7C]);
  // The reserved floats hold whatever the editor's memory did.
  params.extend(floats(&[2.1e37, 5.26e36, 0.0, 0.0, 0.0, 0.0]));
  bytes.extend(chunk(BuildPrjFile::PARAMS_CHUNK_ID, &params)?);

  bytes.extend(chunk(
    BuildPrjFile::VERTICES_CHUNK_ID,
    &floats(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]),
  )?);

  let mut faces: Vec<u8> = face_bytes(0, 5);

  faces.extend(face_bytes(1, 34));
  bytes.extend(chunk(BuildPrjFile::FACES_CHUNK_ID, &faces)?);

  if has_smoothing_groups {
    bytes.extend(chunk(BuildPrjFile::SMOOTHING_GROUPS_CHUNK_ID, &words(&[1, 4]))?);
  }

  let mut material: Vec<u8> = [2u16, 1, 0, 0, 0, 0]
    .iter()
    .flat_map(|value| value.to_le_bytes())
    .collect();

  material.extend(words(&[0]));
  bytes.extend(chunk(BuildPrjFile::MATERIALS_CHUNK_ID, &material)?);
  bytes.extend(chunk(
    BuildPrjFile::RENDER_SHADERS_CHUNK_ID,
    &name_field("default", b"_shaders\\def_vertex", BuildPrjName::LONG_SIZE),
  )?);
  bytes.extend(chunk(
    BuildPrjFile::COMPILER_SHADERS_CHUNK_ID,
    &name_field("default", b"", BuildPrjName::LONG_SIZE),
  )?);

  let mut texture: Vec<u8> = name_field("briks\\briks_big_01", b"", BuildPrjName::LONG_SIZE);

  texture.extend(words(&[512, 512, 0, 0]));
  bytes.extend(chunk(BuildPrjFile::TEXTURES_CHUNK_ID, &texture)?);

  let mut glow: Vec<u8> = floats(&[-3.8, 2.7, 0.06, 0.7]);

  glow.extend(words(&[0, 0]));
  bytes.extend(chunk(BuildPrjFile::GLOWS_CHUNK_ID, &glow)?);

  let mut portal: Vec<u8> = [0u16, 1].iter().flat_map(|value| value.to_le_bytes()).collect();

  portal.extend(floats(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0]));
  portal.extend(floats(&[7.0; 6]));
  portal.extend(words(&[4]));
  bytes.extend(chunk(BuildPrjFile::PORTALS_CHUNK_ID, &portal)?);

  let mut controls: Vec<u8> = Vec::new();

  for name in ["$hemi", "$sun", "$static"] {
    controls.extend(name_field(name, b"ic\xCC\xCC", BuildPrjName::SHORT_SIZE));
    controls.extend(words(&[0]));
  }

  bytes.extend(chunk(BuildPrjFile::LIGHT_CONTROLS_CHUNK_ID, &controls)?);

  let mut static_lights: Vec<u8> = light_bytes(0, 3);

  static_lights.extend(light_bytes(2, 1));
  bytes.extend(chunk(BuildPrjFile::STATIC_LIGHTS_CHUNK_ID, &static_lights)?);

  let mut dynamic_light: Vec<u8> = light_bytes(2, 1);

  // One sector reached, and the fifteen slots past it holding stale memory.
  dynamic_light.extend((0u16..16).flat_map(|slot| (slot * 1_000).to_le_bytes()));
  dynamic_light.extend(words(&[1]));
  bytes.extend(chunk(BuildPrjFile::DYNAMIC_LIGHTS_CHUNK_ID, &dynamic_light)?);

  let mut lod: Vec<u8> = Vec::new();

  for _ in 0..BuildPrjLod::FACES {
    lod.extend(floats(&[0.5; 12]));
    lod.extend(floats(&[0.25; 8]));
  }

  lod.extend(words(&[3]));
  bytes.extend(chunk(BuildPrjFile::LODS_CHUNK_ID, &lod)?);

  let mut model: Vec<u8> = string("tree_01");

  model.extend(words(&[3]));
  model.extend(floats(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]));
  model.extend(words(&[1]));
  model.extend(face_bytes(2, 6));
  model.extend_from_slice(&0u16.to_le_bytes());

  if has_smoothing_groups {
    model.extend(words(&[8]));
  }

  bytes.extend(chunk(BuildPrjFile::MU_MODELS_CHUNK_ID, &model)?);

  let mut reference: Vec<u8> = words(&[0]);

  reference.extend(floats(&[
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 5.0, 0.0, 5.0, 1.0,
  ]));
  reference.extend(words(&[1]));
  reference.extend_from_slice(&[2, 0, 0, 0]);
  reference.extend(words(&[0; 8]));
  bytes.extend(chunk(BuildPrjFile::MU_REFERENCES_CHUNK_ID, &reference)?);

  Ok(bytes)
}

#[test]
fn every_record_is_the_width_the_editor_blits_it_at() {
  assert_eq!(BuildPrjParams::SERIALIZED_SIZE, 52);
  assert_eq!(BuildPrjFace::SERIALIZED_SIZE, 44);
  assert_eq!(BuildPrjMaterial::SERIALIZED_SIZE, 16);
  assert_eq!(BuildPrjShader::SERIALIZED_SIZE, 128);
  assert_eq!(BuildPrjTexture::SERIALIZED_SIZE, 144);
  assert_eq!(BuildPrjGlow::SERIALIZED_SIZE, 24);
  assert_eq!(BuildPrjPortal::SERIALIZED_SIZE, 80);
  assert_eq!(BuildPrjLightControl::HEAD_SIZE, 68);
  assert_eq!(LevelDynamicLight::SERIALIZED_SIZE, 108);
  assert_eq!(BuildPrjDynamicLight::SERIALIZED_SIZE, 144);
  assert_eq!(BuildPrjLod::SERIALIZED_SIZE, 644);
  assert_eq!(BuildPrjMuReference::SERIALIZED_SIZE, 108);
}

#[test]
fn a_project_reads_every_chunk_the_editor_writes() -> XrfResult {
  let file: BuildPrjFile = BuildPrjFile::read_from_bytes::<XRayByteOrder>(project_bytes(18)?)?;

  assert_eq!(file.version, BuildPrjFile::CURRENT_VERSION);
  assert_eq!(file.params.smooth_angle, 75.0);
  assert_eq!(file.params.quality, 2);
  assert_eq!(file.vertices.len(), 3);
  assert_eq!(file.faces.len(), 2);
  assert_eq!(file.faces[1].game_material, 34);
  assert_eq!(file.smoothing_groups, Some(vec![1, 4]));
  assert_eq!(file.materials[0].texture, 2);
  assert_eq!(file.render_shaders[0].name.value, "default");
  assert_eq!(file.textures[0].name.value, "briks\\briks_big_01");
  assert_eq!(file.textures[0].width, 512);
  assert_eq!(file.glows[0].size, 0.7);
  assert_eq!(file.portals[0].get_vertices().len(), 4);
  assert_eq!(
    file
      .light_controls
      .iter()
      .map(|control| control.name.value.as_str())
      .collect::<Vec<&str>>(),
    ["$hemi", "$sun", "$static"]
  );
  assert_eq!(file.static_lights.len(), 2);
  assert_eq!(file.static_lights[1].controller, 2);
  assert_eq!(file.dynamic_lights[0].get_sectors(), [0]);
  assert_eq!(file.lods[0].material, 3);
  assert_eq!(file.mu_models[0].name, "tree_01");
  assert_eq!(file.mu_models[0].smoothing_groups, Some(vec![8]));
  assert_eq!(file.mu_references[0].sector, 2);

  Ok(())
}

#[test]
fn a_project_of_either_version_is_written_back_byte_for_byte() -> XrfResult {
  for version in [BuildPrjFile::SOC_VERSION, BuildPrjFile::CURRENT_VERSION] {
    let bytes: Vec<u8> = project_bytes(version)?;
    let file: BuildPrjFile = BuildPrjFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;
    let mut writer: ChunkWriter = ChunkWriter::new();

    file.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.buffer, bytes, "version {version}");
  }

  Ok(())
}

#[test]
fn a_soc_project_carries_no_smoothing_groups() -> XrfResult {
  let file: BuildPrjFile = BuildPrjFile::read_from_bytes::<XRayByteOrder>(project_bytes(17)?)?;

  assert_eq!(file.smoothing_groups, None);
  assert_eq!(file.mu_models[0].smoothing_groups, None);
  assert_eq!(file.mu_models[0].lod, 0);

  Ok(())
}

#[test]
fn a_name_changed_without_its_trailing_bytes_is_refused_rather_than_misframed() -> XrfResult {
  let mut file: BuildPrjFile = BuildPrjFile::read_from_bytes::<XRayByteOrder>(project_bytes(18)?)?;

  file.render_shaders[0].name.value = String::from("def_shaders\\def_vertex");

  let error: String = file
    .write::<XRayByteOrder>(&mut ChunkWriter::new())
    .expect_err("expect the name to overrun its field")
    .to_string();

  assert!(
    error.contains("do not fill the 128-byte field"),
    "Unexpected error: {error}"
  );

  file.render_shaders[0].name = BuildPrjName::new("def_shaders\\def_vertex", BuildPrjName::LONG_SIZE)?;
  file.write::<XRayByteOrder>(&mut ChunkWriter::new())?;

  Ok(())
}

#[test]
fn a_multi_use_model_disagreeing_with_the_version_is_refused() -> XrfResult {
  let mut file: BuildPrjFile = BuildPrjFile::read_from_bytes::<XRayByteOrder>(project_bytes(18)?)?;

  file.mu_models[0].smoothing_groups = None;

  let error: String = file
    .write::<XRayByteOrder>(&mut ChunkWriter::new())
    .expect_err("expect the missing groups to be refused")
    .to_string();

  assert!(error.contains("lacks smoothing groups"), "Unexpected error: {error}");

  Ok(())
}

#[test]
fn a_project_carrying_a_chunk_no_editor_writes_is_refused() -> XrfResult {
  let mut bytes: Vec<u8> = project_bytes(18)?;

  bytes.extend(chunk(17, &[0; 4])?);

  let error: String = BuildPrjFile::read_from_bytes::<XRayByteOrder>(bytes)
    .expect_err("expect the unknown chunk to be refused")
    .to_string();

  assert!(
    error.contains("chunk 17, which no editor writes"),
    "Unexpected error: {error}"
  );

  Ok(())
}

#[test]
fn a_project_carrying_a_chunk_twice_is_refused_rather_than_rewritten_short() -> XrfResult {
  let mut bytes: Vec<u8> = project_bytes(18)?;

  bytes.extend(chunk(BuildPrjFile::GLOWS_CHUNK_ID, &[])?);

  let error: String = BuildPrjFile::read_from_bytes::<XRayByteOrder>(bytes)
    .expect_err("expect the repeated chunk to be refused")
    .to_string();

  assert!(error.contains("chunk 8 twice"), "Unexpected error: {error}");

  Ok(())
}

#[test]
fn a_project_of_a_version_no_editor_writes_is_refused() -> XrfResult {
  let error: String = BuildPrjFile::read_from_bytes::<XRayByteOrder>(project_bytes(19)?)
    .expect_err("expect the version to be refused")
    .to_string();

  assert!(error.contains("version 19"), "Unexpected error: {error}");

  Ok(())
}
