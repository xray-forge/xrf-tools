use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use xrf_level::{
  LevelFile, LevelGeomFile, LevelGeomVertexBuffer, LevelSector, LevelShaderEntry, LevelVisual, LevelVisualsChunk,
};
use xrf_ogf::OgfModelType;

/// What `level info` read out of a compiled level.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelInfoReport {
  pub header: LevelHeaderReport,
  pub shaders: LevelShadersReport,
  /// Absent for a level carrying no visuals chunk, which is a level that draws nothing.
  pub visuals: Option<LevelVisualsReport>,
  /// `level.geom`, absent when the directory holds none.
  pub geometry: Option<LevelGeomReport>,
  /// `level.geomx`, the fast-path twin, absent when the directory holds none.
  pub detail_geometry: Option<LevelGeomReport>,
  pub structure: LevelStructureReport,
}

/// What the level is built out of beside its geometry: the sectors it culls by and the lights it loads.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelStructureReport {
  pub sectors: usize,
  pub portals: usize,
  /// Portals named by a sector, which is twice the portals when every one joins two of them.
  pub sector_portal_references: usize,
  pub lights: usize,
  /// Whether the compiler wrote the directional light the loader keeps as the sun.
  pub has_sun: bool,
}

impl LevelStructureReport {
  fn of(level: &LevelFile) -> Self {
    let sectors: &[LevelSector] = level.sectors.as_ref().map_or(&[], |chunk| &chunk.sectors);

    Self {
      has_sun: level.lights.as_ref().is_some_and(|chunk| chunk.get_sun().is_some()),
      lights: level.lights.as_ref().map_or(0, |chunk| chunk.lights.len()),
      portals: level.portals.as_ref().map_or(0, |chunk| chunk.portals.len()),
      sector_portal_references: sectors.iter().map(|sector| sector.portals.len()).sum(),
      sectors: sectors.len(),
    }
  }
}

impl LevelInfoReport {
  pub fn new(
    level: &LevelFile,
    visuals: Option<&LevelVisualsChunk>,
    geometry: Option<&LevelGeomFile>,
    detail_geometry: Option<&LevelGeomFile>,
  ) -> Self {
    Self {
      header: LevelHeaderReport {
        xrlc_quality: level.header.xrlc_quality,
        xrlc_version: level.header.xrlc_version,
      },
      shaders: LevelShadersReport::of(level),
      visuals: visuals.map(LevelVisualsReport::of),
      geometry: geometry.map(LevelGeomReport::of),
      detail_geometry: detail_geometry.map(LevelGeomReport::of),
      structure: LevelStructureReport::of(level),
    }
  }
}

/// What the compiler stamped the level with.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelHeaderReport {
  pub xrlc_quality: u16,
  pub xrlc_version: u16,
}

/// The shader table, counted by what each entry is rather than listed whole.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelShadersReport {
  pub total: usize,
  pub references: usize,
  pub lightmapped: usize,
  /// Entries the renderer skips because they name nothing.
  pub empty: usize,
  /// Entries without the `/` delimiter, which the renderer dereferences without a null check.
  pub malformed: usize,
}

impl LevelShadersReport {
  fn of(level: &LevelFile) -> Self {
    let entries: &[LevelShaderEntry] = match &level.shaders {
      Some(shaders) => &shaders.entries,
      None => &[],
    };

    Self {
      empty: entries
        .iter()
        .filter(|it| matches!(it, LevelShaderEntry::Empty))
        .count(),
      lightmapped: entries
        .iter()
        .filter(|it| matches!(it, LevelShaderEntry::Reference(reference) if reference.textures.len() > 1))
        .count(),
      malformed: entries
        .iter()
        .filter(|it| matches!(it, LevelShaderEntry::Malformed(_)))
        .count(),
      references: entries
        .iter()
        .filter(|it| matches!(it, LevelShaderEntry::Reference(_)))
        .count(),
      total: entries.len(),
    }
  }
}

