use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::LtxImportExport;
use xrf_ltx::read_ltx_field;
use xrf_ltx::{Ltx, Section};

use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectPhysic {
  pub base: AlifeObjectDynamicVisual,
  pub skeleton: AlifeObjectSkeleton,
  pub physic_type: u32,
  pub mass: f32,
  pub fixed_bones: String,
}

impl ChunkReadWrite for AlifeObjectPhysic {
  /// Read ALife physic object from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      skeleton: reader.read_xr::<T, _>()?,
      physic_type: reader.read_u32::<T>()?,
      mass: reader.read_f32::<T>()?,
      fixed_bones: reader.read_w1251_string()?,
    })
  }

  /// Write alife physic object into the chunk.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_xr::<T, _>(&self.skeleton)?;
    writer.write_u32::<T>(self.physic_type)?;
    writer.write_f32::<T>(self.mass)?;
    writer.write_w1251_string(&self.fixed_bones)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectPhysic {
  /// Import ALife physic object data from ltx config section.
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
      skeleton: AlifeObjectSkeleton::import(section_name, ltx)?,
      physic_type: read_ltx_field("physic_type", section)?,
      mass: read_ltx_field("mass", section)?,
      fixed_bones: read_ltx_field("fixed_bones", section)?,
    })
  }
  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;
    self.skeleton.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("physic_type", self.physic_type.to_string())
      .set("mass", self.mass.to_string())
      .set("fixed_bones", &self.fixed_bones);

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_physic::AlifeObjectPhysic;
  use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectPhysic = AlifeObjectPhysic {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 35794,
          distance: 25.23,
          direct_control: 1243,
          level_vertex_id: 34623,
          flags: 62,
          custom_data: String::from("custom-data"),
          story_id: 825679,
          spawn_story_id: 1452,
        },
        visual_name: String::from("visual-name"),
        visual_flags: 34,
      },
      skeleton: AlifeObjectSkeleton {
        name: String::from("skeleton-name"),
        flags: 0,
        source_id: 2153,
      },
      physic_type: 6,
      mass: 5.0,
      fixed_bones: String::from("fixed-bones"),
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 88);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 88);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 88 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeObjectPhysic::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }
}
