use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::LtxImportExport;
use xrf_ltx::read_ltx_field;
use xrf_ltx::{Ltx, Section};
use xrf_utils::{vector_from_string, vector_to_string};

use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectCreature {
  pub base: AlifeObjectDynamicVisual,
  pub team: u8,
  pub squad: u8,
  pub group: u8,
  pub health: f32,
  pub dynamic_out_restrictions: Vec<u16>,
  pub dynamic_in_restrictions: Vec<u16>,
  pub killer_id: u16,
  pub game_death_time: u64,
}

impl ChunkReadWrite for AlifeObjectCreature {
  /// Read ALife creature object data from the chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      team: reader.read_u8()?,
      squad: reader.read_u8()?,
      group: reader.read_u8()?,
      health: reader.read_f32::<T>()?,
      dynamic_out_restrictions: reader.read_u16_vector::<T>()?,
      dynamic_in_restrictions: reader.read_u16_vector::<T>()?,
      killer_id: reader.read_u16::<T>()?,
      game_death_time: reader.read_u64::<T>()?,
    })
  }

  /// Write ALife creature object data into the chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;

    writer.write_u8(self.team)?;
    writer.write_u8(self.squad)?;
    writer.write_u8(self.group)?;
    writer.write_f32::<T>(self.health)?;

    writer.write_u16_vector::<T>(&self.dynamic_out_restrictions)?;
    writer.write_u16_vector::<T>(&self.dynamic_in_restrictions)?;

    writer.write_u16::<T>(self.killer_id)?;
    writer.write_u64::<T>(self.game_death_time)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectCreature {
  /// Import ALife creature object from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectDynamicVisual::import(section_name, ltx)?,
      team: read_ltx_field("creature.team", section)?,
      squad: read_ltx_field("creature.squad", section)?,
      group: read_ltx_field("creature.group", section)?,
      health: read_ltx_field("creature.health", section)?,
      dynamic_out_restrictions: vector_from_string(&read_ltx_field::<String>(
        "creature.dynamic_out_restrictions",
        section,
      )?)?,
      dynamic_in_restrictions: vector_from_string(&read_ltx_field::<String>(
        "creature.dynamic_in_restrictions",
        section,
      )?)?,
      killer_id: read_ltx_field("creature.killer_id", section)?,
      game_death_time: read_ltx_field("creature.game_death_time", section)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("creature.team", self.team.to_string())
      .set("creature.squad", self.squad.to_string())
      .set("creature.group", self.group.to_string())
      .set("creature.health", self.health.to_string())
      .set(
        "creature.dynamic_out_restrictions",
        vector_to_string(&self.dynamic_out_restrictions),
      )
      .set(
        "creature.dynamic_in_restrictions",
        vector_to_string(&self.dynamic_in_restrictions),
      )
      .set("creature.killer_id", self.killer_id.to_string())
      .set("creature.game_death_time", self.game_death_time.to_string());

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write};

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_ltx::LtxImportExport;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_resource_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_creature::AlifeObjectCreature;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectCreature = AlifeObjectCreature {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 1001,
          distance: 65.25,
          direct_control: 412421,
          level_vertex_id: 66231,
          flags: 33,
          custom_data: String::from("custom_data"),
          story_id: 400,
          spawn_story_id: 25,
        },
        visual_name: String::from("abcdef"),
        visual_flags: 33,
      },
      team: 2,
      squad: 3,
      group: 4,
      health: 1.0,
      dynamic_out_restrictions: vec![1, 2, 3, 4],
      dynamic_in_restrictions: vec![5, 6, 7, 8],
      killer_id: 25,
      game_death_time: 0,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 87);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 87);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 87 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;
    let read_object: AlifeObjectCreature = AlifeObjectCreature::read::<XRayByteOrder, _>(&mut reader)?;

    assert_eq!(read_object, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = build_relative_test_sample_file_path(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let first: AlifeObjectCreature = AlifeObjectCreature {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 3215,
          distance: 332.25,
          direct_control: 32451,
          level_vertex_id: 63663,
          flags: 36,
          custom_data: String::from("custom_data_first"),
          story_id: 243,
          spawn_story_id: 323,
        },
        visual_name: String::from("abcdef_first"),
        visual_flags: 33,
      },
      team: 3,
      squad: 4,
      group: 5,
      health: 0.5,
      dynamic_out_restrictions: vec![1, 2, 3, 4],
      dynamic_in_restrictions: vec![5, 6, 7, 8],
      killer_id: 25,
      game_death_time: 16,
    };

    let second: AlifeObjectCreature = AlifeObjectCreature {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 32,
          distance: 25.25,
          direct_control: 255,
          level_vertex_id: 634,
          flags: 36,
          custom_data: String::from("custom_data_second"),
          story_id: 43,
          spawn_story_id: 32,
        },
        visual_name: String::from("abcdef_second"),
        visual_flags: 35,
      },
      team: 4,
      squad: 5,
      group: 6,
      health: 0.5,
      dynamic_out_restrictions: vec![1, 2, 3, 4],
      dynamic_in_restrictions: vec![5, 6, 7, 8],
      killer_id: 25,
      game_death_time: 17,
    };

    first.export("first", &mut ltx)?;
    second.export("second", &mut ltx)?;

    ltx.write_to(&mut overwrite_generated_test_resource_as_file(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(build_absolute_generated_test_resource_path(&ltx_filename))?;

    assert_eq!(AlifeObjectCreature::import("first", &source)?, first);
    assert_eq!(AlifeObjectCreature::import("second", &source)?, second);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectCreature = AlifeObjectCreature {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 3215,
          distance: 332.25,
          direct_control: 32451,
          level_vertex_id: 63663,
          flags: 36,
          custom_data: String::from("custom_data_serde"),
          story_id: 243,
          spawn_story_id: 323,
        },
        visual_name: String::from("abcdef_serde"),
        visual_flags: 33,
      },
      team: 3,
      squad: 4,
      group: 5,
      health: 0.5,
      dynamic_out_restrictions: vec![1, 2, 3, 4],
      dynamic_in_restrictions: vec![5, 6, 7, 8],
      killer_id: 25,
      game_death_time: 0,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;

    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<AlifeObjectCreature>(&serialized)?);

    Ok(())
  }
}
