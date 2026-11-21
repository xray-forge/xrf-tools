use serde::{Deserialize, Serialize};
use xrf_chunk::{
  ChunkDataSource, ChunkIterator, ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter, XRayRead, XRayWrite,
};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxImportExport, LtxRead, LtxSectionRead, LtxSectionWrite, LtxWrite, is_value_kept};
use xrf_math::Vector3d;
use xrf_utils::{assert_equal, decode_w1251_bytes_to_string, encode_string_to_w1251_bytes, narrow_format_size};

/// `CPatrolPoint::load_raw`, `CPatrolPoint::load` in xray codebase.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatrolPoint {
  pub name: String,
  pub position: Vector3d<f32>,
  pub flags: u32,
  pub level_vertex_id: u32,
  pub game_vertex_id: u16,
}

impl PatrolPoint {
  pub const INDEX_CHUNK_ID: u32 = 0;
  pub const DATA_CHUNK_ID: u32 = 1;
}

impl ChunkReadWriteList for PatrolPoint {
  /// Read points from the chunk reader.
  fn read_list<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<Self>> {
    let mut points: Vec<Self> = Vec::new();

    for (index, point_reader) in ChunkIterator::from_start(reader)?.enumerate() {
      let mut point_reader: ChunkReader<D> = point_reader?;

      let mut index_reader: ChunkReader<D> = point_reader.read_child_by_index(Self::INDEX_CHUNK_ID)?;
      let mut data_reader: ChunkReader<D> = point_reader.read_child_by_index(Self::DATA_CHUNK_ID)?;

      assert_equal(
        index,
        index_reader.read_u32()? as usize,
        "Expect correct patrol point index",
      )?;

      points.push(Self::read(&mut data_reader)?);

      index_reader.assert_read("Patrol point index chunk should be read")?;
      point_reader.assert_read("Patrol point data chunk should be read")?;
    }

    reader.assert_read("Patrol points chunk should be read")?;

    Ok(points)
  }

  /// Write list of patrol points into chunk writer.
  fn write_list(writer: &mut ChunkWriter, list: &[Self]) -> XrfResult {
    for (index, point) in list.iter().enumerate() {
      let index: u32 = narrow_format_size(index, "patrol point index")?;

      let mut point_chunk_writer: ChunkWriter = ChunkWriter::new();

      let mut point_index_writer: ChunkWriter = ChunkWriter::new();
      let mut point_writer: ChunkWriter = ChunkWriter::new();

      point_index_writer.write_u32(index)?;
      point.write(&mut point_writer)?;

      point_index_writer.flush_chunk_into(&mut point_chunk_writer, Self::INDEX_CHUNK_ID)?;
      point_writer.flush_chunk_into(&mut point_chunk_writer, Self::DATA_CHUNK_ID)?;

      point_chunk_writer.flush_chunk_into(writer, index)?;
    }

    Ok(())
  }
}

impl ChunkReadWrite for PatrolPoint {
  /// Read patrol point data from the chunk reader.
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let point: Self = Self {
      name: reader.read_w1251_string()?,
      position: reader.read_xr()?,
      flags: reader.read_u32()?,
      level_vertex_id: reader.read_u32()?,
      game_vertex_id: reader.read_u16()?,
    };

    reader.assert_read("Chunk data should be read for patrol point")?;

    Ok(point)
  }

  /// Write patrol point data into chunk writer.
  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.name)?;
    writer.write_xr(&self.position)?;
    writer.write_u32(self.flags)?;
    writer.write_u32(self.level_vertex_id)?;
    writer.write_u16(self.game_vertex_id)?;

    Ok(())
  }
}

impl PatrolPoint {
  /// The name as `name_bytes` carries it where no value can: its Windows-1251 bytes in hex.
  fn get_name_bytes(&self) -> XrfResult<String> {
    Ok(
      encode_string_to_w1251_bytes(&self.name)?
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect(),
    )
  }

  /// The name `name_bytes` spells.
  fn read_name_bytes(section_name: &str, hex: &str) -> XrfResult<String> {
    let invalid = || {
      XrfError::new_parsing_error(format!(
        "Patrol point '{section_name}' has name bytes '{hex}' no hex spells"
      ))
    };

    if !hex.len().is_multiple_of(2) || !hex.is_ascii() {
      return Err(invalid());
    }

    let bytes: Vec<u8> = (0..hex.len())
      .step_by(2)
      .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).map_err(|_| invalid()))
      .collect::<XrfResult<_>>()?;

    Ok(decode_w1251_bytes_to_string(&bytes)?)
  }
}

impl LtxImportExport for PatrolPoint {
  /// Import patrol point data from ltx config: its name from `name`, or from `name_bytes` where no value could carry it.
  fn import<L: LtxRead>(section_name: &str, ltx: &L) -> XrfResult<Self> {
    let section: &L::Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Patrol point section '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;
    let name: String = match section.read_optional_field::<String>("name_bytes")? {
      Some(hex) => Self::read_name_bytes(section_name, &hex)?,
      None => section.read_field("name")?,
    };

    Ok(Self {
      name,
      position: section.read_field("position")?,
      flags: section.read_field("flags")?,
      level_vertex_id: section.read_field("level_vertex_id")?,
      game_vertex_id: section.read_field("game_vertex_id")?,
    })
  }

