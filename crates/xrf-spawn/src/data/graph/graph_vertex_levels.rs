use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::XrfResult;

use crate::data::alife::alife_object::AlifeObject;
use crate::data::graph::graph_header::GraphHeader;
use crate::data::graph::graph_level::GraphLevel;
use crate::data::graph::graph_vertex::GraphVertex;

/// Which level each game vertex stands on: the head of a graph chunk, read without its edges, points and cross tables.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphVertexLevels {
  pub levels: Vec<GraphLevel>,
  /// Each game vertex's level id, by vertex.
  pub vertex_levels: Vec<u8>,
}

impl GraphVertexLevels {
  /// Reads the graph's header, levels and vertices, leaving the rest of the chunk unread.
  ///
  /// # Errors
  ///
  /// Returns an error when the head of the chunk cannot be read.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let header: GraphHeader = reader.read_xr::<T, _>()?;

    let mut levels: Vec<GraphLevel> = reader.new_bounded_vec(
      header.levels_count.into(),
      GraphLevel::MIN_SERIALIZED_SIZE,
      "graph levels",
    )?;

    for _ in 0..header.levels_count {
      levels.push(reader.read_xr::<T, _>()?);
    }

    let mut vertex_levels: Vec<u8> = reader.new_bounded_vec(
      header.vertices_count.into(),
      GraphVertex::MIN_SERIALIZED_SIZE,
      "graph vertices",
    )?;

    for _ in 0..header.vertices_count {
      vertex_levels.push(reader.read_xr::<T, GraphVertex>()?.level_id);
    }

    Ok(Self { levels, vertex_levels })
  }

  /// The level a spawned object stands on, by its game vertex; `None` for a graph point, which has none, or a vertex
  /// the graph does not hold.
  pub fn get_object_level(&self, object: &AlifeObject) -> Option<&GraphLevel> {
    self.get_level(object.inherited.get_abstract()?.game_vertex_id)
  }

  /// The level a game vertex stands on, or `None` for a vertex or level id the graph does not hold.
  pub fn get_level(&self, game_vertex_id: u16) -> Option<&GraphLevel> {
    let id: u8 = *self.vertex_levels.get(usize::from(game_vertex_id))?;

    self.levels.iter().find(|level| level.id == id)
  }
}

#[cfg(test)]
mod tests {
  use uuid::uuid;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_math::Vector3d;

  use crate::chunks::spawn_graphs_chunk::SpawnGraphsChunk;
  use crate::data::graph::graph_header::GraphHeader;
  use crate::data::graph::graph_level::GraphLevel;
  use crate::data::graph::graph_vertex::GraphVertex;
  use crate::data::graph::graph_vertex_levels::GraphVertexLevels;

  fn new_level(id: u8, name: &str) -> GraphLevel {
    GraphLevel {
      id,
      name: String::from(name),
      section: String::from(name),
      guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
      offset: Vector3d::new(0.0, 0.0, 0.0),
    }
  }

  fn new_vertex(level_id: u8) -> GraphVertex {
    GraphVertex {
      level_point: Vector3d::new(0.0, 0.0, 0.0),
      game_point: Vector3d::new(0.0, 0.0, 0.0),
      level_id,
      level_vertex_id: 0,
      vertex_type: (0, 0, 0, 0).into(),
      edges_offset: 0,
      level_points_offset: 0,
      edges_count: 0,
      level_points_count: 0,
    }
  }

  #[test]
  fn names_the_level_of_each_vertex_by_its_id_not_its_place() -> XrfResult {
    let graphs: SpawnGraphsChunk = SpawnGraphsChunk {
      header: GraphHeader {
        version: 10,
        vertices_count: 3,
        edges_count: 0,
        points_count: 0,
        guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
        levels_count: 2,
      },
      levels: vec![new_level(118, "pripyat"), new_level(108, "zaton")],
      vertices: vec![new_vertex(108), new_vertex(118), new_vertex(7)],
      edges: vec![],
      points: vec![],
      cross_tables: vec![],
    };
    let mut writer: ChunkWriter = ChunkWriter::new();

    graphs.write::<XRayByteOrder>(&mut writer)?;

    let bytes: Vec<u8> = writer.flush_chunk_into_buffer::<XRayByteOrder>(SpawnGraphsChunk::CHUNK_ID)?;
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_vec(bytes)?
      .read_child_by_index(0)
      .expect("the graph chunk to exist");
    let levels: GraphVertexLevels = GraphVertexLevels::read::<XRayByteOrder, _>(&mut reader)?;

    assert_eq!(levels.vertex_levels, vec![108, 118, 7]);
    assert_eq!(levels.get_level(0).map(|it| it.name.as_str()), Some("zaton"));
    assert_eq!(levels.get_level(1).map(|it| it.name.as_str()), Some("pripyat"));
    assert_eq!(levels.get_level(2), None);
    assert_eq!(levels.get_level(3), None);

    Ok(())
  }
}
