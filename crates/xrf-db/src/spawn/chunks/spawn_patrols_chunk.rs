use std::fmt;
use std::io::Write;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_ltx::Ltx;
use xrf_utils::{assert_length, open_export_file, to_format_size};

use crate::data::patrols::patrol::Patrol;
use crate::export::FileImportExport;

/// `CPatrolPathStorage::load` in xray engine.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SpawnPatrolsChunk {
  pub patrols: Vec<Patrol>,
}

impl SpawnPatrolsChunk {
  pub const CHUNK_ID: u32 = 3;
  pub const META_NESTED_CHUNK_ID: u32 = 0;
  pub const DATA_NESTED_CHUNK_ID: u32 = 1;
}

impl ChunkReadWrite for SpawnPatrolsChunk {
  /// Read patrols list from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    log::info!("Reading patrols chunk, bytes {}", reader.read_bytes_remain());

    let mut meta_reader: ChunkReader<D> = reader.read_child_by_index(Self::META_NESTED_CHUNK_ID)?;
    let mut data_reader: ChunkReader<D> = reader.read_child_by_index(Self::DATA_NESTED_CHUNK_ID)?;

    let count: u32 = meta_reader.read_u32::<T>()?;
    let patrols: Vec<Patrol> = data_reader.read_xr_list::<T, _>()?;

    assert_length(&patrols, count as usize, "Expect defined count of patrols to be read")?;

    meta_reader.assert_read("Expect patrol meta chunk to be ended")?;
    data_reader.assert_read("Expect patrol data chunk to be ended")?;
    reader.assert_read("Expect patrols chunk to be ended")?;

    Ok(Self { patrols })
  }

  /// Write patrols data into chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut meta_writer: ChunkWriter = ChunkWriter::new();
    let mut data_writer: ChunkWriter = ChunkWriter::new();

    meta_writer.write_u32::<T>(to_format_size(self.patrols.len(), "spawn patrols")?)?;
    data_writer.write_xr_list::<T, _>(&self.patrols)?;

    writer.write_all(
      meta_writer
        .flush_chunk_into_buffer::<T>(Self::META_NESTED_CHUNK_ID)?
        .as_slice(),
    )?;
    writer.write_all(
      data_writer
        .flush_chunk_into_buffer::<T>(Self::DATA_NESTED_CHUNK_ID)?
        .as_slice(),
    )?;

    log::info!("Written patrols chunk, {} bytes", writer.bytes_written());

    Ok(())
  }
}

impl FileImportExport for SpawnPatrolsChunk {
  /// Import patrols data from provided path.
  fn import<P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    let patrols_ltx: Ltx = Ltx::read_from_path(path.as_ref().join("patrols.ltx"))?;
    let patrol_points_ltx: Ltx = Ltx::read_from_path(path.as_ref().join("patrol_points.ltx"))?;
    let patrol_links_ltx: Ltx = Ltx::read_from_path(path.as_ref().join("patrol_links.ltx"))?;

    let mut patrols: Vec<Patrol> = Vec::new();

    for section in patrols_ltx.sections() {
      patrols.push(Patrol::import(
        section,
        &patrols_ltx,
        &patrol_points_ltx,
        &patrol_links_ltx,
      )?);
    }

    log::info!("Imported patrols chunk");

    Ok(Self { patrols })
  }

  /// Export patrols data into provided path.
  fn export<P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    let mut patrols_ltx: Ltx = Ltx::new();
    let mut patrol_points_ltx: Ltx = Ltx::new();
    let mut patrol_links_ltx: Ltx = Ltx::new();

    for patrol in &self.patrols {
      patrol.export(
        &patrol.name,
        &mut patrols_ltx,
        &mut patrol_points_ltx,
        &mut patrol_links_ltx,
      )?;
    }

    patrols_ltx.write_to(&mut open_export_file(path.as_ref().join("patrols.ltx"))?)?;
    patrol_points_ltx.write_to(&mut open_export_file(path.as_ref().join("patrol_points.ltx"))?)?;
    patrol_links_ltx.write_to(&mut open_export_file(path.as_ref().join("patrol_links.ltx"))?)?;

    log::info!("Exported patrols chunk");

    Ok(())
  }
}

impl fmt::Debug for SpawnPatrolsChunk {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(formatter, "PatrolsChunk {{ patrols: Vector[{}] }}", self.patrols.len(),)
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

  use crate::data::generic::vector_3d::Vector3d;
  use crate::data::patrols::patrol::Patrol;
  use crate::data::patrols::patrol_link::PatrolLink;
  use crate::data::patrols::patrol_point::PatrolPoint;
  use crate::spawn::chunks::spawn_patrols_chunk::SpawnPatrolsChunk;

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: SpawnPatrolsChunk = SpawnPatrolsChunk {
      patrols: vec![
        Patrol {
          name: String::from("patrol-1"),
          points: vec![
            PatrolPoint {
              name: String::from("patrol-point-1"),
              position: Vector3d::new(1.5, -2.3, 1.0),
              flags: 33,
              level_vertex_id: 250,
              game_vertex_id: 555,
            },
            PatrolPoint {
              name: String::from("patrol-point-2"),
              position: Vector3d::new(2.5, -5.3, 3.0),
              flags: 64,
              level_vertex_id: 5500,
              game_vertex_id: 666,
            },
          ],
          links: vec![PatrolLink {
            index: 0,
            links: vec![(10, 50.5), (15, 60.25)],
          }],
        },
        Patrol {
          name: String::from("patrol-2"),
          points: vec![
            PatrolPoint {
              name: String::from("patrol-point-1"),
              position: Vector3d::new(7.5, -4.3, 3.0),
              flags: 1,
              level_vertex_id: 601,
              game_vertex_id: 541,
            },
            PatrolPoint {
              name: String::from("patrol-point-2"),
              position: Vector3d::new(4.5, -5.3, 3.0),
              flags: 0,
              level_vertex_id: 600,
              game_vertex_id: 542,
            },
          ],
          links: vec![PatrolLink {
            index: 0,
            links: vec![(10, 50.5), (15, 60.25)],
          }],
        },
      ],
    };

    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 450);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 450);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 450 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;
    let read: SpawnPatrolsChunk = SpawnPatrolsChunk::read::<XRayByteOrder, _>(&mut reader)?;

    assert_eq!(read, original);

    Ok(())
  }
}