/// The visuals run, counted by what the visuals are and what they draw.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelVisualsReport {
  pub total: usize,
  /// Visuals naming a range in the level's shared buffers. A hierarchy visual names none.
  pub drawable: usize,
  /// Visuals the compiler also gave a `level.geomx` range.
  pub fastpath: usize,
  /// Visuals carrying their own shader and texture names rather than taking them from the shader table.
  pub textured: usize,
  /// Vertices the drawable visuals address, counting a shared vertex once per visual that names it.
  pub vertices: u64,
  pub indices: u64,
  pub triangles: u64,
  /// Distinct geometry ranges the drawable visuals name.
  pub unique_ranges: usize,
  /// How many visuals of each engine model type, by `MT_*` identifier.
  pub model_types: BTreeMap<String, usize>,
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

  fn of(visuals: &LevelVisualsChunk) -> Self {
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
      triangles: indices / 3,
      unique_ranges: ranges.len(),
      vertices,
    }
  }
}

/// One render geometry file, reported by the shape of its buffers.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGeomReport {
  pub vertex_buffers: usize,
  pub index_buffers: usize,
  pub slide_windows: usize,
  pub vertices: u64,
  pub indices: u64,
  /// Distinct vertex declarations, each with what it is spelled as and how much of the level uses it.
  pub layouts: Vec<LevelVertexLayoutReport>,
}

impl LevelGeomReport {
  fn of(geometry: &LevelGeomFile) -> Self {
    let mut layouts: BTreeMap<(String, Option<u32>), (usize, u64)> = BTreeMap::new();

    for buffer in &geometry.vertex_buffers {
      let entry: &mut (usize, u64) = layouts
        .entry((LevelVertexLayoutReport::describe(buffer), buffer.get_vertex_size()))
        .or_default();

      entry.0 += 1;
      entry.1 += buffer.vertex_count as u64;
    }

    Self {
      index_buffers: geometry.index_buffers.len(),
      indices: geometry.index_buffers.iter().map(|it| it.index_count as u64).sum(),
      layouts: layouts
        .into_iter()
        .map(|((declaration, stride), (buffers, vertices))| LevelVertexLayoutReport {
          buffers,
          declaration,
          stride,
          vertices,
        })
        .collect(),
      slide_windows: geometry.slide_windows.len(),
      vertex_buffers: geometry.vertex_buffers.len(),
      vertices: geometry.vertex_buffers.iter().map(|it| it.vertex_count as u64).sum(),
    }
  }
}

/// One vertex declaration and how much of a level's geometry is stored in it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelVertexLayoutReport {
  pub declaration: String,
  /// Bytes one vertex occupies, absent when the declaration names a type `D3DDECLTYPE` does not.
  pub stride: Option<u32>,
  pub buffers: usize,
  pub vertices: u64,
}

impl LevelVertexLayoutReport {
  /// `D3DDECLUSAGE` by value; an index past the end is a usage the enumeration does not name.
  const USAGES: [&'static str; 14] = [
    "POSITION",
    "BLENDWEIGHT",
    "BLENDINDICES",
    "NORMAL",
    "PSIZE",
    "TEXCOORD",
    "TANGENT",
    "BINORMAL",
    "TESSFACTOR",
    "POSITIONT",
    "COLOR",
    "FOG",
    "DEPTH",
    "SAMPLE",
  ];

  /// `D3DDECLTYPE` by value, in the order `g_declTypeSizes` sizes them.
  const KINDS: [&'static str; 17] = [
    "FLOAT1",
    "FLOAT2",
    "FLOAT3",
    "FLOAT4",
    "D3DCOLOR",
    "UBYTE4",
    "SHORT2",
    "SHORT4",
    "UBYTE4N",
    "SHORT2N",
    "SHORT4N",
    "USHORT2N",
    "USHORT4N",
    "UDEC3",
    "DEC3N",
    "FLOAT16_2",
    "FLOAT16_4",
  ];

  /// A declaration as one line, so two buffers sharing a layout collapse into one reported row.
  pub fn describe(buffer: &LevelGeomVertexBuffer) -> String {
    buffer
      .declaration
      .iter()
      .map(|element| {
        format!(
          "{}{}:{}",
          Self::describe_usage(element.usage),
          element.usage_index,
          Self::describe_kind(element.kind)
        )
      })
      .collect::<Vec<_>>()
      .join(" | ")
  }

  fn describe_usage(usage: u8) -> String {
    Self::USAGES
      .get(usage as usize)
      .map_or_else(|| format!("USAGE({usage})"), |it| String::from(*it))
  }

  fn describe_kind(kind: u8) -> String {
    Self::KINDS
      .get(kind as usize)
      .map_or_else(|| format!("TYPE({kind})"), |it| String::from(*it))
  }
}
