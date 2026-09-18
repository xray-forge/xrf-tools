use std::collections::BTreeMap;

use xrf_level::{LevelGeomVertexBuffer, LevelVertex, LevelVertexLayout};
use xrf_math::Vector3d;
use xrf_ogf::{OgfGeometryContainerChunk, OgfModelType};

/// A normal the compiler wrote is a unit vector, so one whose length misses by more than this came out of the wrong
/// element rather than out of rounding.
const NORMAL_LENGTH_TOLERANCE: f32 = 0.1;

/// What a level held, beside what the sweep found.
#[derive(Debug, Default)]
pub struct LevelVerificationCensus {
  pub visuals: usize,
  pub drawable_visuals: usize,
  pub fastpath_visuals: usize,
  pub vertices: u64,
  pub indices: u64,
  pub shader_entries: usize,
  /// Visuals of each engine model type, by `MT_*` identifier.
  pub model_types: BTreeMap<String, usize>,
  /// Vertex declarations the level stores geometry in, by the spelling a reader gives them.
  pub layouts: BTreeMap<String, usize>,
  /// The box every decoded position fell in, which is the level's own extent.
  pub bounds: Option<(Vector3d, Vector3d)>,
  /// Decoded normals whose length is not near one, which a misread normal element would make of all of them.
  pub stray_normals: u64,
  /// The widest base texture coordinate decoded, against the 32 tiles the quantization allows.
  pub widest_coordinate: f32,
  pub sectors: usize,
  pub portals: usize,
  /// Portals named by a sector, counting one named by two sectors twice, which is what a portal joining two is.
  pub sector_portal_references: usize,
  pub lights: usize,
  /// Whether the compiler wrote the directional light the loader keeps as the sun.
  pub has_sun: bool,
}

impl LevelVerificationCensus {
  /// Counts one visual by the kind its header declares.
  pub fn record_visual(&mut self, model_type: u8) {
    self.visuals += 1;
    *self.model_types.entry(OgfModelType::label(model_type)).or_default() += 1;
  }

  /// Counts what one visual draws from the level's ordinary buffers.
  pub fn record_drawable(&mut self, container: &OgfGeometryContainerChunk) {
    self.drawable_visuals += 1;
    self.vertices += u64::from(container.vertex_count);
    self.indices += u64::from(container.index_count);
  }

  /// Counts the declaration a visual's geometry is stored in, and says whether it carries a normal to judge.
  pub fn record_layout(&mut self, buffer: &LevelGeomVertexBuffer) -> bool {
    *self.layouts.entry(Self::describe(buffer)).or_default() += 1;

    LevelVertexLayout::of(buffer).is_ok_and(|layout| layout.get_normal_offset().is_some())
  }

  /// Folds one decoded vertex into the extent, the normal count and the widest coordinate.
  pub fn record_vertex(&mut self, vertex: &LevelVertex, has_normal: bool) {
    self.bounds = Some(match self.bounds.take() {
      None => (vertex.position.clone(), vertex.position.clone()),
      Some((min, max)) => (
        Vector3d {
          x: min.x.min(vertex.position.x),
          y: min.y.min(vertex.position.y),
          z: min.z.min(vertex.position.z),
        },
        Vector3d {
          x: max.x.max(vertex.position.x),
          y: max.y.max(vertex.position.y),
          z: max.z.max(vertex.position.z),
        },
      ),
    });

    if has_normal && (Self::length(&vertex.normal) - 1.0).abs() > NORMAL_LENGTH_TOLERANCE {
      self.stray_normals += 1;
    }

    self.widest_coordinate = self
      .widest_coordinate
      .max(vertex.texture_coordinate.0.abs())
      .max(vertex.texture_coordinate.1.abs());
  }

  /// A declaration as one word, so two buffers sharing a layout count as one spelling.
  fn describe(buffer: &LevelGeomVertexBuffer) -> String {
    match LevelVertexLayout::of(buffer) {
      Ok(layout) if layout.is_fastpath() => String::from("positions only"),
      Ok(layout) if layout.is_tree() => String::from("tree"),
      Ok(layout) if layout.is_lightmapped() => String::from("lightmapped"),
      Ok(_) => String::from("vertex lit"),
      Err(_) => String::from("unreadable"),
    }
  }

  fn length(vector: &Vector3d) -> f32 {
    (vector.x * vector.x + vector.y * vector.y + vector.z * vector.z).sqrt()
  }
}
