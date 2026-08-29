use std::io::Write;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};
use xrf_utils::{assert, assert_equal, assert_not_equal, decode_bytes_from_base64, encode_bytes_to_base64};

use crate::constants::{FLAG_SPAWN_DESTROY_ON_SPAWN, MINIMAL_SUPPORTED_SPAWN_VERSION, NET_ACTION_SPAWN};
use crate::data::alife::alife_object_inherited::AlifeObjectInherited;
use crate::data::generic::vector_3d::Vector3d;
use crate::data::meta::alife_class::AlifeClass;
use crate::data::meta::cls_id::ClsId;
use crate::export::LtxImportExport;
use crate::file_import::read_ltx_field;

/// Generic abstract ALife object base.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObject {
  pub id: u16,
  pub net_action: u16,
  pub section: String,
  pub clsid: ClsId,
  pub name: String,
  pub script_game_id: u8,
  pub script_rp: u8,
  pub position: Vector3d,
  pub direction: Vector3d,
  pub respawn_time: u16,
  pub parent_id: u16,
  pub phantom_id: u16,
  pub script_flags: u16,
  pub version: u16,
  pub game_type: u16,
  pub script_version: u16,
  pub client_data_size: u16,
  pub spawn_id: u16,
  pub inherited: AlifeObjectInherited,
  pub update_data: Vec<u8>, // todo: Parse.
}

impl AlifeObject {
  pub const DATA_SPAWN_CHUNK_ID: u32 = 0;
  pub const DATA_UPDATE_CHUNK_ID: u32 = 1;
}

impl AlifeObject {
  /// One framed object chunk: its own header, plus the header and payload of the index chunk it must carry.
  pub const MIN_SERIALIZED_SIZE: u64 = 8 + 8 + 2;
}

impl ChunkReadWrite for AlifeObject {
  /// Read generic ALife object data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut spawn_reader: ChunkReader<D> = reader.read_child_by_index(Self::DATA_SPAWN_CHUNK_ID)?;

    let data_length: u16 = spawn_reader.read_u16::<T>()?;

    assert_equal(
      data_length as u64 + 2,
      spawn_reader.size,
      "Expect correct data size for ALife object",
    )?;

    let net_action: u16 = spawn_reader.read_u16::<T>()?;

    assert_equal(
      net_action,
      NET_ACTION_SPAWN,
      "Expect only net_spawn actions in ALife object data",
    )?;

    let section: String = spawn_reader.read_w1251_string()?;
    let clsid: ClsId = ClsId::from_section(&section)?;
    let class: AlifeClass = AlifeClass::from_cls_id(&clsid);
    let name: String = spawn_reader.read_w1251_string()?;
    let script_game_id: u8 = spawn_reader.read_u8()?;
    let script_rp: u8 = spawn_reader.read_u8()?;
    let position: Vector3d = spawn_reader.read_xr::<T, _>()?;
    let direction: Vector3d = spawn_reader.read_xr::<T, _>()?;
    let respawn_time: u16 = spawn_reader.read_u16::<T>()?;
    let id: u16 = spawn_reader.read_u16::<T>()?;
    let parent_id: u16 = spawn_reader.read_u16::<T>()?;
    let phantom_id: u16 = spawn_reader.read_u16::<T>()?;
    let script_flags: u16 = spawn_reader.read_u16::<T>()?;
    let version: u16 = if script_flags & FLAG_SPAWN_DESTROY_ON_SPAWN == 0 {
      0
    } else {
      spawn_reader.read_u16::<T>()?
    };

    assert(
      version > MINIMAL_SUPPORTED_SPAWN_VERSION,
      "Unexpected version of ALife object in spawn file, flag is {script_flags}",
    )?;

    let game_type: u16 = spawn_reader.read_u16::<T>()?;
    let script_version: u16 = spawn_reader.read_u16::<T>()?;
    let client_data_size: u16 = spawn_reader.read_u16::<T>()?;

    assert_equal(client_data_size, 0, "Client data is not expected in ALife object")?; // Or read client data?

    let spawn_id: u16 = spawn_reader.read_u16::<T>()?;
    let inherited_size: u16 = spawn_reader.read_u16::<T>()?;

    assert_equal(
      inherited_size as u64 - 2,
      spawn_reader.end_pos() - spawn_reader.cursor_pos(),
      "Expect correct size of inherited data for ALife object",
    )?;

