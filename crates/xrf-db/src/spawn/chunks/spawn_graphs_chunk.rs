use std::fmt;
use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_ltx::Ltx;
use xrf_utils::{assert_length, open_export_file};

use crate::data::graph::graph_cross_table::GraphCrossTable;
use crate::data::graph::graph_edge::GraphEdge;
use crate::data::graph::graph_header::GraphHeader;
use crate::data::graph::graph_level::GraphLevel;
use crate::data::graph::graph_level_point::GraphLevelPoint;
use crate::data::graph::graph_vertex::GraphVertex;
use crate::export::{FileImportExport, LtxImportExport};

/// `GameGraph::CHeader::load`, `GameGraph::SLevel::load`, `CGameGraph::Initialize`
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnGraphsChunk {
  pub header: GraphHeader,
  pub levels: Vec<GraphLevel>,
  pub vertices: Vec<GraphVertex>,
  pub edges: Vec<GraphEdge>,
  pub points: Vec<GraphLevelPoint>,
  pub cross_tables: Vec<GraphCrossTable>,
}

impl SpawnGraphsChunk {
  pub const CHUNK_ID: u32 = 4;
}

impl ChunkReadWrite for SpawnGraphsChunk {
  /// Read graphs chunk by position descriptor.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    log::info!("Reading graphs chunk, bytes {}", reader.read_bytes_remain());

    let header: GraphHeader = reader.read_xr::<T, _>()?;

    let mut levels: Vec<GraphLevel> = reader.new_bounded_vec(
      header.levels_count.into(),
      GraphLevel::MIN_SERIALIZED_SIZE,
      "graph levels",
    )?;

    for _ in 0..header.levels_count {
      levels.push(reader.read_xr::<T, _>()?)
    }

    let mut vertices: Vec<GraphVertex> = reader.new_bounded_vec(
      header.vertices_count.into(),
      GraphVertex::MIN_SERIALIZED_SIZE,
      "graph vertices",
    )?;

    for _ in 0..header.vertices_count {
      vertices.push(reader.read_xr::<T, _>()?);
    }

    let mut edges: Vec<GraphEdge> =
      reader.new_bounded_vec(header.edges_count.into(), GraphEdge::MIN_SERIALIZED_SIZE, "graph edges")?;

    for _ in 0..header.edges_count {
      edges.push(reader.read_xr::<T, _>()?);
    }

    let mut points: Vec<GraphLevelPoint> = reader.new_bounded_vec(
      header.points_count.into(),
      GraphLevelPoint::MIN_SERIALIZED_SIZE,
      "graph level points",
    )?;

    for _ in 0..header.points_count {
      points.push(reader.read_xr::<T, _>()?);
    }

    let cross_tables: Vec<GraphCrossTable> = reader.read_xr_list::<T, _>()?;

    log::info!("Read graphs ver {}, {} bytes", header.version, reader.read_bytes_len(),);

    assert_length(
      &levels,
      header.levels_count as usize,
      "Expect correct levels count to be read",
    )?;
    assert_length(
      &vertices,
      header.vertices_count as usize,
      "Expect correct vertices count to be read",
    )?;
    assert_length(
      &edges,
      header.edges_count as usize,
      "Expect correct edges count to be read",
    )?;
    assert_length(
      &points,
      header.points_count as usize,
      "Expect correct points count to be read",
    )?;
    reader.assert_read("Expect graphs chunk to be ended")?;

    Ok(Self {
      header,
      levels,
      vertices,
      edges,
      points,
      cross_tables,
    })
  }

  /// Write whole graphs chunk into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.header.write::<T>(writer)?;

    for level in &self.levels {
      writer.write_xr::<T, _>(level)?;
    }

    for vertex in &self.vertices {
      writer.write_xr::<T, _>(vertex)?;
    }

    for edge in &self.edges {
      writer.write_xr::<T, _>(edge)?;
    }

    for point in &self.points {
      writer.write_xr::<T, _>(point)?;
    }

    writer.write_xr_list::<T, _>(&self.cross_tables)?;

    log::info!("Written graphs chunk, {} bytes", writer.bytes_written());

    Ok(())
  }
}

impl FileImportExport for SpawnGraphsChunk {
  /// Import graphs data from provided path.
  fn import<P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    let header: GraphHeader =
      GraphHeader::import("header", &Ltx::read_from_path(path.as_ref().join("graphs_header.ltx"))?)?;

    let levels_ltx: Ltx = Ltx::read_from_path(path.as_ref().join("graphs_levels.ltx"))?;
    let mut levels: Vec<GraphLevel> = Vec::new();

    for index in 0..header.levels_count {
      levels.push(GraphLevel::import(&index.to_string(), &levels_ltx)?);
    }

    let vertices_ltx: Ltx = Ltx::read_from_path(path.as_ref().join("graphs_vertices.ltx"))?;
    let mut vertices: Vec<GraphVertex> = Vec::new();

