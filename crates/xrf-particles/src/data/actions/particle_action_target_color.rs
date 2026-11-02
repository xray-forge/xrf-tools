use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};
use xrf_math::Vector3d;

use crate::data::particle_action_type::ParticleActionType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleActionTargetColor {
  pub action_flags: u32,
  pub action_type: ParticleActionType,
  pub color: Vector3d,
  pub alpha: f32,
  pub scale: f32,
  // CS / COP only:
  pub time_from: f32,
  pub time_to: f32,
}

impl ChunkReadWrite for ParticleActionTargetColor {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      action_flags: reader.read_u32::<T>()?,
      action_type: reader.read_xr::<T, _>()?,
      color: reader.read_xr::<T, _>()?,
      alpha: reader.read_f32::<T>()?,
      scale: reader.read_f32::<T>()?,
      // CS / COP only:
      time_from: reader.read_f32::<T>()?,
      time_to: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.action_flags)?;
    writer.write_xr::<T, _>(&self.action_type)?;
    writer.write_xr::<T, _>(&self.color)?;
    writer.write_f32::<T>(self.alpha)?;
    writer.write_f32::<T>(self.scale)?;
    writer.write_f32::<T>(self.time_from)?;
    writer.write_f32::<T>(self.time_to)?;

    Ok(())
  }
}

impl LtxImportExport for ParticleActionTargetColor {
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Particle action section '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      action_flags: section.read_field("action_flags")?,
      action_type: section.read_field("action_type")?,
      color: section.read_field("color")?,
      alpha: section.read_field("alpha")?,
      scale: section.read_field("scale")?,
      time_from: section.read_field("time_from")?,
      time_to: section.read_field("time_to")?,
    })
  }

  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx
      .with_section(section_name)
      .set("action_flags", self.action_flags.to_string())
      .set("action_type", self.action_type.to_string())
      .set("color", self.color.to_string())
      .set("alpha", self.alpha.to_string())
      .set("scale", self.scale.to_string())
      .set("time_from", self.time_from.to_string())
      .set("time_to", self.time_to.to_string());

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write, read_to_string};

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_math::Vector3d;
  use xrf_test_utils::{FileSlice, GeneratedResources};

  use crate::data::actions::particle_action_target_color::ParticleActionTargetColor;
  use crate::data::particle_action_type::ParticleActionType;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = GeneratedResources::sample_name(file!(), "read_write.chunk");

    let original: ParticleActionTargetColor = ParticleActionTargetColor {
      action_flags: 1,
      action_type: ParticleActionType::TargetColor,
      color: Vector3d::new_mock(),
      alpha: 21.4,
      scale: 1.2,
      time_from: 1.0,
      time_to: 2.0,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 36);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut GeneratedResources::overwrite(&filename)?, 0)?;

    assert_eq!(bytes_written, 36);

    let file: FileSlice = GeneratedResources::open_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 36 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(
      ParticleActionTargetColor::read::<XRayByteOrder, _>(&mut reader)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = GeneratedResources::sample_name(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let original: ParticleActionTargetColor = ParticleActionTargetColor {
      action_flags: 1,
      action_type: ParticleActionType::TargetColor,
      color: Vector3d::new_mock(),
      alpha: 1.4,
      scale: 2.2,
      time_from: 2.0,
      time_to: 3.0,
    };

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut GeneratedResources::overwrite(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(GeneratedResources::path(&ltx_filename))?;

    assert_eq!(ParticleActionTargetColor::import("data", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: ParticleActionTargetColor = ParticleActionTargetColor {
      action_flags: 1,
      action_type: ParticleActionType::TargetColor,
      color: Vector3d::new_mock(),
      alpha: 12.4,
      scale: 10.2,
      time_from: 10.0,
      time_to: 20.0,
    };
    let mut file: File =
      GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), "serialize_deserialize.json"))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_to_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(
      original,
      serde_json::from_str::<ParticleActionTargetColor>(&serialized)?
    );

    Ok(())
  }
}
