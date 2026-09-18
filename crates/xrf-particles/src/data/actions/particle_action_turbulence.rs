use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::LtxImportExport;
use xrf_ltx::read_ltx_field;
use xrf_ltx::{Ltx, Section};
use xrf_math::Vector3d;

use crate::data::particle_action_type::ParticleActionType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleActionTurbulence {
  pub action_flags: u32,
  pub action_type: ParticleActionType,
  pub frequency: f32,
  pub octaves: i32,
  pub magnitude: f32,
  pub epsilon: f32,
  pub offset: Vector3d,
}

impl ChunkReadWrite for ParticleActionTurbulence {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      action_flags: reader.read_u32::<T>()?,
      action_type: reader.read_xr::<T, _>()?,
      frequency: reader.read_f32::<T>()?,
      octaves: reader.read_i32::<T>()?,
      magnitude: reader.read_f32::<T>()?,
      epsilon: reader.read_f32::<T>()?,
      offset: reader.read_xr::<T, _>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.action_flags)?;
    writer.write_xr::<T, _>(&self.action_type)?;
    writer.write_f32::<T>(self.frequency)?;
    writer.write_i32::<T>(self.octaves)?;
    writer.write_f32::<T>(self.magnitude)?;
    writer.write_f32::<T>(self.epsilon)?;
    writer.write_xr::<T, _>(&self.offset)?;

    Ok(())
  }
}

impl LtxImportExport for ParticleActionTurbulence {
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Particle action section '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      action_flags: read_ltx_field("action_flags", section)?,
      action_type: read_ltx_field("action_type", section)?,
      frequency: read_ltx_field("frequency", section)?,
      octaves: read_ltx_field("octaves", section)?,
      magnitude: read_ltx_field("magnitude", section)?,
      epsilon: read_ltx_field("epsilon", section)?,
      offset: read_ltx_field("offset", section)?,
    })
  }

  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx
      .with_section(section_name)
      .set("action_flags", self.action_flags.to_string())
      .set("action_type", self.action_type.to_string())
      .set("frequency", self.frequency.to_string())
      .set("octaves", self.octaves.to_string())
      .set("magnitude", self.magnitude.to_string())
      .set("epsilon", self.epsilon.to_string())
      .set("offset", self.offset.to_string());

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
  use xrf_math::Vector3d;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_resource_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_generated_test_resource_as_file,
  };

  use crate::data::actions::particle_action_turbulence::ParticleActionTurbulence;
  use crate::data::particle_action_type::ParticleActionType;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: ParticleActionTurbulence = ParticleActionTurbulence {
      action_flags: 1,
      action_type: ParticleActionType::Turbulence,
      frequency: 3.0,
      octaves: 5,
      magnitude: 1.1,
      epsilon: 0.0005,
      offset: Vector3d::new_mock(),
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 36);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 36);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 36 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(
      ParticleActionTurbulence::read::<XRayByteOrder, _>(&mut reader)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = build_relative_test_sample_file_path(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let original: ParticleActionTurbulence = ParticleActionTurbulence {
      action_flags: 1,
      action_type: ParticleActionType::Turbulence,
      frequency: 4.0,
      octaves: 8,
      magnitude: 1.2004,
      epsilon: 0.0004,
      offset: Vector3d::new_mock(),
    };

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut overwrite_generated_test_resource_as_file(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(build_absolute_generated_test_resource_path(&ltx_filename))?;

    assert_eq!(ParticleActionTurbulence::import("data", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: ParticleActionTurbulence = ParticleActionTurbulence {
      action_flags: 1,
      action_type: ParticleActionType::Turbulence,
      frequency: 1.4,
      octaves: 1,
      magnitude: 2.5,
      epsilon: 0.0000015,
      offset: Vector3d::new_mock(),
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<ParticleActionTurbulence>(&serialized)?);

    Ok(())
  }
}