    for index in 0..header.vertices_count {
      vertices.push(GraphVertex::import(&index.to_string(), &vertices_ltx)?);
    }

    let points_ltx: Ltx = Ltx::read_from_path(path.as_ref().join("graphs_points.ltx"))?;
    let mut points: Vec<GraphLevelPoint> = Vec::new();

    for index in 0..header.points_count {
      points.push(GraphLevelPoint::import(&index.to_string(), &points_ltx)?);
    }

    let edges_ltx: Ltx = Ltx::read_from_path(path.as_ref().join("graphs_edges.ltx"))?;
    let mut edges: Vec<GraphEdge> = Vec::new();

    for index in 0..header.edges_count {
      edges.push(GraphEdge::import(&index.to_string(), &edges_ltx)?);
    }

    let cross_tables: Vec<GraphCrossTable> =
      GraphCrossTable::import_list::<XRayByteOrder>(&mut File::open(path.as_ref().join("graphs_cross_tables.gct"))?)?;

    log::info!("Imported graphs chunk");

    Ok(Self {
      header,
      levels,
      vertices,
      edges,
      points,
      cross_tables,
    })
  }

  /// Export graphs data into provided path.
  /// Constructs many files with contained data.
  fn export<P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    let mut graphs_header_ltx: Ltx = Ltx::new();

    self.header.export("header", &mut graphs_header_ltx)?;

    graphs_header_ltx.write_to(&mut open_export_file(path.as_ref().join("graphs_header.ltx"))?)?;

    let mut graphs_level_ltx: Ltx = Ltx::new();

    for (index, level) in self.levels.iter().enumerate() {
      level.export(&index.to_string(), &mut graphs_level_ltx)?;
    }

    graphs_level_ltx.write_to(&mut open_export_file(path.as_ref().join("graphs_levels.ltx"))?)?;

    log::info!("Exported graph levels");

    let mut graphs_vertices_ltx: Ltx = Ltx::new();

    for (index, vertex) in self.vertices.iter().enumerate() {
      vertex.export(&index.to_string(), &mut graphs_vertices_ltx)?;
    }

    graphs_vertices_ltx.write_to(&mut open_export_file(path.as_ref().join("graphs_vertices.ltx"))?)?;

    log::info!("Exported graph vertices");

    let mut graphs_points_ltx: Ltx = Ltx::new();

    for (index, point) in self.points.iter().enumerate() {
      point.export(&index.to_string(), &mut graphs_points_ltx)?;
    }

    graphs_points_ltx.write_to(&mut open_export_file(path.as_ref().join("graphs_points.ltx"))?)?;

    log::info!("Exported graph points");

    let mut graphs_edges_ltx: Ltx = Ltx::new();

    for (index, edge) in self.edges.iter().enumerate() {
      edge.export(&index.to_string(), &mut graphs_edges_ltx)?;
    }

    graphs_edges_ltx.write_to(&mut open_export_file(path.as_ref().join("graphs_edges.ltx"))?)?;

    log::info!("Exported graph edges");

    GraphCrossTable::export_list::<XRayByteOrder>(
      &self.cross_tables,
      &mut open_export_file(path.as_ref().join("graphs_cross_tables.gct"))?,
    )?;

    log::info!("Exported graph cross tables");

    log::info!("Exported graphs chunk");

    Ok(())
  }
}

impl fmt::Debug for SpawnGraphsChunk {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      formatter,
      "GraphsChunk {{ header: {:?}, levels: Vector[{}], vertices: Vector[{}] }}",
      self.header,
      self.levels.len(),
      self.vertices.len(),
    )
  }
}

#[cfg(test)]
mod tests {
  use uuid::uuid;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::data::generic::vector_3d::Vector3d;
  use crate::data::graph::graph_cross_table::GraphCrossTable;
  use crate::data::graph::graph_edge::GraphEdge;
  use crate::data::graph::graph_header::GraphHeader;
  use crate::data::graph::graph_level::GraphLevel;
  use crate::data::graph::graph_level_point::GraphLevelPoint;
  use crate::data::graph::graph_vertex::GraphVertex;
  use crate::spawn::chunks::spawn_graphs_chunk::SpawnGraphsChunk;

