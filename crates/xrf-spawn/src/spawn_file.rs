use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_ltx::FileImportExport;
use xrf_math::Vector3d;
use xrf_utils::{assert_equal, assert_length, write_new_file_staged};

use crate::chunks::spawn_alife_spawns_chunk::SpawnALifeSpawnsChunk;
use crate::chunks::spawn_artefact_spawns_chunk::SpawnArtefactSpawnsChunk;
use crate::chunks::spawn_graphs_chunk::SpawnGraphsChunk;
use crate::chunks::spawn_header_chunk::SpawnHeaderChunk;
use crate::chunks::spawn_patrols_chunk::SpawnPatrolsChunk;
use crate::data::alife::alife_object::AlifeObject;
use crate::data::alife::alife_object_inherited::AlifeObjectInherited;
use crate::data::alife::spawn_level_arrival::SpawnLevelArrival;
use crate::data::alife::spawn_level_objects::SpawnLevelObjects;
use crate::data::alife::spawn_skipped_object::SpawnSkippedObject;
use crate::data::graph::graph_vertex_levels::GraphVertexLevels;
use crate::data::graph::spawn_level_graph::SpawnLevelGraph;
use crate::data::patrols::patrol::Patrol;

/// Descriptor of generic spawn file used by xray game engine.
///
/// Root level samples by ID:
/// 0 - header
/// 1 - alife spawns
/// 2 - alife objects
/// 3 - patrols
/// 4 - game graphs
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnFile {
  pub header: SpawnHeaderChunk,
  pub alife_spawn: SpawnALifeSpawnsChunk,
  pub artefact_spawn: SpawnArtefactSpawnsChunk,
  pub patrols: SpawnPatrolsChunk,
  pub graphs: SpawnGraphsChunk,
}