  /// Export patrol point data into ltx: the name as `name` where a value reads back as it, else as `name_bytes`, as a
  /// name padded with whitespace, or holding a comment or an open quote, cannot be a value.
  fn export(&self, section_name: &str, ltx: &mut impl LtxWrite) -> XrfResult {
    let mut section = ltx.with_section(section_name);

    if is_value_kept(&self.name) {
      section.set("name", &self.name);
    } else {
      section.set("name_bytes", self.get_name_bytes()?);
    }

    section
      .set("flags", self.flags)
      .set("position", self.position)
      .set("level_vertex_id", self.level_vertex_id)
      .set("game_vertex_id", self.game_vertex_id);

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write, read_to_string};
  use std::path::Path;

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter};
  use xrf_error::{MapXrfErr, XrfResult};
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_math::Vector3d;
  use xrf_test_utils::{FileSlice, GeneratedResources};

  use crate::data::patrols::patrol_point::PatrolPoint;

  #[test]
  fn test_read_write_list() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = GeneratedResources::sample_name(file!(), "read_write_list.chunk");

    let original: Vec<PatrolPoint> = vec![
      PatrolPoint {
        name: String::from("wp00|a=probe_stand"),
        position: Vector3d::new(-39.898224, 24.269588, 324.09656),
        flags: 1,
        level_vertex_id: 866599,
        game_vertex_id: 60,
      },
      PatrolPoint {
        name: String::from("wp01|a=probe_stand"),
        position: Vector3d::new(-32.18162, 24.257412, 315.80127),
        flags: 2,
        level_vertex_id: 882385,
        game_vertex_id: 60,
      },
      PatrolPoint {
        name: String::from("wp01|a=probe_way"),
        position: Vector3d::new(-22.432571, 24.097664, 335.9503),
        flags: 8,
        level_vertex_id: 901200,
        game_vertex_id: 237,
      },
      PatrolPoint {
        name: String::from("wp02|a=probe_stand"),
        position: Vector3d::new(-36.36119, 24.754417, 344.12573),
        flags: 4,
        level_vertex_id: 873987,
        game_vertex_id: 237,
      },
    ];

    PatrolPoint::write_list(&mut writer, &original)?;

    assert_eq!(writer.bytes_written(), 274);

    let bytes_written: usize = writer.flush_chunk_into(&mut GeneratedResources::overwrite(&filename)?, 0)?;

    assert_eq!(bytes_written, 274);

    let file: FileSlice = GeneratedResources::open_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 274 + 8);
    assert_eq!(
      PatrolPoint::read_list(&mut ChunkReader::from_slice(file)?.read_child_by_index(0)?)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = GeneratedResources::sample_name(file!(), "read_write.chunk");

    let original: PatrolPoint = PatrolPoint {
      name: String::from("wp01|a=probe_way"),
      position: Vector3d::new(-22.432571, 24.097664, 335.9503),
      flags: 8,
      level_vertex_id: 901200,
      game_vertex_id: 237,
    };

    original.write(&mut writer)?;

    assert_eq!(writer.bytes_written(), 39);

    let bytes_written: usize = writer.flush_chunk_into(&mut GeneratedResources::overwrite(&filename)?, 0)?;

    assert_eq!(bytes_written, 39);

    let file: FileSlice = GeneratedResources::open_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 39 + 8);
    assert_eq!(
      PatrolPoint::read(&mut ChunkReader::from_slice(file)?.read_child_by_index(0)?)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let config_path: &Path = &GeneratedResources::sample_path(file!(), "import_export.ltx");
    let mut file: File = GeneratedResources::overwrite_path(config_path)?;
    let mut ltx: Ltx = Ltx::new();

    let original: PatrolPoint = PatrolPoint {
      name: String::from("patrol-point-exported"),
      position: Vector3d::new(3.5, -2.3, 6.0),
      flags: 73,
      level_vertex_id: 26543,
      game_vertex_id: 364,
    };

    original.export("data", &mut ltx)?;
    ltx.write_to(&mut file)?;

    assert_eq!(
      PatrolPoint::import("data", &Ltx::read_from_path(config_path)?)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: PatrolPoint = PatrolPoint {
      name: String::from("patrol-point-serialized"),
      position: Vector3d::new(5.5, -2.3, 6.0),
      flags: 53,
      level_vertex_id: 2351,
      game_vertex_id: 321,
    };

    let mut file: File =
      GeneratedResources::overwrite_path(GeneratedResources::sample_path(file!(), "serialize_deserialize.json"))?;

    file.write_all(to_string_pretty(&original).map_serde_err()?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_to_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(
      original,
      serde_json::from_str::<PatrolPoint>(&serialized).map_serde_err()?
    );

    Ok(())
  }
}
