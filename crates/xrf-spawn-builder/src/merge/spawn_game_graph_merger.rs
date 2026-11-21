use uuid::Uuid;
use xrf_chunk::{ChunkReadWrite, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_spawn::{
  GraphCrossTable, GraphCrossTableCell, GraphEdge, GraphHeader, GraphLevel, GraphLevelPoint, GraphVertex,
  SpawnGraphsChunk,
};
use xrf_utils::{ContentGuid, narrow_format_size};

use crate::merge::spawn_game_graph_merge::SpawnGameGraphMerge;
use crate::merge::spawn_merge_level::SpawnMergeLevel;
use crate::merge::spawn_random::SpawnRandom;
use crate::merge::spawn_skipped_connection::SpawnSkippedConnection;

/// The levels' game graphs merged into the one a root spawn holds, `CGraphMerger`.
///
/// Levels go in ascending id order, each level's vertices numbered after those of the levels before. The graph GUID is
/// derived from the content, so the same levels give the same graph.
pub struct SpawnGameGraphMerger;

impl SpawnGameGraphMerger {
  /// `XRAI_CURRENT_VERSION`, the version of the graph and its cross tables.
  pub const VERSION: u8 = 10;

  /// Merges the levels' graphs, drawing level points from `random`, the build's one stream, which the zones'
  /// artefact points draw from next.
  ///
  /// # Errors
  ///
  /// Returns an error when two levels share an id or a name, a level's graph or cross table does not match its points
  /// and map, a connection names a graph point its level lacks, or the graph exceeds its format's counts.
  pub fn merge(levels: &[SpawnMergeLevel<'_>], random: &mut SpawnRandom) -> XrfResult<SpawnGameGraphMerge> {
    let mut ordered: Vec<&SpawnMergeLevel<'_>> = levels.iter().collect();

    ordered.sort_by_key(|level| level.id);

    Self::assert_mergeable(&ordered)?;

    let mut firsts: Vec<u32> = Vec::with_capacity(ordered.len());
    let mut first: usize = 0;

    for level in &ordered {
      firsts.push(narrow_format_size(first, "A game vertex id")?);
      first += level.points.len();
    }

    let mut vertices: Vec<GraphVertex> = Vec::new();
    let mut vertex_edges: Vec<Vec<GraphEdge>> = Vec::new();
    let mut points: Vec<GraphLevelPoint> = Vec::new();

    for (level, first) in ordered.iter().zip(&firsts) {
      let level_points: Vec<Vec<GraphLevelPoint>> = Self::draw_level_points(level, random);

      for ((point, edges), drawn) in level.points.iter().zip(&level.graph.edges).zip(level_points) {
        vertices.push(GraphVertex {
          level_point: point.position,
          game_point: Self::add(&point.position, &level.offset),
          level_id: level.id,
          level_vertex_id: point.node,
          vertex_type: (
            point.locations[0],
            point.locations[1],
            point.locations[2],
            point.locations[3],
          )
            .into(),
          edges_offset: 0,
          level_points_offset: 0,
          edges_count: 0,
          level_points_count: narrow_format_size(drawn.len(), "A vertex's level point count")?,
        });
        vertex_edges.push(
          edges
            .iter()
            .map(|edge| {
              Ok(GraphEdge {
                game_vertex_id: narrow_format_size(*first as usize + usize::from(edge.target), "A game vertex id")?,
                distance: edge.distance,
              })
            })
            .collect::<XrfResult<_>>()?,
        );
        points.extend(drawn);
      }
    }

    let skipped: Vec<SpawnSkippedConnection> = Self::connect(&ordered, &firsts, &vertices, &mut vertex_edges)?;

    for (vertex, edges) in vertices.iter_mut().zip(&vertex_edges) {
      vertex.edges_count = narrow_format_size(edges.len(), "A vertex's edge count")?;
    }

    let mut graph: SpawnGraphsChunk = SpawnGraphsChunk {
      header: GraphHeader {
        version: Self::VERSION,
        vertices_count: 0,
        edges_count: 0,
        points_count: 0,
        guid: Uuid::nil(),
        levels_count: 0,
      },
      levels: ordered
        .iter()
        .map(|level| GraphLevel {
          name: level.name.clone(),
          offset: level.offset,
          id: level.id,
          section: level.section.clone(),
          guid: level.get_guid(),
        })
        .collect(),
      vertices,
      edges: vertex_edges.into_iter().flatten().collect(),
      points,
      cross_tables: ordered
        .iter()
        .zip(&firsts)
        .map(|(level, first)| {
          Ok(GraphCrossTable {
            version: u32::from(Self::VERSION),
            nodes_count: narrow_format_size(level.cells.len(), "A cross table's node count")?,
            vertices_count: narrow_format_size(level.points.len(), "A cross table's vertex count")?,
            level_guid: level.get_guid(),
            game_guid: Uuid::nil(),
            cells: level
              .cells
              .iter()
              .map(|cell| {
                Ok(GraphCrossTableCell {
                  game_vertex_id: narrow_format_size(
                    usize::from(cell.game_vertex_id) + *first as usize,
                    "A game vertex id",
                  )?,
                  distance: cell.distance,
                })
              })
              .collect::<XrfResult<_>>()?,
          })
        })
        .collect::<XrfResult<_>>()?,
    };

    graph.derive_layout()?;
    Self::assign_guid(&mut graph)?;
    graph.assert_engine_layout()?;

    Ok(SpawnGameGraphMerge { graph, skipped })
  }

  /// Refuses levels sharing an id or a name, a level whose graph or cross table does not match its points or map, or
  /// more vertices in all than a game vertex id holds.
  fn assert_mergeable(ordered: &[&SpawnMergeLevel<'_>]) -> XrfResult {
    let invalid = |message: String| Err(XrfError::new_invalid_error(message));

    for (index, level) in ordered.iter().enumerate() {
      if let Some(other) = ordered[..index]
        .iter()
        .find(|other| other.id == level.id || other.name == level.name)
      {
        return invalid(format!(
          "Levels {} and {} share an id or a name",
          other.name, level.name
        ));
      }

      if level.graph.edges.len() != level.points.len() || level.cells.len() != level.map.get_node_count() {
        return invalid(format!(
          "Level {}'s graph or cross table was not built from its {} graph points and {} nodes",
          level.name,
          level.points.len(),
          level.map.get_node_count()
        ));
      }
    }

    let vertices: usize = ordered.iter().map(|level| level.points.len()).sum();

    if vertices > usize::from(u16::MAX) {
      return invalid(format!(
        "{vertices} game vertices are past the 65,535 a game graph counts"
      ));
    }

    Ok(())
  }

  /// `vfGenerateDeathPoints` for every vertex of a level, in vertex order.
  ///
  /// Each vertex's region nodes, in node order, are shuffled; it keeps the first tenth, at most 255, or all of ten or
  /// fewer. Drawn from the stream's start, this gives the shipped points.
  fn draw_level_points(level: &SpawnMergeLevel<'_>, random: &mut SpawnRandom) -> Vec<Vec<GraphLevelPoint>> {
    let mut regions: Vec<Vec<u32>> = vec![Vec::new(); level.points.len()];

    for (node, cell) in level.cells.iter().enumerate() {
      if let Some(region) = regions.get_mut(cell.game_vertex_id as usize) {
        region.push(node as u32);
      }
    }

    regions
      .into_iter()
      .map(|mut region| {
        random.shuffle(&mut region);

        let kept: usize = if region.len() > 10 {
          ((0.1 * region.len() as f32).floor() as usize).min(255)
        } else {
          region.len()
        };

        region[..kept]
          .iter()
          .map(|node| GraphLevelPoint {
            position: level.map.get_node_position(*node),
            level_vertex_id: *node,
            distance: level.cells[*node as usize].distance,
          })
          .collect()
      })
      .collect()
  }

  /// "Adding interconnection points": an edge from each connecting graph point to the vertex it names, weighed by
  /// their distance in the shared world; a connection to a level outside the build is skipped.
  ///
  /// Levels go in id order, each level's points by name.
  fn connect(
    levels: &[&SpawnMergeLevel<'_>],
    firsts: &[u32],
    vertices: &[GraphVertex],
    edges: &mut [Vec<GraphEdge>],
  ) -> XrfResult<Vec<SpawnSkippedConnection>> {
    let mut skipped: Vec<SpawnSkippedConnection> = Vec::new();

    for (level, first) in levels.iter().zip(firsts) {
      let mut named: Vec<(usize, &str)> = level
        .points
        .iter()
        .enumerate()
        .map(|(index, point)| (index, point.name.as_str()))
        .collect();

      // `VERTEX_MAP`: the level's graph points by name, compared byte by byte.
      named.sort_by(|a, b| a.1.as_bytes().cmp(b.1.as_bytes()));

      for (index, _) in named {
        let point = &level.points[index];

        if point.connection_point_name.is_empty() {
          continue;
        }

        // todo: a graph point xrAI dropped within a millimetre of a kept one still connects through that one; such
        // points are not kept here.
        let Some((target_level, target_first)) = levels
          .iter()
          .zip(firsts)
          .find(|(target, _)| target.caption == point.connection_level_name)
        else {
          skipped.push(SpawnSkippedConnection {
            level: level.name.clone(),
            point: point.name.clone(),
            target_level: point.connection_level_name.clone(),
          });
          continue;
        };
        let target: usize = target_level
          .points
          .iter()
          .position(|target| target.name == point.connection_point_name)
          .ok_or_else(|| {
            XrfError::new_invalid_error(format!(
              "Graph point {} of level {} connects to graph point {} of level {}, which that level lacks",
              point.name, level.name, point.connection_point_name, target_level.name
            ))
          })?;
        let (from, to): (usize, usize) = (*first as usize + index, *target_first as usize + target);

        edges[from].push(GraphEdge {
          game_vertex_id: narrow_format_size(to, "A game vertex id")?,
          distance: Self::distance(&vertices[from].game_point, &vertices[to].game_point),
        });
      }
    }

    Ok(skipped)
  }

  /// Sets the graph GUID on the graph and every cross table: the content GUID of the graph chunk written with a nil
  /// GUID.
  fn assign_guid(graph: &mut SpawnGraphsChunk) -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    graph.write(&mut writer)?;

    let guid: Uuid = Uuid::from_bytes(ContentGuid::of(&writer.buffer));

    graph.header.guid = guid;

    for table in &mut graph.cross_tables {
      table.game_guid = guid;
    }

    Ok(())
  }

  fn add(a: &Vector3d<f32>, b: &Vector3d<f32>) -> Vector3d<f32> {
    Vector3d::new(a.x + b.x, a.y + b.y, a.z + b.z)
  }

  /// `Fvector::distance_to`.
  fn distance(a: &Vector3d<f32>, b: &Vector3d<f32>) -> f32 {
    let (dx, dy, dz) = (a.x - b.x, a.y - b.y, a.z - b.z);

    (dx * dx + dy * dy + dz * dz).sqrt()
  }
}