    assert_not_equal(&class, &AlifeClass::Unknown, "Expect known ALife object clsid")?;

    let inherited: AlifeObjectInherited = AlifeObjectInherited::read::<T, _>(&mut spawn_reader, &class)?;

    let mut update_reader: ChunkReader<D> = reader.read_child_by_index(Self::DATA_UPDATE_CHUNK_ID)?;
    let update_data_length: u16 = update_reader.read_u16::<T>()?;
    let update_size: u16 = update_reader.read_u16::<T>()?;

    assert_equal(
      update_data_length as u64 + 2,
      update_reader.size,
      "Expect correct size of ALife object update data",
    )?;
    assert_equal(update_size, 0, "Expect no ALife update data in object chunk")?;

    let update_data: Vec<u8> = update_reader.read_bytes(update_reader.read_bytes_remain() as usize)?;

    spawn_reader.assert_read("Expect ALife object spawn data to be read")?;
    update_reader.assert_read("Expect ALife object update data to be read")?;
    reader.assert_read("Expect ALife object chunk to be read")?;

    Ok(Self {
      net_action,
      section,
      clsid,
      name,
      script_game_id,
      script_rp,
      position,
      direction,
      respawn_time,
      id,
      parent_id,
      phantom_id,
      script_flags,
      version,
      game_type,
      script_version,
      client_data_size,
      spawn_id,
      inherited,
      update_data,
    })
  }

  /// Write ALife object data into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut data_spawn_writer: ChunkWriter = ChunkWriter::new();
    let mut data_update_writer: ChunkWriter = ChunkWriter::new();

    let mut object_data_writer: ChunkWriter = ChunkWriter::new();
    let mut inherited_data_writer: ChunkWriter = ChunkWriter::new();
    let mut updated_data_writer: ChunkWriter = ChunkWriter::new();

    object_data_writer.write_u16::<T>(self.net_action)?;

    object_data_writer.write_w1251_string(&self.section)?;
    object_data_writer.write_w1251_string(&self.name)?;
    object_data_writer.write_u8(self.script_game_id)?;
    object_data_writer.write_u8(self.script_rp)?;

    object_data_writer.write_xr::<T, Vector3d>(&self.position)?;
    object_data_writer.write_xr::<T, Vector3d>(&self.direction)?;

    object_data_writer.write_u16::<T>(self.respawn_time)?;
    object_data_writer.write_u16::<T>(self.id)?;
    object_data_writer.write_u16::<T>(self.parent_id)?;
    object_data_writer.write_u16::<T>(self.phantom_id)?;
    object_data_writer.write_u16::<T>(self.script_flags)?;
    object_data_writer.write_u16::<T>(self.version)?;
    object_data_writer.write_u16::<T>(self.game_type)?;
    object_data_writer.write_u16::<T>(self.script_version)?;
    object_data_writer.write_u16::<T>(self.client_data_size)?;
    object_data_writer.write_u16::<T>(self.spawn_id)?;

    self.inherited.write::<T>(&mut inherited_data_writer)?;

    object_data_writer.write_u16::<T>(inherited_data_writer.bytes_written() as u16 + 2)?;
    object_data_writer.write_all(inherited_data_writer.flush_raw_into_buffer()?.as_slice())?;

    data_spawn_writer.write_u16::<T>(object_data_writer.bytes_written() as u16)?;
    data_spawn_writer.write_all(object_data_writer.flush_raw_into_buffer()?.as_slice())?;

    updated_data_writer.write_u16::<T>(0)?;
    updated_data_writer.write_all(&self.update_data)?;

    data_update_writer.write_u16::<T>(updated_data_writer.bytes_written() as u16)?;
    data_update_writer.write_all(updated_data_writer.flush_raw_into_buffer()?.as_slice())?;

    writer.write_all(
      data_spawn_writer
        .flush_chunk_into_buffer::<T>(Self::DATA_SPAWN_CHUNK_ID)?
        .as_slice(),
    )?;
    writer.write_all(
      data_update_writer
        .flush_chunk_into_buffer::<T>(Self::DATA_UPDATE_CHUNK_ID)?
        .as_slice(),
    )?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObject {
  /// Import ALife object data from ltx file section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object base '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    let object_section: String = read_ltx_field("section", section)?;
    let clsid: ClsId = ClsId::from_section(&object_section)?;

    Ok(Self {
      id: read_ltx_field("id", section)?,
      net_action: read_ltx_field("net_action", section)?,
      clsid: clsid.clone(),
      section: object_section,
      name: read_ltx_field("name", section)?,
      script_game_id: read_ltx_field("script_game_id", section)?,
      script_rp: read_ltx_field("script_rp", section)?,
      position: read_ltx_field("position", section)?,
      direction: read_ltx_field("direction", section)?,
      respawn_time: read_ltx_field("respawn_time", section)?,
      parent_id: read_ltx_field("parent_id", section)?,
      phantom_id: read_ltx_field("phantom_id", section)?,
      script_flags: read_ltx_field("script_flags", section)?,
      version: read_ltx_field("version", section)?,
      game_type: read_ltx_field("game_type", section)?,
      script_version: read_ltx_field("script_version", section)?,
      client_data_size: read_ltx_field("client_data_size", section)?,
      spawn_id: read_ltx_field("spawn_id", section)?,
      inherited: AlifeObjectInherited::import(section_name, ltx, &AlifeClass::from_cls_id(&clsid))?,
      update_data: decode_bytes_from_base64(&read_ltx_field::<String>("update_data", section)?)?,
    })
  }

  /// Export ALife object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx
      .with_section(section_name)
      .set("id", self.id.to_string())
      .set("net_action", self.net_action.to_string())
      .set("section", &self.section)
      .set("name", &self.name)
      .set("script_game_id", self.script_game_id.to_string())
      .set("script_rp", self.script_rp.to_string())
      .set("position", self.position.to_string())
      .set("direction", self.direction.to_string())
      .set("respawn_time", self.respawn_time.to_string())
      .set("parent_id", self.parent_id.to_string())
      .set("phantom_id", self.phantom_id.to_string())
      .set("script_flags", self.script_flags.to_string())
      .set("version", self.version.to_string())
      .set("game_type", self.game_type.to_string())
      .set("script_version", self.script_version.to_string())
      .set("client_data_size", self.client_data_size.to_string())
      .set("spawn_id", self.spawn_id.to_string());

    self.inherited.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("update_data", encode_bytes_to_base64(&self.update_data));

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::{self, File};
  use std::io::{Seek, SeekFrom, Write};
  use std::path::PathBuf;

  use byteorder::WriteBytesExt;
  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::{XrfError, XrfResult};
  use xrf_ltx::Ltx;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_resource_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::alife_object::AlifeObject;
  use crate::data::alife::alife_object_inherited::AlifeObjectInherited;
  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;
  use crate::data::alife::inherited::alife_object_item_custom_outfit::AlifeObjectItemCustomOutfit;
  use crate::data::generic::vector_3d::Vector3d;
  use crate::data::meta::cls_id::ClsId;
  use crate::export::LtxImportExport;

  #[test]
  fn binary_read_returns_error_for_unknown_section() -> XrfResult {
    let mut object_data_writer: ChunkWriter = ChunkWriter::new();
    object_data_writer.write_u16::<XRayByteOrder>(1)?;
    object_data_writer.write_w1251_string("unknown_section")?;

    let mut spawn_data_writer: ChunkWriter = ChunkWriter::new();
    spawn_data_writer.write_u16::<XRayByteOrder>(object_data_writer.bytes_written() as u16)?;
    spawn_data_writer.write_all(&object_data_writer.flush_raw_into_buffer()?)?;

    let bytes: Vec<u8> = spawn_data_writer.flush_chunk_into_buffer::<XRayByteOrder>(0)?;
    let path: PathBuf = std::env::temp_dir().join(format!("xrf-db-unknown-clsid-{}.chunk", std::process::id()));
    fs::write(&path, bytes)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_file(File::open(&path)?)?;
    let result: XrfResult<AlifeObject> = AlifeObject::read::<XRayByteOrder, _>(&mut reader);

    drop(reader);
    fs::remove_file(path)?;

    let error: XrfError = result.unwrap_err();

    assert_eq!(
      error.to_string(),
      "Parsing error: Unknown ALife object section 'unknown_section', no CLSID mapping exists"
    );

    Ok(())
  }

  #[test]
  fn ltx_import_returns_error_for_unknown_section() {
    let mut ltx: Ltx = Ltx::new();
    ltx.with_section("object").set("section", "unknown_section");

    let error: XrfError = AlifeObject::import("object", &ltx).unwrap_err();

    assert_eq!(
      error.to_string(),
      "Parsing error: Unknown ALife object section 'unknown_section', no CLSID mapping exists"
    );
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObject = AlifeObject {
      id: 340,
      net_action: 1,
      section: String::from("dolg_heavy_outfit"),
      clsid: ClsId::EStlk,
      name: String::from("test-outfit-object"),
      script_game_id: 2,
      script_rp: 3,
      position: Vector3d::new(1.0, 2.0, 3.0),
      direction: Vector3d::new(3.0, 2.0, 1.0),
      respawn_time: 50000,
      parent_id: 2143,
      phantom_id: 0,
      script_flags: 33,
      version: 128,
      game_type: 1,
      script_version: 10,
      client_data_size: 0,
      spawn_id: 2354,
      inherited: AlifeObjectInherited::CseAlifeItemCustomOutfit(Box::new(AlifeObjectItemCustomOutfit {
        base: AlifeObjectItem {
          base: AlifeObjectDynamicVisual {
            base: AlifeObjectAbstract {
              game_vertex_id: 12434,
              distance: 124.33,
              direct_control: 624345,
              level_vertex_id: 48528,
              flags: 34,
              custom_data: String::from("custom-data"),
              story_id: 523,
              spawn_story_id: 2865268,
            },
            visual_name: String::from("visual-name"),
            visual_flags: 0,
          },
          condition: 1.0,
          upgrades_count: 0,
        },
      })),
      update_data: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 185);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 185);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 185 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeObject::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = build_relative_test_sample_file_path(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let original: AlifeObject = AlifeObject {
      id: 345,
      net_action: 1,
      section: String::from("dolg_heavy_outfit"),
      clsid: ClsId::EStlk,
      name: String::from("test-outfit-object"),
      script_game_id: 3,
      script_rp: 3,
      position: Vector3d::new(1.0, 2.0, 3.0),
      direction: Vector3d::new(3.0, 2.0, 1.0),
      respawn_time: 50000,
      parent_id: 2143,
      phantom_id: 0,
      script_flags: 33,
      version: 128,
      game_type: 1,
      script_version: 10,
      client_data_size: 0,
      spawn_id: 2354,
      inherited: AlifeObjectInherited::CseAlifeItemCustomOutfit(Box::new(AlifeObjectItemCustomOutfit {
        base: AlifeObjectItem {
          base: AlifeObjectDynamicVisual {
            base: AlifeObjectAbstract {
              game_vertex_id: 12,
              distance: 100.0,
              direct_control: 52,
              level_vertex_id: 364,
              flags: 33,
              custom_data: String::from("custom-data"),
              story_id: 523,
              spawn_story_id: 2865268,
            },
            visual_name: String::from("visual-name"),
            visual_flags: 0,
          },
          condition: 0.2,
          upgrades_count: 0,
        },
      })),
      update_data: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
    };

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut overwrite_generated_test_resource_as_file(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(build_absolute_generated_test_resource_path(&ltx_filename))?;

    assert_eq!(AlifeObject::import("data", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObject = AlifeObject {
      id: 341,
      net_action: 1,
      section: String::from("dolg_heavy_outfit"),
      clsid: ClsId::EStlk,
      name: String::from("test-outfit-object"),
      script_game_id: 2,
      script_rp: 3,
      position: Vector3d::new_mock(),
      direction: Vector3d::new_mock(),
      respawn_time: 50000,
      parent_id: 2143,
      phantom_id: 0,
      script_flags: 33,
      version: 128,
      game_type: 1,
      script_version: 10,
      client_data_size: 0,
      spawn_id: 2354,
      inherited: AlifeObjectInherited::CseAlifeItemCustomOutfit(Box::new(AlifeObjectItemCustomOutfit {
        base: AlifeObjectItem {
          base: AlifeObjectDynamicVisual {
            base: AlifeObjectAbstract {
              game_vertex_id: 145,
              distance: 52.33,
              direct_control: 256,
              level_vertex_id: 3634,
              flags: 24,
              custom_data: String::from("custom-data"),
              story_id: 414,
              spawn_story_id: 124,
            },
            visual_name: String::from("visual-name"),
            visual_flags: 2,
          },
          condition: 1.0,
          upgrades_count: 0,
        },
      })),
      update_data: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);

    assert_eq!(serde_json::from_str::<AlifeObject>(&serialized)?, original);

    Ok(())
  }
}
