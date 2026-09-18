use std::fmt::Display;
use std::str::FromStr;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ParticleDomain {
  pub domain_type: u32,
  pub coordinates: (Vector3d, Vector3d),
  pub basis: (Vector3d, Vector3d),
  pub radius1: f32,
  pub radius2: f32,
  pub radius1_sqr: f32,
  pub radius2_sqr: f32,
}

impl ChunkReadWrite for ParticleDomain {
  /// Read particle domain from chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      domain_type: reader.read_u32::<T>()?,
      coordinates: (reader.read_xr::<T, _>()?, reader.read_xr::<T, _>()?),
      basis: (reader.read_xr::<T, _>()?, reader.read_xr::<T, _>()?),
      radius1: reader.read_f32::<T>()?,
      radius2: reader.read_f32::<T>()?,
      radius1_sqr: reader.read_f32::<T>()?,
      radius2_sqr: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<XRayByteOrder>(self.domain_type)?;
    writer.write_xr::<T, _>(&self.coordinates.0)?;
    writer.write_xr::<T, _>(&self.coordinates.1)?;
    writer.write_xr::<T, _>(&self.basis.0)?;
    writer.write_xr::<T, _>(&self.basis.1)?;
    writer.write_f32::<XRayByteOrder>(self.radius1)?;
    writer.write_f32::<XRayByteOrder>(self.radius2)?;
    writer.write_f32::<XRayByteOrder>(self.radius1_sqr)?;
    writer.write_f32::<XRayByteOrder>(self.radius2_sqr)?;

    Ok(())
  }
}

impl Display for ParticleDomain {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      formatter,
      "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
      self.domain_type,
      self.coordinates.0.x,
      self.coordinates.0.y,
      self.coordinates.0.z,
      self.coordinates.1.x,
      self.coordinates.1.y,
      self.coordinates.1.z,
      self.basis.0.x,
      self.basis.0.y,
      self.basis.0.z,
      self.basis.1.x,
      self.basis.1.y,
      self.basis.1.z,
      self.radius1,
      self.radius2,
      self.radius1_sqr,
      self.radius2_sqr,
    )
  }
}

impl FromStr for ParticleDomain {
  type Err = XrfError;

  fn from_str(string: &str) -> Result<Self, Self::Err> {
    let parts: Vec<&str> = string.split(',').map(str::trim).collect();

    if parts.len() != 17 {
      return Err(XrfError::new_parsing_error(
        "Failed to parse particle domain from string, expected 17 numbers",
      ));
    }

    Ok(Self {
      domain_type: parts[0].parse::<u32>().or(Err(XrfError::new_parsing_error(
        "Failed to parse vector domain_type value",
      )))?,
      coordinates: (
        Vector3d {
          x: parts[1].parse::<f32>().or(Err(XrfError::new_parsing_error(
            "Failed to parse coordinates 0 vector x value",
          )))?,
          y: parts[2].parse::<f32>().or(Err(XrfError::new_parsing_error(
            "Failed to parse coordinates 0 vector y value",
          )))?,
          z: parts[3].parse::<f32>().or(Err(XrfError::new_parsing_error(
            "Failed to parse coordinates 0 vector z value",
          )))?,
        },
        Vector3d {
          x: parts[4].parse::<f32>().or(Err(XrfError::new_parsing_error(
            "Failed to parse coordinates 1 vector x value",
          )))?,
          y: parts[5].parse::<f32>().or(Err(XrfError::new_parsing_error(
            "Failed to parse coordinates 1 vector y value",
          )))?,
          z: parts[6].parse::<f32>().or(Err(XrfError::new_parsing_error(
            "Failed to parse coordinates 1 vector z value",
          )))?,
        },
      ),
      basis: (
        Vector3d {
          x: parts[7].parse::<f32>().or(Err(XrfError::new_parsing_error(
            "Failed to parse basis 0 vector x value",
          )))?,
          y: parts[8].parse::<f32>().or(Err(XrfError::new_parsing_error(
            "Failed to parse basis 0 vector y value",
          )))?,
          z: parts[9].parse::<f32>().or(Err(XrfError::new_parsing_error(
            "Failed to parse basis 0 vector z value",
          )))?,
        },
        Vector3d {
          x: parts[10].parse::<f32>().or(Err(XrfError::new_parsing_error(
            "Failed to parse basis 1 vector x value",
          )))?,
          y: parts[11].parse::<f32>().or(Err(XrfError::new_parsing_error(
            "Failed to parse basis 1 vector y value",
          )))?,
          z: parts[12].parse::<f32>().or(Err(XrfError::new_parsing_error(
            "Failed to parse basis 1 vector z value",
          )))?,
        },
      ),
      radius1: parts[13]
        .parse::<f32>()
        .or(Err(XrfError::new_parsing_error("Failed to parse vector radius1 value")))?,
      radius2: parts[14]
        .parse::<f32>()
        .or(Err(XrfError::new_parsing_error("Failed to parse vector radius2 value")))?,
      radius1_sqr: parts[15].parse::<f32>().or(Err(XrfError::new_parsing_error(
        "Failed to parse vector radius1_sqr value",
      )))?,
      radius2_sqr: parts[16].parse::<f32>().or(Err(XrfError::new_parsing_error(
        "Failed to parse vector radius2_sqr value",
      )))?,
    })
  }
}

#[cfg(test)]
impl ParticleDomain {
  pub fn new_mock() -> Self {
    Self {
      domain_type: 1,
      coordinates: (
        Vector3d {
          x: 1.5,
          y: 1.25,
          z: 1.75,
        },
        Vector3d { x: 2.5, y: 2.1, z: 2.8 },
      ),
      basis: (
        Vector3d {
          x: 0.5,
          y: -1.3,
          z: 0.4,
        },
        Vector3d { x: 2.3, y: 0.3, z: 3.3 },
      ),
      radius1: 100.0,
      radius2: 400.0,
      radius1_sqr: 10.0,
      radius2_sqr: 20.0,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write};
  use std::str::FromStr;

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::data::particles::particle_domain::ParticleDomain;

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: ParticleDomain = ParticleDomain::new_mock();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 68);

    let bytes_written: usize = writer.flush_chunk_into::<XRayByteOrder>(
      &mut overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?,
      0,
    )?;

    assert_eq!(bytes_written, 68);

    let file: FileSlice =
      open_generated_test_resource_as_slice(&build_relative_test_sample_file_path(file!(), &filename))?;

    assert_eq!(file.bytes_remaining(), 68 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(ParticleDomain::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_from_to_str() -> XrfResult {
    let original: ParticleDomain = ParticleDomain::new_mock();

    assert_eq!(
      original.to_string(),
      "1,1.5,1.25,1.75,2.5,2.1,2.8,0.5,-1.3,0.4,2.3,0.3,3.3,100,400,10,20"
    );
    assert_eq!(
      ParticleDomain::from_str("1,1.5,1.25,1.75,2.5,2.1,2.8,0.5,-1.3,0.4,2.3,0.3,3.3,100,400,10,20")?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: ParticleDomain = ParticleDomain::new_mock();

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<ParticleDomain>(&serialized)?);

    Ok(())
  }
}
