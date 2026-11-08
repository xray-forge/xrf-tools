use xrf_chunk::{ChunkPayload, ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_ini::{IniFile, IniFormat, IniSection};

use crate::{EditableLevelGuid, EditablePartHeader};

pub const GUID: EditableLevelGuid = EditableLevelGuid {
  g0: 5065284203709945560,
  g1: 13177767572218497455,
};

/// A scene's main file with every section `EScene::SaveLTX` writes, as the CoP SDK saved `labx8.level`.
pub fn level_text(guid: &EditableLevelGuid) -> XrfResult<String> {
  let mut file: IniFile = IniFile::default();
  let build: &mut IniSection = file.section_mut("build_params");

  for (key, value) in [
    ("light_jitter_samples", "9"),
    ("light_pixel_per_meter", "20.000000"),
    ("light_quality", "2"),
    ("light_quality_reserved", "29795"),
    ("light_rms", "4"),
    ("light_rms_zero", "4"),
    ("reserved_0", "18011438074424197120000000.000000"),
    ("smooth_angle", "75.000000"),
    ("weld_distance", "0.005000"),
  ] {
    build.set(key, value);
  }

  file.section_mut("camera").set("hpb", "-0.410068, -0.471046, 0.000000");
  file
    .section_mut("camera")
    .set("pos", "-80.052984, -6.285084, 75.509568");
  guid.write_to_section(file.section_mut("guid"));

  for (key, value) in [
    ("bop", "\"\""),
    ("game_type", "1"),
    ("level_path", "labx8"),
    ("level_prefix", "labx8"),
    ("light_hemi_quality", "3"),
    ("light_sun_quality", "0"),
    ("map_version", "1.0"),
    ("version", "12"),
    ("version_bp", "9"),
  ] {
    file.section_mut("level_options").set(key, value);
  }

  file.section_mut("level_tag").set("create_time", "1234951740");
  file.section_mut("level_tag").set("owner", "\\\\HOST\\author");
  file.section_mut("version").set("value", "5");

  file.write_to_string_as(&IniFormat::XRAY_SAVE_AS)
}

/// A spawn tool's part with two objects: a restrictor with an attached shape, and a box whose spawn data carries custom
/// data spanning lines.
pub fn spawn_part_text(guid: &EditableLevelGuid, objects_count: u32) -> XrfResult<String> {
  let mut file: IniFile = IniFile::default();

  guid.write_to_section(file.section_mut("guid"));
  EditablePartHeader {
    version: 0,
    objects_count,
    flags: 0,
  }
  .write_to_section(file.section_mut("main"));
  file.section_mut("modif").set("name", "");
  file.section_mut("modif").set("time", "0");

  for (index, name) in [(0, "lx8_restrictor"), (1, "lx8_box")] {
    let section: &mut IniSection = file.section_mut(&format!("object_{index}"));

    section.set("clsid", "6");
    section.set("co_flags", "0");
    section.set("name", name);
    section.set("position", "-50.298076, -20.165922, 67.970768");
    section.set("rotation", "0.000507, -0.010585, -0.022456");
    section.set("scale", "1.000000, 1.000000, 1.000000");
    section.set("version", "23");
  }

  file
    .section_mut("object_0_attached_0")
    .set("shape_radius_0", "1.000000");
  file.section_mut("object_1_spawndata").set("000001", "1");
  file.section_mut("object_1_spawndata").set(
    "000024",
    "\"[logic]\r\ncfg = scripts\\labx8\\box.ltx\r\n\r\n[story_object]\r\nstory_id = box\"",
  );

  file.write_to_string_as(&IniFormat::XRAY_SAVE_AS)
}

/// A binary part as `EScene::SaveLTX` frames it, its chunks in the order given.
pub fn binary_part_bytes(chunks: &[ChunkPayload]) -> XrfResult<Vec<u8>> {
  let mut file: ChunkWriter = ChunkWriter::new();

  for chunk in chunks {
    chunk.write::<XRayByteOrder>(&mut file)?;
  }

  Ok(file.buffer)
}

pub fn chunk(id: u32, bytes: &[u8]) -> ChunkPayload {
  ChunkPayload {
    id,
    bytes: bytes.to_vec(),
  }
}

/// The 16 bytes a binary part stores its GUID as.
pub fn guid_bytes(guid: &EditableLevelGuid) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = Vec::new();

  guid.write::<XRayByteOrder>(&mut bytes)?;

  Ok(bytes)
}
