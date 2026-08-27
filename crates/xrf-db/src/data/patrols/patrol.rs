use std::io::Write;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkIterator, ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};
use xrf_utils::{assert_equal, assert_length};

use crate::data::patrols::patrol_link::PatrolLink;
use crate::data::patrols::patrol_point::PatrolPoint;
use crate::export::LtxImportExport;
use crate::file_import::read_ltx_field;

/// Patrols list is represented by list of samples containing patrol chunk.
/// 0...N, where N is chunk.
///
/// `CPatrolPathStorage::load`, `CPatrolPath::load_raw` in xray codebase.
///
/// Patrol chunk has the following structure:
/// 0 - metadata
///   - name
/// 1 - data
///     0 - points count
///     1 - patrol points
///     2 - patrol points links
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patrol {
  pub name: String,
  pub points: Vec<PatrolPoint>,
  pub links: Vec<PatrolLink>,
}

impl Patrol {
  pub const META_CHUNK_ID: u32 = 0;
  pub const DATA_CHUNK_ID: u32 = 1;
  pub const DATA_POINT_COUNT_CHUNK_ID: u32 = 0;
  pub const DATA_POINT_DATA_CHUNK_ID: u32 = 1;
  pub const DATA_LIST_CHUNK_ID: u32 = 2;
}

impl ChunkReadWriteList for Patrol {
  /// Read chunk as list of patrol samples.
  fn read_list<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<Self>> {
    let mut patrols: Vec<Self> = Vec::new();

    for patrol_reader in ChunkIterator::from_start(reader)? {
      let mut patrol_reader: ChunkReader<D> = patrol_reader?;

      patrols.push(Self::read::<T, _>(&mut patrol_reader)?);
    }

    reader.assert_read("Chunk data should be read for patrols list")?;

    Ok(patrols)
  }

  /// Write list of patrols into chunk writer.
  fn write_list<T: ByteOrder>(writer: &mut ChunkWriter, list: &[Self]) -> XrfResult {
    for (index, patrol) in list.iter().enumerate() {
      let mut patrol_writer: ChunkWriter = ChunkWriter::new();

      patrol.write::<T>(&mut patrol_writer)?;

      writer.write_all(&patrol_writer.flush_chunk_into_buffer::<T>(index as u32)?)?;
    }

    Ok(())
  }
}

impl ChunkReadWrite for Patrol {
  /// Read chunk as patrol.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut meta_reader: ChunkReader<D> = reader.read_child_by_index(Self::META_CHUNK_ID)?;
    let mut data_reader: ChunkReader<D> = reader.read_child_by_index(Self::DATA_CHUNK_ID)?;

    let mut point_count_reader: ChunkReader<D> = data_reader.read_child_by_index(Self::DATA_POINT_COUNT_CHUNK_ID)?;
    let mut points_reader: ChunkReader<D> = data_reader.read_child_by_index(Self::DATA_POINT_DATA_CHUNK_ID)?;
    let mut links_reader: ChunkReader<D> = data_reader.read_child_by_index(Self::DATA_LIST_CHUNK_ID)?;

    let name: String = meta_reader.read_w1251_string()?;

    assert_equal(
      name.len() + 1,
      meta_reader.size as usize,
      "Expect correct patrol name data to be read",
    )?; // Count null termination char.

    let points_count: u32 = point_count_reader.read_u32::<T>()?;
    let points: Vec<PatrolPoint> = points_reader.read_xr_list::<T, _>()?;
    let links: Vec<PatrolLink> = links_reader.read_xr_list::<T, _>()?;

    assert_length(
      &points,
      points_count as usize,
      "Expected defined count of patrol points to be read",
    )?;
    reader.assert_read("Expect patrol chunk to be ended")?;

    Ok(Self { name, points, links })
  }

  /// Write single patrol entity into chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut meta_writer: ChunkWriter = ChunkWriter::new();
    let mut data_writer: ChunkWriter = ChunkWriter::new();

    let mut point_count_writer: ChunkWriter = ChunkWriter::new();
    let mut points_writer: ChunkWriter = ChunkWriter::new();
    let mut links_writer: ChunkWriter = ChunkWriter::new();

    meta_writer.write_w1251_string(&self.name)?;
    writer.write_all(&meta_writer.flush_chunk_into_buffer::<T>(Self::META_CHUNK_ID)?)?;

    point_count_writer.write_u32::<T>(self.points.len() as u32)?;
    data_writer.write_all(&point_count_writer.flush_chunk_into_buffer::<T>(Self::DATA_POINT_COUNT_CHUNK_ID)?)?;

    points_writer.write_xr_list::<T, _>(&self.points)?;
    data_writer.write_all(&points_writer.flush_chunk_into_buffer::<T>(Self::DATA_POINT_DATA_CHUNK_ID)?)?;

    links_writer.write_xr_list::<T, _>(&self.links)?;
    data_writer.write_all(&links_writer.flush_chunk_into_buffer::<T>(Self::DATA_LIST_CHUNK_ID)?)?;

    writer.write_all(&data_writer.flush_chunk_into_buffer::<T>(Self::DATA_CHUNK_ID)?)?;

    Ok(())
  }
}

