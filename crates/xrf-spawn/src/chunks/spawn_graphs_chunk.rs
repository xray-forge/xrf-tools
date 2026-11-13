use std::fmt;
use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{FileImportExport, Ltx, LtxImportExport, LtxRecordReader, LtxRecordWriter};
use xrf_utils::{StagedFile, assert_length};

use crate::data::graph::graph_cross_table::GraphCrossTable;
use crate::data::graph::graph_edge::GraphEdge;
use crate::data::graph::graph_header::GraphHeader;
use crate::data::graph::graph_level::GraphLevel;
use crate::data::graph::graph_level_point::GraphLevelPoint;
use crate::data::graph::graph_vertex::GraphVertex;

/// One of a graph's tables after its vertices, its edges or its level points: where it spans, bytes from the first
/// vertex, and the size of each of its records.
struct GraphTable {
  start: u64,
  end: u64,
  record: u64,
}

impl GraphTable {
  /// Whether `count` records from `offset` start on a record of the table and end within it.
  fn holds(&self, offset: u32, count: u8) -> bool {
    let start: u64 = u64::from(offset);

    start >= self.start
      && start + u64::from(count) * self.record <= self.end
      && (start - self.start).is_multiple_of(self.record)
  }
}

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

  /// The largest node id a vertex stores, in 24 bits.
  pub const MAX_LEVEL_VERTEX_ID: u32 = (1 << 24) - 1;

  /// Lays the graph out as `CGameGraph` stores it, from what it holds: the header's counts, and each vertex's edge and
  /// level point offsets, bytes from the first vertex, its edges and level points following those of the vertices
  /// before it. A graph assembled in memory needs no offset set by hand.
  ///
  /// # Errors
  ///
  /// Returns an error when the graph holds more vertices, edges, level points or levels than the header counts.
  pub fn derive_layout(&mut self) -> XrfResult {
    self.header.vertices_count = Self::narrow(self.vertices.len() as u64, "game graph vertices")?;
    self.header.edges_count = Self::narrow(self.edges.len() as u64, "game graph edges")?;
    self.header.points_count = Self::narrow(self.points.len() as u64, "game graph level points")?;
    self.header.levels_count = Self::narrow(self.levels.len() as u64, "game graph levels")?;

    let [edge_table, point_table] = self.get_tables();
    let (mut edges, mut points): (u64, u64) = (0, 0);

    for vertex in &mut self.vertices {
      vertex.edges_offset = Self::narrow(edge_table.start + edges * edge_table.record, "edge offset")?;
      vertex.level_points_offset = Self::narrow(point_table.start + points * point_table.record, "level point offset")?;
      edges += u64::from(vertex.edges_count);
      points += u64::from(vertex.level_points_count);
    }

    if edges != self.edges.len() as u64 || points != self.points.len() as u64 {
      return Err(XrfError::new_invalid_error(format!(
        "The game graph's vertices count {edges} edges and {points} level points, but it holds {} and {}",
        self.edges.len(),
        self.points.len()
      )));
    }

    Ok(())
  }

  /// Refuses a graph the engine would read wrong: an edge to a vertex past the graph, a vertex on no level of the
  /// header or past the node ids its 24 bits hold, edges or level points off their tables' records, and cross tables
  /// that are not one a level in ascending level id order (`CGameGraph::set_current_level`), each naming its level and
  /// this graph, sized to the level's vertices, every cell's vertex on that level.
  ///
  /// # Errors
  ///
  /// Returns an error naming the first disagreement.
  pub fn assert_engine_layout(&self) -> XrfResult {
    let vertices: u64 = self.vertices.len() as u64;
    let [edge_table, point_table] = self.get_tables();
    let invalid = |message: String| Err(XrfError::new_invalid_error(message));

    for (index, edge) in self.edges.iter().enumerate() {
      if u64::from(edge.game_vertex_id) >= vertices {
        return invalid(format!(
          "Game graph edge {index} leads to vertex {} of {vertices}",
          edge.game_vertex_id
        ));
      }
    }

    for (index, vertex) in self.vertices.iter().enumerate() {
      if !self.levels.iter().any(|level| level.id == vertex.level_id) {
        return invalid(format!(
          "Game vertex {index} stands on level {}, which the graph lacks",
          vertex.level_id
        ));
      }

      if vertex.level_vertex_id > Self::MAX_LEVEL_VERTEX_ID {
        return invalid(format!(
          "Game vertex {index} names node {}, past 24 bits",
          vertex.level_vertex_id
        ));
      }

      if !edge_table.holds(vertex.edges_offset, vertex.edges_count) {
        return invalid(format!("Game vertex {index}'s edges lie off the edge table"));
      }

      if !point_table.holds(vertex.level_points_offset, vertex.level_points_count) {
        return invalid(format!(
          "Game vertex {index}'s level points lie off the level point table"
        ));
      }
    }

    if self.cross_tables.len() != self.levels.len() {
      return invalid(format!(
        "The game graph holds {} levels but {} cross tables",
        self.levels.len(),
        self.cross_tables.len()
      ));
    }

    let mut ranked: Vec<&GraphLevel> = self.levels.iter().collect();

    ranked.sort_by_key(|level| level.id);

    for (level, table) in ranked.into_iter().zip(&self.cross_tables) {
      let level_vertices: u32 = self
        .vertices
        .iter()
        .filter(|vertex| vertex.level_id == level.id)
        .count() as u32;

      if table.level_guid != level.guid || table.game_guid != self.header.guid {
        return invalid(format!(
          "The cross table of level {} names level {} and graph {}, not {} and {}",
          level.name, table.level_guid, table.game_guid, level.guid, self.header.guid
        ));
      }

      if table.vertices_count != level_vertices {
        return invalid(format!(
          "The cross table of level {} counts {} vertices, the level holds {level_vertices}",
          level.name, table.vertices_count
        ));
      }

      if let Some((node, cell)) = table.cells.iter().enumerate().find(|(_, cell)| {
        self
          .vertices
          .get(cell.game_vertex_id as usize)
          .is_none_or(|vertex| vertex.level_id != level.id)
      }) {
        return invalid(format!(
          "Node {node} of level {} belongs to game vertex {}, not one of the level's",
          level.name, cell.game_vertex_id
        ));
      }
    }

    Ok(())
  }

  /// The edge table and the level point table, after the vertices. Every record of the three is of fixed size, its
  /// `MIN_SERIALIZED_SIZE`.
  fn get_tables(&self) -> [GraphTable; 2] {
    let edges: u64 = self.vertices.len() as u64 * GraphVertex::MIN_SERIALIZED_SIZE;
    let points: u64 = edges + self.edges.len() as u64 * GraphEdge::MIN_SERIALIZED_SIZE;

    [
      GraphTable {
        start: edges,
        end: points,
        record: GraphEdge::MIN_SERIALIZED_SIZE,
      },
      GraphTable {
        start: points,
        end: points + self.points.len() as u64 * GraphLevelPoint::MIN_SERIALIZED_SIZE,
        record: GraphLevelPoint::MIN_SERIALIZED_SIZE,
      },
    ]
  }

  /// A count or offset in the width the format stores it.
  fn narrow<T: TryFrom<u64>>(value: u64, what: &str) -> XrfResult<T> {
    T::try_from(value).map_err(|_| XrfError::new_invalid_error(format!("Too many {what} for the game graph's format")))
  }

  /// Refuse a header whose counts are not what the graph holds: `CGameGraph` finds each table by the counts before it.
  fn assert_header_counts(&self) -> XrfResult {
    assert_length(
      &self.levels,
      self.header.levels_count as usize,
      "Expect correct levels count to be read",
    )?;
    assert_length(
      &self.vertices,
      self.header.vertices_count as usize,
      "Expect correct vertices count to be read",
    )?;
    assert_length(
      &self.edges,
      self.header.edges_count as usize,
      "Expect correct edges count to be read",
    )?;
    assert_length(
      &self.points,
      self.header.points_count as usize,
      "Expect correct points count to be read",
    )
  }
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
      levels.push(reader.read_xr::<T, _>()?);
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

    let graphs: Self = Self {
      header,
      levels,
      vertices,
      edges,
      points,
      cross_tables,
    };

    graphs.assert_header_counts()?;
    reader.assert_read("Expect graphs chunk to be ended")?;

    Ok(graphs)
  }

  /// Write whole graphs chunk into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.assert_header_counts()?;
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

    let levels: Vec<GraphLevel> =
      LtxRecordReader::import_indexed(path.as_ref().join("graphs_levels.ltx"), header.levels_count as usize)?;
    let vertices: Vec<GraphVertex> = LtxRecordReader::import_indexed(
      path.as_ref().join("graphs_vertices.ltx"),
      header.vertices_count as usize,
    )?;
    let points: Vec<GraphLevelPoint> =
      LtxRecordReader::import_indexed(path.as_ref().join("graphs_points.ltx"), header.points_count as usize)?;
    let edges: Vec<GraphEdge> =
      LtxRecordReader::import_indexed(path.as_ref().join("graphs_edges.ltx"), header.edges_count as usize)?;

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
    let mut graphs_header_ltx: LtxRecordWriter = LtxRecordWriter::new();

    self.header.export("header", &mut graphs_header_ltx)?;

    graphs_header_ltx.write_to_path(path.as_ref().join("graphs_header.ltx"))?;

    let mut graphs_level_ltx: LtxRecordWriter = LtxRecordWriter::new();

    for (index, level) in self.levels.iter().enumerate() {
      level.export(&index.to_string(), &mut graphs_level_ltx)?;
    }

    graphs_level_ltx.write_to_path(path.as_ref().join("graphs_levels.ltx"))?;

    log::info!("Exported graph levels");

    let mut graphs_vertices_ltx: LtxRecordWriter = LtxRecordWriter::new();

    for (index, vertex) in self.vertices.iter().enumerate() {
      vertex.export(&index.to_string(), &mut graphs_vertices_ltx)?;
    }

    graphs_vertices_ltx.write_to_path(path.as_ref().join("graphs_vertices.ltx"))?;

    log::info!("Exported graph vertices");

    let mut graphs_points_ltx: LtxRecordWriter = LtxRecordWriter::new();

    for (index, point) in self.points.iter().enumerate() {
      point.export(&index.to_string(), &mut graphs_points_ltx)?;
    }

    graphs_points_ltx.write_to_path(path.as_ref().join("graphs_points.ltx"))?;

    log::info!("Exported graph points");

    let mut graphs_edges_ltx: LtxRecordWriter = LtxRecordWriter::new();

    for (index, edge) in self.edges.iter().enumerate() {
      edge.export(&index.to_string(), &mut graphs_edges_ltx)?;
    }

    graphs_edges_ltx.write_to_path(path.as_ref().join("graphs_edges.ltx"))?;

    log::info!("Exported graph edges");

    let mut cross_tables: Vec<u8> = Vec::new();

    GraphCrossTable::export_list::<XRayByteOrder>(&self.cross_tables, &mut cross_tables)?;
    StagedFile::write_creating_directories(&path.as_ref().join("graphs_cross_tables.gct"), &cross_tables)?;

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
  use xrf_math::Vector3d;
  use xrf_test_utils::{FileSlice, GeneratedResources};

  use crate::chunks::spawn_graphs_chunk::SpawnGraphsChunk;
  use crate::data::graph::graph_cross_table::GraphCrossTable;
  use crate::data::graph::graph_cross_table_cell::GraphCrossTableCell;
  use crate::data::graph::graph_edge::GraphEdge;
  use crate::data::graph::graph_header::GraphHeader;
  use crate::data::graph::graph_level::GraphLevel;
  use crate::data::graph::graph_level_point::GraphLevelPoint;
  use crate::data::graph::graph_vertex::GraphVertex;

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
      &mut GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), &filename))?,
      0,
    )?;

    assert_eq!(bytes_written, 28);

    let file: FileSlice = GeneratedResources::open_slice(&GeneratedResources::sample_name(file!(), &filename))?;

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
          nodes_count: 2,
          vertices_count: 4000,
          level_guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
          game_guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b8"),
          cells: vec![
            GraphCrossTableCell {
              game_vertex_id: 0,
              distance: 1.4,
            },
            GraphCrossTableCell {
              game_vertex_id: 1,
              distance: 0.0,
            },
          ],
        },
        GraphCrossTable {
          version: 16,
          nodes_count: 1,
          vertices_count: 3000,
          level_guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
          game_guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b8"),
          cells: vec![GraphCrossTableCell {
            game_vertex_id: 1,
            distance: 2.1,
          }],
        },
      ],
    };

    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 432);

    let bytes_written: usize = writer.flush_chunk_into::<XRayByteOrder>(
      &mut GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), &filename))?,
      0,
    )?;

    assert_eq!(bytes_written, 432);

    let file: FileSlice = GeneratedResources::open_slice(&GeneratedResources::sample_name(file!(), &filename))?;

    assert_eq!(file.bytes_remaining(), 432 + 8);

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

  /// Two levels listed out of id order, level 20 with two vertices joined both ways and a level point each, level 10
  /// with one, each cross table one node a vertex; laid out by [`SpawnGraphsChunk::derive_layout`].
  fn laid_out_graph() -> SpawnGraphsChunk {
    let guid = uuid!("11111111-1111-1111-1111-111111111111");
    let level = |id: u8, name: &str, guid| GraphLevel {
      id,
      name: String::from(name),
      section: format!("level{id}"),
      guid,
      offset: Vector3d::new(0.0, 0.0, 0.0),
    };
    let vertex = |level_id: u8, edges_count: u8| GraphVertex {
      level_point: Vector3d::new(0.0, 0.0, 0.0),
      game_point: Vector3d::new(0.0, 0.0, 0.0),
      level_id,
      level_vertex_id: 0,
      vertex_type: (0, 0, 0, 0).into(),
      edges_offset: 0,
      level_points_offset: 0,
      edges_count,
      level_points_count: 1,
    };
    let point = GraphLevelPoint {
      position: Vector3d::new(0.0, 0.0, 0.0),
      level_vertex_id: 0,
      distance: 0.0,
    };
    let table = |level_guid, vertices_count: u32, game_vertex_id: u16| GraphCrossTable {
      version: 10,
      nodes_count: 1,
      vertices_count,
      level_guid,
      game_guid: guid,
      cells: vec![GraphCrossTableCell {
        game_vertex_id,
        distance: 0.0,
      }],
    };
    let (far, near) = (
      uuid!("22222222-2222-2222-2222-222222222222"),
      uuid!("33333333-3333-3333-3333-333333333333"),
    );
    let mut graph: SpawnGraphsChunk = SpawnGraphsChunk {
      header: GraphHeader {
        version: 10,
        vertices_count: 0,
        edges_count: 0,
        points_count: 0,
        guid,
        levels_count: 0,
      },
      levels: vec![level(20, "far", far), level(10, "near", near)],
      vertices: vec![vertex(20, 1), vertex(20, 1), vertex(10, 0)],
      edges: vec![
        GraphEdge {
          game_vertex_id: 1,
          distance: 5.0,
        },
        GraphEdge {
          game_vertex_id: 0,
          distance: 5.0,
        },
      ],
      points: vec![point.clone(), point.clone(), point],
      cross_tables: vec![table(near, 1, 2), table(far, 2, 1)],
    };

    graph.derive_layout().expect("the graph lays out");
    graph
  }

  #[test]
  fn derives_counts_and_offsets_from_what_the_graph_holds() -> XrfResult {
    let graph: SpawnGraphsChunk = laid_out_graph();
    let offsets: Vec<(u32, u32)> = graph
      .vertices
      .iter()
      .map(|vertex| (vertex.edges_offset, vertex.level_points_offset))
      .collect();

    assert_eq!(
      (
        graph.header.vertices_count,
        graph.header.edges_count,
        graph.header.points_count,
        graph.header.levels_count
      ),
      (3, 2, 3, 2)
    );
    // Three vertices of 42 bytes, then two edges of 6, then the level points of 20.
    assert_eq!(offsets, vec![(126, 138), (132, 158), (138, 178)]);

    graph.assert_engine_layout()?;

    let mut writer: ChunkWriter = ChunkWriter::new();

    graph.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(
      SpawnGraphsChunk::read::<XRayByteOrder, _>(&mut ChunkReader::from_bytes(&writer.buffer)?)?,
      graph
    );

    Ok(())
  }

  #[test]
  fn refuses_counts_the_vertices_do_not_add_up_to() {
    let mut graph: SpawnGraphsChunk = laid_out_graph();

    graph.edges.pop();

    let error: String = graph.derive_layout().expect_err("expect a missing edge").to_string();

    assert!(error.contains("count 2 edges"), "Unexpected error: {error}");
  }

  /// The laid out graph changed by `change` is refused with an error naming `expected`.
  fn assert_refused(change: impl FnOnce(&mut SpawnGraphsChunk), expected: &str) {
    let mut graph: SpawnGraphsChunk = laid_out_graph();

    change(&mut graph);

    let error: String = graph
      .assert_engine_layout()
      .expect_err("expect the engine's layout refused")
      .to_string();

    assert!(error.contains(expected), "Expected {expected:?}, got: {error}");
  }

  #[test]
  fn refuses_what_the_engine_would_misread() {
    assert_refused(|graph| graph.edges[0].game_vertex_id = 3, "leads to vertex 3 of 3");
    assert_refused(
      |graph| graph.vertices[2].level_id = 30,
      "level 30, which the graph lacks",
    );
    assert_refused(|graph| graph.vertices[0].level_vertex_id = 1 << 24, "past 24 bits");
    assert_refused(
      |graph| graph.vertices[1].edges_offset = 0,
      "edges lie off the edge table",
    );
    // Within the table, but between two edges.
    assert_refused(
      |graph| graph.vertices[0].edges_offset += 1,
      "edges lie off the edge table",
    );
    assert_refused(
      |graph| graph.vertices[2].level_points_offset += 20,
      "level points lie off the level point table",
    );
    assert_refused(
      |graph| {
        graph.cross_tables.pop();
      },
      "2 levels but 1 cross tables",
    );
    assert_refused(
      |graph| graph.cross_tables.swap(0, 1),
      "The cross table of level near names level",
    );
    assert_refused(
      |graph| graph.cross_tables[1].cells[0].game_vertex_id = 2,
      "belongs to game vertex 2",
    );
  }
}