impl SpawnFile {
  /// Read spawn file from provided path.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path)?)
  }

  /// Read spawn file from file.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// The route an archived entry takes: a volume holds no file to slice, only bytes.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Self::read_from_chunks::<T, _>(&chunks)
  }

  /// Read spawn file from chunks.
  pub fn read_from_chunks<T: ByteOrder, D: ChunkDataSource>(chunks: &[ChunkReader<D>]) -> XrfResult<Self> {
    assert_length(chunks, 5, "Unexpected chunks count in spawn file root, expected 5")?;

    let spawn_file: Self = {
      Self {
        header: ChunkReader::find_required_by_id(chunks, SpawnHeaderChunk::CHUNK_ID)?.read_xr::<T, _>()?,
        alife_spawn: ChunkReader::find_required_by_id(chunks, SpawnALifeSpawnsChunk::CHUNK_ID)?.read_xr::<T, _>()?,
        artefact_spawn: ChunkReader::find_required_by_id(chunks, SpawnArtefactSpawnsChunk::CHUNK_ID)?
          .read_xr::<T, _>()?,
        patrols: ChunkReader::find_required_by_id(chunks, SpawnPatrolsChunk::CHUNK_ID)?.read_xr::<T, _>()?,
        graphs: ChunkReader::find_required_by_id(chunks, SpawnGraphsChunk::CHUNK_ID)?.read_xr::<T, _>()?,
      }
    };

    assert_length(
      &spawn_file.alife_spawn.objects,
      spawn_file.header.objects_count as usize,
      "Expected correct objects count",
    )?;
    assert_equal(
      spawn_file.header.levels_count,
      spawn_file.graphs.header.levels_count as u32,
      "Expected correct levels count",
    )?;

    Ok(spawn_file)
  }

  /// Read only the game graphs chunk of the spawn file by provided path.
  ///
  /// Unlike [`Self::read_from_path`], ALife, artefact spawn and patrol chunks are not parsed at all.
  /// Consumers that only need the level roster stay readable on spawn files containing ALife object
  /// classes without a CLSID mapping, and skip the bulk of the file.
  pub fn read_graphs_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<SpawnGraphsChunk> {
    Self::read_graphs_from_file::<T>(File::open(path)?)
  }

  /// Read only the game graphs chunk of the spawn file from file.
  pub fn read_graphs_from_file<T: ByteOrder>(file: File) -> XrfResult<SpawnGraphsChunk> {
    Self::read_graphs_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Read only the game graphs chunk from a reader over any data source.
  ///
  /// The route an archived spawn file takes: a volume holds no file to slice, only bytes.
  pub fn read_graphs_from_chunk<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
  ) -> XrfResult<SpawnGraphsChunk> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Self::read_graphs_from_chunks::<T, _>(&chunks)
  }

  /// Read only the game graphs chunk of the spawn file from chunks.
  pub fn read_graphs_from_chunks<T: ByteOrder, D: ChunkDataSource>(
    chunks: &[ChunkReader<D>],
  ) -> XrfResult<SpawnGraphsChunk> {
    ChunkReader::find_required_by_id(chunks, SpawnGraphsChunk::CHUNK_ID)?.read_xr::<T, _>()
  }

  /// Reads which level each game vertex stands on out of a spawn file's root chunks, and nothing else of the graph.
  ///
  /// # Errors
  ///
  /// Returns an error when the graph chunk is missing or its head cannot be read.
  pub(crate) fn read_vertex_levels_from_chunks<T: ByteOrder, D: ChunkDataSource>(
    chunks: &[ChunkReader<D>],
  ) -> XrfResult<GraphVertexLevels> {
    GraphVertexLevels::read::<T, _>(&mut ChunkReader::find_required_by_id(
      chunks,
      SpawnGraphsChunk::CHUNK_ID,
    )?)
  }

  /// The alife objects a spawn file places on one level, by the level each one's game vertex stands on, reading nothing
  /// of the file but its objects and the head of its graph.
  ///
  /// # Errors
  ///
  /// Returns an error when the objects chunk or the head of the graph is missing or cannot be read.
  pub fn read_level_objects_from_chunks<T: ByteOrder, D: ChunkDataSource>(
    chunks: &[ChunkReader<D>],
    level: &str,
  ) -> XrfResult<SpawnLevelObjects> {
    let levels: GraphVertexLevels = Self::read_vertex_levels_from_chunks::<T, _>(chunks)?;
    let mut objects: Vec<AlifeObject> = Vec::new();
    let mut arrivals: Vec<SpawnLevelArrival> = Vec::new();
    let mut skipped: Vec<SpawnSkippedObject> = Vec::new();
    let total: u32 = SpawnALifeSpawnsChunk::read_each::<T, _>(
      &mut ChunkReader::find_required_by_id(chunks, SpawnALifeSpawnsChunk::CHUNK_ID)?,
      |index, object| match object {
        Ok(object) => {
          if let AlifeObjectInherited::SeLevelChanger(changer) = &object.inherited {
            let destination: Option<&str> = levels.get_level(changer.dest_game_vertex_id).map(|it| it.name.as_str());

            arrivals.extend(SpawnLevelArrival::of(&object.name, changer, destination, level));
          }

          if levels
            .get_object_level(&object)
            .is_some_and(|it| it.name.eq_ignore_ascii_case(level))
          {
            objects.push(object);
          }
        }
        Err(error) => skipped.push(SpawnSkippedObject {
          index,
          reason: error.to_string(),
        }),
      },
    )?;

    Ok(SpawnLevelObjects {
      arrivals,
      objects,
      total,
      skipped,
    })
  }

  /// What a spawn file's graph and patrols hold of one level: the graph's levels, the level's game vertices, and the
  /// patrols whose first point's game vertex stands on it, reading nothing of the objects.
  ///
  /// # Errors
  ///
  /// Returns an error when the graph or patrols chunk is missing or cannot be read.
  pub fn read_level_graph_from_chunks<T: ByteOrder, D: ChunkDataSource>(
    chunks: &[ChunkReader<D>],
    level: &str,
  ) -> XrfResult<SpawnLevelGraph> {
    let graph: GraphVertexLevels = Self::read_vertex_levels_from_chunks::<T, _>(chunks)?;
    let is_on_level = |game_vertex_id: u16| {
      graph
        .get_level(game_vertex_id)
        .is_some_and(|it| it.name.eq_ignore_ascii_case(level))
    };
    let points: Vec<(u16, Vector3d)> = (0..=u16::MAX)
      .zip(&graph.vertices)
      .filter(|(id, _)| is_on_level(*id))
      .map(|(id, vertex)| (id, vertex.level_point.clone()))
      .collect();
    let patrols: Vec<Patrol> = ChunkReader::find_required_by_id(chunks, SpawnPatrolsChunk::CHUNK_ID)?
      .read_xr::<T, SpawnPatrolsChunk>()?
      .patrols
      .into_iter()
      .filter(|patrol| {
        patrol
          .points
          .first()
          .is_some_and(|point| is_on_level(point.game_vertex_id))
      })
      .collect();

    Ok(SpawnLevelGraph {
      levels: graph.levels,
      points,
      patrols,
    })
  }

  /// Write spawn file data to the file by provided path, making the directories above it: encoded whole first and
  /// staged beside it, so a failure leaves whatever was there before.
  pub fn write_to_path<T: ByteOrder, P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    let mut bytes: Vec<u8> = Vec::new();

    self.write_to::<T>(&mut bytes)?;

    Ok(write_new_file_staged(path.as_ref(), &bytes)?)
  }

  /// Write spawn file data to the writer.
  pub fn write_to<T: ByteOrder>(&self, writer: &mut dyn Write) -> XrfResult {
    let mut header_chunk_writer: ChunkWriter = ChunkWriter::new();
    header_chunk_writer.write_xr::<T, _>(&self.header)?;
    header_chunk_writer.flush_chunk_into::<T>(writer, SpawnHeaderChunk::CHUNK_ID)?;

    let mut alife_spawn_chunk_writer: ChunkWriter = ChunkWriter::new();
    alife_spawn_chunk_writer.write_xr::<T, _>(&self.alife_spawn)?;
    alife_spawn_chunk_writer.flush_chunk_into::<T>(writer, SpawnALifeSpawnsChunk::CHUNK_ID)?;

    let mut artefact_spawn_chunk_writer: ChunkWriter = ChunkWriter::new();
    artefact_spawn_chunk_writer.write_xr::<T, _>(&self.artefact_spawn)?;
    artefact_spawn_chunk_writer.flush_chunk_into::<T>(writer, SpawnArtefactSpawnsChunk::CHUNK_ID)?;

    let mut patrols_chunk_writer: ChunkWriter = ChunkWriter::new();
    patrols_chunk_writer.write_xr::<T, _>(&self.patrols)?;
    patrols_chunk_writer.flush_chunk_into::<T>(writer, SpawnPatrolsChunk::CHUNK_ID)?;

    let mut graphs_chunk_writer: ChunkWriter = ChunkWriter::new();
    graphs_chunk_writer.write_xr::<T, _>(&self.graphs)?;
    graphs_chunk_writer.flush_chunk_into::<T>(writer, SpawnGraphsChunk::CHUNK_ID)?;

    Ok(())
  }

  /// Read spawn file from provided path.
  pub fn import_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Ok(Self {
      header: SpawnHeaderChunk::import(path)?,
      alife_spawn: SpawnALifeSpawnsChunk::import(path)?,
      artefact_spawn: SpawnArtefactSpawnsChunk::import(path)?,
      patrols: SpawnPatrolsChunk::import(path)?,
      graphs: SpawnGraphsChunk::import(path)?,
    })
  }

  /// Export unpacked ALife spawn file into provided path.
  pub fn export_to_path<T: ByteOrder, P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    fs::create_dir_all(path)?;

    self.header.export(path)?;
    self.alife_spawn.export(path)?;
    self.artefact_spawn.export(path)?;
    self.patrols.export(path)?;
    self.graphs.export(path)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use uuid::{Uuid, uuid};
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_math::Vector3d;
  use xrf_test_utils::utils::{assert_publication_left_previous, create_generated_test_resource_dir};
  use xrf_utils::staging_faults::fail_next_staged_write;

  use super::SpawnFile;
  use crate::chunks::spawn_alife_spawns_chunk::SpawnALifeSpawnsChunk;
  use crate::chunks::spawn_artefact_spawns_chunk::SpawnArtefactSpawnsChunk;
  use crate::chunks::spawn_graphs_chunk::SpawnGraphsChunk;
  use crate::chunks::spawn_header_chunk::SpawnHeaderChunk;
  use crate::chunks::spawn_patrols_chunk::SpawnPatrolsChunk;
  use crate::data::graph::graph_header::GraphHeader;
  use crate::data::graph::graph_level::GraphLevel;
  use crate::data::graph::graph_vertex::GraphVertex;
  use crate::data::graph::spawn_level_graph::SpawnLevelGraph;
  use crate::data::patrols::patrol::Patrol;
  use crate::data::patrols::patrol_point::PatrolPoint;

  fn new_level(id: u8, name: &str, offset: f32) -> GraphLevel {
    GraphLevel {
      id,
      name: String::from(name),
      section: String::from(name),
      guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
      offset: Vector3d::new(offset, 0.0, 0.0),
    }
  }

  fn new_vertex(level_id: u8, x: f32) -> GraphVertex {
    GraphVertex {
      level_point: Vector3d::new(x, 1.0, -x),
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

  fn new_patrol(name: &str, game_vertex_id: u16) -> Patrol {
    Patrol {
      name: String::from(name),
      points: vec![PatrolPoint {
        name: String::from("wp00"),
        position: Vector3d::new(0.0, 0.0, 0.0),
        flags: 0,
        level_vertex_id: 0,
        game_vertex_id,
      }],
      links: vec![],
    }
  }

  /// A write that fails part way leaves the spawn a caller had, and nothing staged beside it.
  #[test]
  fn keeps_the_previous_file_when_publication_fails() -> XrfResult {
    let path: PathBuf = create_generated_test_resource_dir("spawn_file/publication")?.join("all.spawn");
    let guid: Uuid = uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7");
    let spawn: SpawnFile = SpawnFile {
      header: SpawnHeaderChunk {
        version: 10,
        guid,
        graph_guid: guid,
        objects_count: 0,
        levels_count: 0,
      },
      alife_spawn: SpawnALifeSpawnsChunk { objects: Vec::new() },
      artefact_spawn: SpawnArtefactSpawnsChunk { nodes: Vec::new() },
      patrols: SpawnPatrolsChunk { patrols: Vec::new() },
      graphs: SpawnGraphsChunk {
        header: GraphHeader {
          version: 10,
          vertices_count: 0,
          edges_count: 0,
          points_count: 0,
          guid,
          levels_count: 0,
        },
        levels: Vec::new(),
        vertices: Vec::new(),
        edges: Vec::new(),
        points: Vec::new(),
        cross_tables: Vec::new(),
      },
    };

    fs::write(&path, b"previous")?;
    fail_next_staged_write();

    assert!(spawn.write_to_path::<XRayByteOrder, _>(&path).is_err());
    assert_publication_left_previous(&path, b"previous");

    Ok(())
  }

  #[test]
  fn reads_the_graph_points_and_patrols_of_one_level() -> XrfResult {
    let graphs: SpawnGraphsChunk = SpawnGraphsChunk {
      header: GraphHeader {
        version: 10,
        vertices_count: 3,
        edges_count: 0,
        points_count: 0,
        guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
        levels_count: 2,
      },
      levels: vec![new_level(118, "pripyat", 900.0), new_level(108, "zaton", 0.0)],
      vertices: vec![new_vertex(108, 10.0), new_vertex(118, 20.0), new_vertex(108, 30.0)],
      edges: vec![],
      points: vec![],
      cross_tables: vec![],
    };
    let patrols: SpawnPatrolsChunk = SpawnPatrolsChunk {
      patrols: vec![new_patrol("zat_walk", 2), new_patrol("pri_walk", 1)],
    };
    let mut graphs_writer: ChunkWriter = ChunkWriter::new();
    let mut patrols_writer: ChunkWriter = ChunkWriter::new();

    graphs.write::<XRayByteOrder>(&mut graphs_writer)?;
    patrols.write::<XRayByteOrder>(&mut patrols_writer)?;

    let mut bytes: Vec<u8> = graphs_writer.flush_chunk_into_buffer::<XRayByteOrder>(SpawnGraphsChunk::CHUNK_ID)?;

    bytes.extend(patrols_writer.flush_chunk_into_buffer::<XRayByteOrder>(SpawnPatrolsChunk::CHUNK_ID)?);

    let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_vec(bytes)?.read_children()?;
    let graph: SpawnLevelGraph = SpawnFile::read_level_graph_from_chunks::<XRayByteOrder, _>(&chunks, "Zaton")?;

    assert_eq!(
      graph.points,
      vec![
        (0, Vector3d::new(10.0, 1.0, -10.0)),
        (2, Vector3d::new(30.0, 1.0, -30.0))
      ]
    );
    assert_eq!(
      graph.patrols.iter().map(|it| it.name.as_str()).collect::<Vec<_>>(),
      vec!["zat_walk"]
    );
    assert_eq!(graph.get_level("PRIPYAT").map(|it| it.offset.x), Some(900.0));

    Ok(())
  }
}
