use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::LtxImportExport;
use xrf_ltx::META_TYPE_FIELD;
use xrf_ltx::read_ltx_field;
use xrf_ltx::{Ltx, Section};
use xrf_utils::assert_equal;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEffectFrame {
  pub texture_size: (f32, f32),
  pub reserved: (f32, f32),
  pub frame_dimension_x: u32,
  pub frame_count: u32,
  pub frame_speed: f32,
}

impl ParticleEffectFrame {
  pub const META_TYPE: &'static str = "particle_effect_frame";
}

impl ChunkReadWrite for ParticleEffectFrame {
  /// Read frame data from chunk redder.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let particle_frame: Self = Self {
      texture_size: (reader.read_f32::<T>()?, reader.read_f32::<T>()?),
      reserved: (reader.read_f32::<T>()?, reader.read_f32::<T>()?),
      frame_dimension_x: reader.read_u32::<T>()?,
      frame_count: reader.read_u32::<T>()?,
      frame_speed: reader.read_f32::<T>()?,
    };

    assert!(reader.is_ended(), "Expect particle frame chunk to be ended");

    Ok(particle_frame)
  }

  /// Write frame data into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32::<T>(self.texture_size.0)?;
    writer.write_f32::<T>(self.texture_size.1)?;
    writer.write_f32::<T>(self.reserved.0)?;
    writer.write_f32::<T>(self.reserved.1)?;
    writer.write_u32::<T>(self.frame_dimension_x)?;
    writer.write_u32::<T>(self.frame_count)?;
    writer.write_f32::<T>(self.frame_speed)?;

    Ok(())
  }
}

impl LtxImportExport for ParticleEffectFrame {
  /// Import particle effect frame data from provided path.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Particle group '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    let meta_type: String = read_ltx_field(META_TYPE_FIELD, section)?;

    assert_equal(
      meta_type.as_str(),
      Self::META_TYPE,
      "Expected corrected meta type field for particle effect frame importing",
    )?;

    let texture_size: Vec<String> = read_ltx_field::<String>("texture_size", section)?
      .split(',')
      .map(String::from)
      .collect();
    let reserved: Vec<String> = read_ltx_field::<String>("reserved", section)?
      .split(',')
      .map(String::from)
      .collect();

    Ok(Self {
      texture_size: (
        texture_size[0]
          .trim()
          .parse::<f32>()
          .or(Err(XrfError::new_parsing_error("Failed to parse texture_size W value")))?,
        texture_size[1]
          .trim()
          .parse::<f32>()
          .or(Err(XrfError::new_parsing_error("Failed to parse texture_size H value")))?,
      ),
      reserved: (
        reserved[0]
          .trim()
          .parse::<f32>()
          .or(Err(XrfError::new_parsing_error("Failed to parse reserved X value")))?,
        reserved[1]
          .trim()
          .parse::<f32>()
          .or(Err(XrfError::new_parsing_error("Failed to parse reserved Y value")))?,
      ),
      frame_dimension_x: read_ltx_field("frame_dimension_x", section)?,
      frame_count: read_ltx_field("frame_count", section)?,
      frame_speed: read_ltx_field("frame_speed", section)?,
    })
  }

  /// Export particle effect frame data into provided path.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx
      .with_section(section_name)
      .set(META_TYPE_FIELD, Self::META_TYPE)
      .set(
        "texture_size",
        format!("{},{}", self.texture_size.0, self.texture_size.1),
      )
      .set("reserved", format!("{},{}", self.reserved.0, self.reserved.1))
      .set("frame_dimension_x", self.frame_dimension_x.to_string())
      .set("frame_count", self.frame_count.to_string())
      .set("frame_speed", self.frame_speed.to_string());

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write};
  use std::path::Path;

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_ltx::LtxImportExport;
  use xrf_ltx::META_TYPE_FIELD;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_sample_file_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_file, overwrite_generated_test_resource_as_file,
  };

  use crate::data::particle_effect_frame::ParticleEffectFrame;

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: ParticleEffectFrame = ParticleEffectFrame {
      texture_size: (15.0, 54.5),
      reserved: (25.5, 325.5),
      frame_dimension_x: 155,
      frame_count: 30,
      frame_speed: 2857.0,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 28);

    let bytes_written: usize = writer.flush_chunk_into::<XRayByteOrder>(
      &mut overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?,
      0,
    )?;

    assert_eq!(bytes_written, 28);

    let file: FileSlice =
      open_generated_test_resource_as_slice(&build_relative_test_sample_file_path(file!(), &filename))?;

    assert_eq!(file.bytes_remaining(), 28 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    let read: ParticleEffectFrame = ParticleEffectFrame::read::<XRayByteOrder, _>(&mut reader)?;

    assert_eq!(read, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let config_path: &Path = &build_absolute_generated_test_sample_file_path(file!(), "import_export.ltx");
    let mut file: File = overwrite_file(config_path)?;
    let mut ltx: Ltx = Ltx::new();

    let original: ParticleEffectFrame = ParticleEffectFrame {
      texture_size: (1.5, 2.5),
      reserved: (5.3, 6.1),
      frame_dimension_x: 5,
      frame_count: 24,
      frame_speed: 61.3,
    };

    original.export("data", &mut ltx)?;
    ltx.write_to(&mut file)?;

    let read: ParticleEffectFrame = ParticleEffectFrame::import("data", &Ltx::read_from_path(config_path)?)?;

    assert_eq!(read, original);

    Ok(())
  }

  #[test]
  fn test_import_rejects_incorrect_meta_type() {
    let mut ltx: Ltx = Ltx::new();
    ltx.with_section("data").set(META_TYPE_FIELD, "incorrect_meta_type");

    assert!(
      ParticleEffectFrame::import("data", &ltx).is_err(),
      "Expect import to reject an incorrect particle effect frame meta type"
    );
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: ParticleEffectFrame = ParticleEffectFrame {
      texture_size: (74.0, 236.5),
      reserved: (263.5, 5369.5),
      frame_dimension_x: 7352,
      frame_count: 44,
      frame_speed: 1.5,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<ParticleEffectFrame>(&serialized)?);

    Ok(())
  }
}
