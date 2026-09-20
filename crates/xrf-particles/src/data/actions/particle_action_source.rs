use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section, read_ltx_field};
use xrf_math::Vector3d;

use crate::data::particle_action_type::ParticleActionType;
use crate::data::particle_domain::ParticleDomain;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleActionSource {
  pub action_flags: u32,
  pub action_type: ParticleActionType,
  pub position: ParticleDomain,
  pub velocity: ParticleDomain,
  pub rot: ParticleDomain,
  pub size: ParticleDomain,
  pub color: ParticleDomain,
  pub alpha: f32,
  pub particle_rate: f32,
  pub age: f32,
  pub age_sigma: f32,
  pub parent_vel: Vector3d,
  pub parent_motion: f32,
}

impl ChunkReadWrite for ParticleActionSource {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<ParticleActionSource> {
    Ok(Self {
      action_flags: reader.read_u32::<T>()?,
      action_type: reader.read_xr::<T, _>()?,
      position: reader.read_xr::<T, _>()?,
      velocity: reader.read_xr::<T, _>()?,
      rot: reader.read_xr::<T, _>()?,
      size: reader.read_xr::<T, _>()?,
      color: reader.read_xr::<T, _>()?,
      alpha: reader.read_f32::<T>()?,
      particle_rate: reader.read_f32::<T>()?,
      age: reader.read_f32::<T>()?,
      age_sigma: reader.read_f32::<T>()?,
      parent_vel: reader.read_xr::<T, _>()?,
      parent_motion: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.action_flags)?;
    writer.write_xr::<T, _>(&self.action_type)?;
    writer.write_xr::<T, _>(&self.position)?;
    writer.write_xr::<T, _>(&self.velocity)?;
    writer.write_xr::<T, _>(&self.rot)?;
    writer.write_xr::<T, _>(&self.size)?;
    writer.write_xr::<T, _>(&self.color)?;
    writer.write_f32::<T>(self.alpha)?;
    writer.write_f32::<T>(self.particle_rate)?;
    writer.write_f32::<T>(self.age)?;
    writer.write_f32::<T>(self.age_sigma)?;
    writer.write_xr::<T, _>(&self.parent_vel)?;
    writer.write_f32::<T>(self.parent_motion)?;

    Ok(())
  }
}

impl LtxImportExport for ParticleActionSource {
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
      position: read_ltx_field("position", section)?,
      velocity: read_ltx_field("velocity", section)?,
      rot: read_ltx_field("rot", section)?,
      size: read_ltx_field("size", section)?,
      color: read_ltx_field("color", section)?,
      alpha: read_ltx_field("alpha", section)?,
      particle_rate: read_ltx_field("particle_rate", section)?,
      age: read_ltx_field("age", section)?,
      age_sigma: read_ltx_field("age_sigma", section)?,
      parent_vel: read_ltx_field("parent_vel", section)?,
      parent_motion: read_ltx_field("parent_motion", section)?,
    })
  }

  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx
      .with_section(section_name)
      .set("action_flags", self.action_flags.to_string())
      .set("action_type", self.action_type.to_string())
      .set("position", self.position.to_string())
      .set("velocity", self.velocity.to_string())
      .set("rot", self.rot.to_string())
      .set("size", self.size.to_string())
      .set("color", self.color.to_string())
      .set("alpha", self.alpha.to_string())
      .set("particle_rate", self.particle_rate.to_string())
      .set("age", self.age.to_string())
      .set("age_sigma", self.age_sigma.to_string())
      .set("parent_vel", self.parent_vel.to_string())
      .set("parent_motion", self.parent_motion.to_string());

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
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_math::Vector3d;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_resource_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_generated_test_resource_as_file,
  };

  use crate::data::actions::particle_action_source::ParticleActionSource;
  use crate::data::particle_action_type::ParticleActionType;
  use crate::data::particle_domain::ParticleDomain;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: ParticleActionSource = ParticleActionSource {
      action_flags: 1,
      action_type: ParticleActionType::Move,
      position: ParticleDomain::new_mock(),
      velocity: ParticleDomain::new_mock(),
      rot: ParticleDomain::new_mock(),
      size: ParticleDomain::new_mock(),
      color: ParticleDomain::new_mock(),
      alpha: 0.45,
      particle_rate: 2.3,
      age: 452.33,
      age_sigma: 2.532,
      parent_vel: Vector3d::new_mock(),
      parent_motion: 1.4324,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 380);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 380);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 380 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(ParticleActionSource::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = build_relative_test_sample_file_path(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let original: ParticleActionSource = ParticleActionSource {
      action_flags: 1,
      action_type: ParticleActionType::Move,
      position: ParticleDomain::new_mock(),
      velocity: ParticleDomain::new_mock(),
      rot: ParticleDomain::new_mock(),
      size: ParticleDomain::new_mock(),
      color: ParticleDomain::new_mock(),
      alpha: 0.45,
      particle_rate: 2.3,
      age: 452.33,
      age_sigma: 2.532,
      parent_vel: Vector3d::new_mock(),
      parent_motion: 1.4324,
    };

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut overwrite_generated_test_resource_as_file(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(build_absolute_generated_test_resource_path(&ltx_filename))?;

    assert_eq!(ParticleActionSource::import("data", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: ParticleActionSource = ParticleActionSource {
      action_flags: 1,
      action_type: ParticleActionType::Move,
      position: ParticleDomain::new_mock(),
      velocity: ParticleDomain::new_mock(),
      rot: ParticleDomain::new_mock(),
      size: ParticleDomain::new_mock(),
      color: ParticleDomain::new_mock(),
      alpha: 0.45,
      particle_rate: 2.3,
      age: 452.33,
      age_sigma: 2.532,
      parent_vel: Vector3d::new_mock(),
      parent_motion: 1.4324,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<ParticleActionSource>(&serialized)?);

    Ok(())
  }
}