  #[test]
  fn test_read_write_empty() -> XrfResult {
    let filename: String = String::from("read_write_empty.chunk");

    let original: SpawnGraphsChunk = SpawnGraphsChunk {
      header: GraphHeader {
        version: 10,
        vertices_count: 0,
        edges_count: 0,
        points_count: 0,
        guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
        levels_count: 0,
      },
      levels: vec![],
      vertices: vec![],
      edges: vec![],
      points: vec![],
      cross_tables: vec![],
    };

    let mut writer: ChunkWriter = ChunkWriter::new();

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

    assert_eq!(SpawnGraphsChunk::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");

    let original: SpawnGraphsChunk = SpawnGraphsChunk {
      header: GraphHeader {
        version: 12,
        vertices_count: 2,
        edges_count: 4,
        points_count: 3,
        guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
        levels_count: 2,
      },
      levels: vec![
        GraphLevel {
          id: 25,
          name: String::from("test-level-1"),
          section: String::from("test-level-section-1"),
          guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
          offset: Vector3d::new(2.5, 4.55, -6.5),
        },
        GraphLevel {
          id: 26,
          name: String::from("test-level"),
          section: String::from("test-level-section"),
          guid: uuid!("89e55023-10b1-426f-9247-bb680e5fe0c8"),
          offset: Vector3d::new(3.5, 5.55, -7.5),
        },
      ],
      vertices: vec![
        GraphVertex {
          level_point: Vector3d::new(12.5, 11.6, 12.3),
          game_point: Vector3d::new(1.5, -4.0, 1000.0),
          level_id: 255,
          level_vertex_id: 4000,
          vertex_type: (1, 2, 3, 4).into(),
          edges_offset: 540,
          level_points_offset: 4000,
          edges_count: 252,
          level_points_count: 253,
        },
        GraphVertex {
          level_point: Vector3d::new(43.5, 15.6, 0.3),
          game_point: Vector3d::new(0.5, -4.0, 44.0),
          level_id: 255,
          level_vertex_id: 3000,
          vertex_type: (4, 2, 4, 4).into(),
          edges_offset: 31,
          level_points_offset: 623,
          edges_count: 252,
          level_points_count: 23,
        },
      ],
      edges: vec![
        GraphEdge {
          game_vertex_id: 713,
          distance: 21.50,
        },
        GraphEdge {
          game_vertex_id: 714,
          distance: 8443.50,
        },
        GraphEdge {
          game_vertex_id: 715,
          distance: 4.50,
        },
        GraphEdge {
          game_vertex_id: 716,
          distance: 3.0,
        },
      ],
      points: vec![
        GraphLevelPoint {
          position: Vector3d::new(1.5, 11.6, 12.7),
          distance: 100.50,
          level_vertex_id: 8000,
        },
        GraphLevelPoint {
          position: Vector3d::new(2.5, 11.6, 12.7),
          distance: 200.50,
          level_vertex_id: 8001,
        },
        GraphLevelPoint {
          position: Vector3d::new(3.5, 11.6, 12.7),
          distance: 300.50,
          level_vertex_id: 8002,
        },
      ],
      cross_tables: vec![
        GraphCrossTable {
          version: 16,
          nodes_count: 51,
          vertices_count: 4000,
          level_guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
          game_guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b8"),
          data: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        },
        GraphCrossTable {
          version: 16,
          nodes_count: 4232,
          vertices_count: 3000,
          level_guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
          game_guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b8"),
          data: vec![1, 2, 3, 4, 5],
        },
      ],
    };

    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 430);

    let bytes_written: usize = writer.flush_chunk_into::<XRayByteOrder>(
      &mut overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?,
      0,
    )?;

    assert_eq!(bytes_written, 430);

    let file: FileSlice =
      open_generated_test_resource_as_slice(&build_relative_test_sample_file_path(file!(), &filename))?;

    assert_eq!(file.bytes_remaining(), 430 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(SpawnGraphsChunk::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  /// A graphs header declaring the supplied collection counts and nothing else.
  fn mock_graphs_header(vertices_count: u16, edges_count: u32, points_count: u32, levels_count: u8) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![10];

    bytes.extend(vertices_count.to_le_bytes());
    bytes.extend(edges_count.to_le_bytes());
    bytes.extend(points_count.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.push(levels_count);

    bytes
  }

  fn read_graphs_error(bytes: &[u8]) -> XrfResult<String> {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(bytes)?;

    Ok(
      SpawnGraphsChunk::read::<XRayByteOrder, _>(&mut reader)
        .expect_err("expect the declared count to exceed the chunk")
        .to_string(),
    )
  }

  #[test]
  fn rejects_level_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    let error: String = read_graphs_error(&mock_graphs_header(0, 0, 0, 255))?;

    assert!(
      error.contains("graph levels declares 255 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }

  #[test]
  fn rejects_vertex_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    let error: String = read_graphs_error(&mock_graphs_header(65535, 0, 0, 0))?;

    assert!(
      error.contains("graph vertices declares 65535 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }

  #[test]
  fn rejects_edge_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    let error: String = read_graphs_error(&mock_graphs_header(0, u32::MAX, 0, 0))?;

    assert!(
      error.contains("graph edges declares 4294967295 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }

  #[test]
  fn rejects_level_point_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    let error: String = read_graphs_error(&mock_graphs_header(0, 0, u32::MAX, 0))?;

    assert!(
      error.contains("graph level points declares 4294967295 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }
}
