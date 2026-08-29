use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};

use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
use crate::data::alife::inherited::alife_object_motion::AlifeObjectMotion;
use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;
use crate::export::LtxImportExport;
use crate::file_import::read_ltx_field;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectHelicopter {
  pub base: AlifeObjectDynamicVisual,
  pub motion: AlifeObjectMotion,
  pub skeleton: AlifeObjectSkeleton,
  pub startup_animation: String,
  pub engine_sound: String,
}

impl ChunkReadWrite for AlifeObjectHelicopter {
  /// Read helicopter data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      motion: reader.read_xr::<T, _>()?,
      skeleton: reader.read_xr::<T, _>()?,
      startup_animation: reader.read_w1251_string()?,
      engine_sound: reader.read_w1251_string()?,
    })
  }

  /// Write helicopter data into the chunk.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_xr::<T, _>(&self.motion)?;
    writer.write_xr::<T, _>(&self.skeleton)?;

    writer.write_w1251_string(&self.startup_animation)?;
    writer.write_w1251_string(&self.engine_sound)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectHelicopter {
  /// Import helicopter object data from ltx config section.
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
      motion: AlifeObjectMotion::import(section_name, ltx)?,
      startup_animation: read_ltx_field("helicopter.startup_animation", section)?,
      engine_sound: read_ltx_field("helicopter.engine_sound", section)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;
    self.motion.export(section_name, ltx)?;
    self.skeleton.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("helicopter.startup_animation", &self.startup_animation)
      .set("helicopter.engine_sound", &self.engine_sound);

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
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_resource_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_helicopter::AlifeObjectHelicopter;
  use crate::data::alife::inherited::alife_object_motion::AlifeObjectMotion;
  use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;
  use crate::export::LtxImportExport;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectHelicopter = AlifeObjectHelicopter {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 6432,
          distance: 243.53,
          direct_control: 25364,
          level_vertex_id: 3541,
          flags: 43,
          custom_data: String::from("custom-data"),
          story_id: 64353,
          spawn_story_id: 2533,
        },
        visual_name: String::from("visual-name"),
        visual_flags: 43,
      },
      skeleton: AlifeObjectSkeleton {
        name: String::from("skeleton-name"),
        flags: 0,
        source_id: 235,
      },
      motion: AlifeObjectMotion {
        motion_name: String::from("motion-name"),
      },
      startup_animation: String::from("startup-animation"),
      engine_sound: String::from("engine-sound"),
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 111);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 111);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 111 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeObjectHelicopter::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = build_relative_test_sample_file_path(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let original: AlifeObjectHelicopter = AlifeObjectHelicopter {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 253,
          distance: 25.53,
          direct_control: 236,
          level_vertex_id: 26,
          flags: 364,
          custom_data: String::from("custom-data"),
          story_id: 26,
          spawn_story_id: 346,
        },
        visual_name: String::from("visual-name"),
        visual_flags: 32,
      },
      skeleton: AlifeObjectSkeleton {
        name: String::from("skeleton-name"),
        flags: 32,
        source_id: 235,
      },
      motion: AlifeObjectMotion {
        motion_name: String::from("motion-name"),
      },
      startup_animation: String::from("startup-animation"),
      engine_sound: String::from("engine-sound"),
    };

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut overwrite_generated_test_resource_as_file(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(build_absolute_generated_test_resource_path(&ltx_filename))?;

    assert_eq!(AlifeObjectHelicopter::import("data", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectHelicopter = AlifeObjectHelicopter {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 25,
          distance: 253.53,
          direct_control: 126,
          level_vertex_id: 6,
          flags: 263,
          custom_data: String::from("custom-data"),
          story_id: 253,
          spawn_story_id: 235,
        },
        visual_name: String::from("visual-name"),
        visual_flags: 253,
      },
      skeleton: AlifeObjectSkeleton {
        name: String::from("skeleton-name"),
        flags: 10,
        source_id: 253,
      },
      motion: AlifeObjectMotion {
        motion_name: String::from("motion-name"),
      },
      startup_animation: String::from("startup-animation"),
      engine_sound: String::from("engine-sound"),
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);

    assert_eq!(serde_json::from_str::<AlifeObjectHelicopter>(&serialized)?, original);

    Ok(())
  }
}
