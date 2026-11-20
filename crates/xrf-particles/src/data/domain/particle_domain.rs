use std::fmt::Display;
use std::str::FromStr;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::data::domain::particle_domain_basis::ParticleDomainBasis;
use crate::data::domain::particle_domain_points::ParticleDomainPoints;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ParticleDomain {
  pub domain_type: u32,
  pub coordinates: ParticleDomainPoints,
  pub basis: ParticleDomainBasis,
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
      coordinates: ParticleDomainPoints {
        p1: reader.read_xr::<T, _>()?,
        p2: reader.read_xr::<T, _>()?,
      },
      basis: ParticleDomainBasis {
        u: reader.read_xr::<T, _>()?,
        v: reader.read_xr::<T, _>()?,
      },
      radius1: reader.read_f32::<T>()?,
      radius2: reader.read_f32::<T>()?,
      radius1_sqr: reader.read_f32::<T>()?,
      radius2_sqr: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.domain_type)?;
    writer.write_xr::<T, _>(&self.coordinates.p1)?;
    writer.write_xr::<T, _>(&self.coordinates.p2)?;
    writer.write_xr::<T, _>(&self.basis.u)?;
    writer.write_xr::<T, _>(&self.basis.v)?;
    writer.write_f32::<T>(self.radius1)?;
    writer.write_f32::<T>(self.radius2)?;
    writer.write_f32::<T>(self.radius1_sqr)?;
    writer.write_f32::<T>(self.radius2_sqr)?;

    Ok(())
  }
}

impl Display for ParticleDomain {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      formatter,
      "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
      self.domain_type,
      self.coordinates.p1.x,
      self.coordinates.p1.y,
      self.coordinates.p1.z,
      self.coordinates.p2.x,
      self.coordinates.p2.y,
      self.coordinates.p2.z,
      self.basis.u.x,
      self.basis.u.y,
      self.basis.u.z,
      self.basis.v.x,
      self.basis.v.y,
      self.basis.v.z,
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

    let invalid = |index: usize, what: &str| {
      XrfError::new_parsing_error(format!(
        "Failed to parse particle domain {what} value '{}'",
        parts[index]
      ))
    };
    let number =
      |index: usize, what: &str| -> XrfResult<f32> { parts[index].parse::<f32>().map_err(|_| invalid(index, what)) };
    let vector = |first: usize, what: &str| -> XrfResult<Vector3d> {
      Ok(Vector3d {
        x: number(first, &format!("{what} x"))?,
        y: number(first + 1, &format!("{what} y"))?,
        z: number(first + 2, &format!("{what} z"))?,
      })
    };

    Ok(Self {
      domain_type: parts[0].parse::<u32>().map_err(|_| invalid(0, "domain_type"))?,
      coordinates: ParticleDomainPoints {
        p1: vector(1, "coordinates 0")?,
        p2: vector(4, "coordinates 1")?,
      },
      basis: ParticleDomainBasis {
        u: vector(7, "basis 0")?,
        v: vector(10, "basis 1")?,
      },
      radius1: number(13, "radius1")?,
      radius2: number(14, "radius2")?,
      radius1_sqr: number(15, "radius1_sqr")?,
      radius2_sqr: number(16, "radius2_sqr")?,
    })
  }
}

#[cfg(test)]
impl ParticleDomain {
  pub fn new_mock() -> Self {
    Self {
      domain_type: 1,
      coordinates: ParticleDomainPoints {
        p1: Vector3d {
          x: 1.5,
          y: 1.25,
          z: 1.75,
        },
        p2: Vector3d { x: 2.5, y: 2.1, z: 2.8 },
      },
      basis: ParticleDomainBasis {
        u: Vector3d {
          x: 0.5,
          y: -1.3,
          z: 0.4,
        },
        v: Vector3d { x: 2.3, y: 0.3, z: 3.3 },
      },
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
  use std::io::{Seek, SeekFrom, Write, read_to_string};
  use std::str::FromStr;

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::{FileSlice, GeneratedResources};

  use crate::data::domain::particle_domain::ParticleDomain;

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: ParticleDomain = ParticleDomain::new_mock();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 68);

    let bytes_written: usize = writer.flush_chunk_into::<XRayByteOrder>(
      &mut GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), &filename))?,
      0,
    )?;

    assert_eq!(bytes_written, 68);

    let file: FileSlice = GeneratedResources::open_slice(&GeneratedResources::sample_name(file!(), &filename))?;

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

    let mut file: File =
      GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), "serialize_deserialize.json"))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_to_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<ParticleDomain>(&serialized)?);

    Ok(())
  }

  #[test]
  fn test_serializes_each_pair_as_an_array() -> XrfResult {
    let value: serde_json::Value = serde_json::to_value(ParticleDomain::new_mock())?;

    assert_eq!(value["coordinates"].as_array().map(Vec::len), Some(2));
    assert_eq!(value["coordinates"][0]["x"], 1.5);
    assert_eq!(value["basis"].as_array().map(Vec::len), Some(2));
    assert_eq!(value["basis"][1]["z"], 3.3_f32);

    Ok(())
  }
}