// todo: Generic import-export impl.
impl Patrol {
  /// Import patrols data from provided path.
  pub fn import(
    section_name: &str,
    patrols_ltx: &Ltx,
    patrol_points_ltx: &Ltx,
    patrol_links_ltx: &Ltx,
  ) -> XrfResult<Self> {
    let section: &Section = patrols_ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Patrol section '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    let name: String = read_ltx_field("name", section)?;
    let points_list: String = read_ltx_field("points", section)?;
    let links_count: usize = read_ltx_field("links_count", section)?;

    let mut points: Vec<PatrolPoint> = Vec::new();
    let mut links: Vec<PatrolLink> = Vec::with_capacity(links_count);

    for (index, listed) in points_list.split(',').map(str::trim).enumerate() {
      let prefix: String = format!("{name}.{index}.");
      let exact: String = format!("{prefix}{listed}");

      // Ltx trims values, so a point name padded with whitespace survives only in the section name. The listed name is
      // what the trimmed name looks like; the section it belongs to carries the real one.
      let section_name: String = if patrol_points_ltx.has_section(&exact) {
        exact
      } else {
        let padded: String = patrol_points_ltx
          .sections()
          .find(|section| {
            section
              .strip_prefix(prefix.as_str())
              .is_some_and(|point| point.trim() == listed)
          })
          .map(String::from)
          .ok_or_else(|| {
            XrfError::new_parsing_error(format!(
              "Patrol point section '{}' should be defined in ltx file ({})",
              exact,
              file!()
            ))
          })?;

        log::warn!(
          "Patrol point '{}' of patrol '{}' recovered from padded section '{}'",
          listed,
          name,
          padded
        );

        padded
      };

      points.push(PatrolPoint::import_named(
        &section_name,
        String::from(&section_name[prefix.len()..]),
        patrol_points_ltx,
      )?);
    }

    for index in 0..links_count {
      links.push(PatrolLink::import(&format!("{}.{}", name, index), patrol_links_ltx)?);
    }

    assert_length(&links, links_count, "Expect defined count of patrols to be imported")?;

    Ok(Self { name, points, links })
  }

  /// Export patrol data into ltx config files.
  /// Creates separate files for patrols, points and links.
  pub fn export(
    &self,
    section_name: &str,
    patrols_ltx: &mut Ltx,
    patrol_points_ltx: &mut Ltx,
    patrol_links_ltx: &mut Ltx,
  ) -> XrfResult {
    patrols_ltx
      .with_section(section_name)
      .set("name", &self.name)
      .set(
        "points",
        self
          .points
          .iter()
          .map(|it| it.name.clone())
          .collect::<Vec<String>>()
          .join(","),
      )
      .set("links_count", self.links.len().to_string());

    for (index, point) in self.points.iter().enumerate() {
      point.export(&format!("{}.{}.{}", self.name, index, point.name), patrol_points_ltx)?;
    }

    for (index, link) in self.links.iter().enumerate() {
      link.export(&format!("{}.{}", self.name, index), patrol_links_ltx)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write};
  use std::path::Path;

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_sample_file_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_file, overwrite_generated_test_resource_as_file,
  };

