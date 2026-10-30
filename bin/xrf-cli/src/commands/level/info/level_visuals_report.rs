use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use xrf_level::{LevelVisual, LevelVisualsChunk};
use xrf_ogf::OgfModelType;
use xrf_visual::TreeTypes;

/// The visuals run, counted by what the visuals are and what they draw.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelVisualsReport {
  total: usize,
  /// Visuals naming a range in the level's shared buffers. A hierarchy visual names none.
  drawable: usize,
  /// Visuals the compiler also gave a `level.geomx` range.
  fastpath: usize,
  /// Visuals carrying their own shader and texture names rather than taking them from the shader table.
  textured: usize,
  /// Vertices the drawable visuals address, counting a shared vertex once per visual that names it.
  vertices: u64,
  /// Indices of a triangle list, so a third of them is the triangle count.
  indices: u64,
  /// Distinct geometry ranges the drawable visuals name.
  unique_ranges: usize,
  /// How many visuals of each engine model type, by `MT_*` identifier.
  model_types: BTreeMap<String, usize>,
  trees: LevelTreesReport,
}

/// The level's trees: each tree visual a part standing at its tree's transform, the trees standing the same parts one
/// type.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelTreesReport {
  /// Tree visuals, a part of a tree each: a mesh and its shader.
  parts: usize,
  /// Trees: the distinct transforms the parts stand at.
  trees: usize,
  /// Kinds of tree: the distinct sets of parts standing together.
  types: usize,
  /// The kinds, the most stood first: how many trees of each, its parts and the triangles one of them draws.
  kinds: Vec<LevelTreeKindReport>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelTreeKindReport {
  trees: u32,
  /// Each part's first visual, by its index in the run.
  visuals: Vec<u32>,
  triangles: u32,
}

impl LevelTreesReport {
  pub fn of(visuals: &LevelVisualsChunk) -> Self {
    let trees: TreeTypes = TreeTypes::of(visuals);

    Self {
      parts: trees.parts,
      trees: trees.placements,
      types: trees.types.len(),
      kinds: trees
        .types
        .iter()
        .map(|kind| LevelTreeKindReport {
          trees: kind.instances,
          visuals: kind.parts.iter().map(|part| part.visual).collect(),
          triangles: kind.get_triangles(),
        })
        .collect(),
    }
  }
}

impl LevelVisualsReport {
  /// One visual as a line for verbose output, naming what it is and where it draws from.
  pub fn describe_visual(index: usize, visual: &LevelVisual) -> String {
    let geometry: String = match &visual.geometry {
      Some(container) => format!(
        "vb {} [{}..{}), ib {} [{}..{}), {} tris",
        container.vertex_buffer_id,
        container.vertex_base,
        container.vertex_base + container.vertex_count,
        container.index_buffer_id,
        container.index_base,
        container.index_base + container.index_count,
        container.get_primitive_count()
      ),
      None => String::from("no geometry"),
    };

    match &visual.texture {
      Some(texture) => format!(
        "[{}] {}, shader {}, texture {}, {}",
        index,
        OgfModelType::label(visual.header.model_type),
        texture.shader_name,
        texture.texture_name,
        geometry
      ),
      None => format!(
        "[{}] {}, shader id {}, {}",
        index,
        OgfModelType::label(visual.header.model_type),
        visual.header.shader_id,
        geometry
      ),
    }
  }

  pub fn of(visuals: &LevelVisualsChunk) -> Self {
    let mut model_types: BTreeMap<String, usize> = BTreeMap::new();
    let mut ranges: BTreeSet<(u32, u32, u32, u32, u32, u32)> = BTreeSet::new();
    let mut vertices: u64 = 0;
    let mut indices: u64 = 0;

    for visual in &visuals.visuals {
      *model_types
        .entry(OgfModelType::label(visual.header.model_type))
        .or_default() += 1;

      if let Some(geometry) = &visual.geometry {
        vertices += geometry.vertex_count as u64;
        indices += geometry.index_count as u64;

        ranges.insert((
          geometry.vertex_buffer_id,
          geometry.vertex_base,
          geometry.vertex_count,
          geometry.index_buffer_id,
          geometry.index_base,
          geometry.index_count,
        ));
      }
    }

    Self {
      drawable: visuals.count_drawable(),
      fastpath: visuals.count_fastpath(),
      indices,
      model_types,
      textured: visuals.visuals.iter().filter(|it| it.texture.is_some()).count(),
      total: visuals.visuals.len(),
      trees: LevelTreesReport::of(visuals),
      unique_ranges: ranges.len(),
      vertices,
    }
  }
}