  use crate::data::generic::vector_3d::Vector3d;
  use crate::data::patrols::patrol::Patrol;
  use crate::data::patrols::patrol_link::PatrolLink;
  use crate::data::patrols::patrol_point::PatrolPoint;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: Patrol = Patrol {
      name: String::from("patrol-name"),
      points: vec![
        PatrolPoint {
          name: String::from("patrol-point-1"),
          position: Vector3d::new(7.5, -2.3, -100.0),
          flags: 33,
          level_vertex_id: 63463634,
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
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 210);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 210);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 210 + 8);
    assert_eq!(
      Patrol::read::<XRayByteOrder, _>(&mut ChunkReader::from_slice(file)?.read_child_by_index(0)?)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_read_write_list() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write_list.chunk");

    let original: Vec<Patrol> = vec![
      Patrol {
        name: String::from("patrol-1"),
        points: vec![
          PatrolPoint {
            name: String::from("patrol-point-1"),
            position: Vector3d::new(10.5, -20.3, 10.0),
            flags: 33,
            level_vertex_id: 250,
            game_vertex_id: 555,
          },
          PatrolPoint {
            name: String::from("patrol-point-2"),
            position: Vector3d::new(20.5, -50.3, 30.0),
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
            position: Vector3d::new(70.5, -40.3, 30.0),
            flags: 1,
            level_vertex_id: 601,
            game_vertex_id: 541,
          },
          PatrolPoint {
            name: String::from("patrol-point-2"),
            position: Vector3d::new(20.5, -50.3, 30.0),
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
    ];

    Patrol::write_list::<XRayByteOrder>(&mut writer, &original)?;

    assert_eq!(writer.bytes_written(), 430);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 430);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 430 + 8);
    assert_eq!(
      Patrol::read_list::<XRayByteOrder, _>(&mut ChunkReader::from_slice(file)?.read_child_by_index(0)?)?,
      original
    );

    Ok(())
  }

  /// Anomaly ships patrol points whose names carry trailing whitespace. Ltx trims values, so the name survives only
  /// in the section it was exported under, and importing has to take it from there or the repack loses those bytes.
  #[test]
  fn test_import_export_preserves_padded_point_names() -> XrfResult {
    let original: Patrol = Patrol {
      name: String::from("depo_terrain_camper_2_walk"),
      points: vec![PatrolPoint {
        name: String::from("wp00 "),
        position: Vector3d::new(428.2036, 9.95521, -73.75238),
        flags: 1,
        level_vertex_id: 1393284,
        game_vertex_id: 4553,
      }],
      links: vec![],
    };

    let patrol_config_path: &Path = &build_absolute_generated_test_sample_file_path(file!(), "padded_names.ltx");
    let points_config_path: &Path = &build_absolute_generated_test_sample_file_path(file!(), "padded_names_points.ltx");
    let links_config_path: &Path = &build_absolute_generated_test_sample_file_path(file!(), "padded_names_links.ltx");

    let mut patrol_ltx: Ltx = Ltx::new();
    let mut points_ltx: Ltx = Ltx::new();
    let mut links_ltx: Ltx = Ltx::new();

    original.export(&original.name, &mut patrol_ltx, &mut points_ltx, &mut links_ltx)?;

    patrol_ltx.write_to(&mut overwrite_file(patrol_config_path)?)?;
    points_ltx.write_to(&mut overwrite_file(points_config_path)?)?;
    links_ltx.write_to(&mut overwrite_file(links_config_path)?)?;

    let imported: Patrol = Patrol::import(
      &original.name,
      &Ltx::read_from_path(patrol_config_path)?,
      &Ltx::read_from_path(points_config_path)?,
      &Ltx::read_from_path(links_config_path)?,
    )?;

    assert_eq!(imported.points.first().unwrap().name, "wp00 ");
    assert_eq!(imported, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: Patrol = Patrol {
      name: String::from("patrol-name-exp"),
      points: vec![
        PatrolPoint {
          name: String::from("patrol-point-1-exp"),
          position: Vector3d::new(7.5, -2.42, -4.0),
          flags: 53,
          level_vertex_id: 2533,
          game_vertex_id: 512,
        },
        PatrolPoint {
          name: String::from("patrol-point-2-exp"),
          position: Vector3d::new(4.5, -5.3, 4.5),
          flags: 12,
          level_vertex_id: 23421,
          game_vertex_id: 5233,
        },
        PatrolPoint {
          name: String::from("patrol-point-1-exp"),
          position: Vector3d::new(3.5, -7.3, 1.5),
          flags: 6,
          level_vertex_id: 23421,
          game_vertex_id: 5233,
        },
      ],
      links: vec![PatrolLink {
        index: 0,
        links: vec![(22, 34.5), (24, 553.25)],
      }],
    };

    let patrol_config_path: &Path = &build_absolute_generated_test_sample_file_path(file!(), "import_export.ltx");
    let points_config_path: &Path =
      &build_absolute_generated_test_sample_file_path(file!(), "import_export_points.ltx");
    let links_config_path: &Path = &build_absolute_generated_test_sample_file_path(file!(), "import_export_links.ltx");

    let mut patrol_file: File = overwrite_file(patrol_config_path)?;
    let mut points_file: File = overwrite_file(points_config_path)?;
    let mut links_file: File = overwrite_file(links_config_path)?;

    let mut patrol_ltx: Ltx = Ltx::new();
    let mut links_ltx: Ltx = Ltx::new();
    let mut points_ltx: Ltx = Ltx::new();

    original.export(&original.name, &mut patrol_ltx, &mut points_ltx, &mut links_ltx)?;

    patrol_ltx.write_to(&mut patrol_file)?;
    points_ltx.write_to(&mut points_file)?;
    links_ltx.write_to(&mut links_file)?;

    assert_eq!(
      Patrol::import(
        &original.name,
        &Ltx::read_from_path(patrol_config_path)?,
        &Ltx::read_from_path(points_config_path)?,
        &Ltx::read_from_path(links_config_path)?,
      )?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: Patrol = Patrol {
      name: String::from("patrol-name-serde"),
      points: vec![
        PatrolPoint {
          name: String::from("patrol-point-1-serde"),
          position: Vector3d::new(4.5, -5.42, -3.2),
          flags: 83,
          level_vertex_id: 4657,
          game_vertex_id: 457,
        },
        PatrolPoint {
          name: String::from("patrol-point-2-serde"),
          position: Vector3d::new(6.21, -5.34, 3.23),
          flags: 53,
          level_vertex_id: 6345,
          game_vertex_id: 15211,
        },
      ],
      links: vec![PatrolLink {
        index: 0,
        links: vec![(32, 34.5), (24, 53.25)],
      }],
    };

    let mut file: File = overwrite_file(build_absolute_generated_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<Patrol>(&serialized)?);

    Ok(())
  }
}
