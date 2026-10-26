use std::collections::BTreeMap;

use serde::Serialize;
use xrf_math::Vector3d;
use xrf_ogf::{OgfGeometryContainerChunk, OgfModelType};

use crate::{LevelGeomVertexBuffer, LevelGeometryLayoutKind, LevelVertex, LevelVertexLayout};

/// A normal the compiler wrote is a unit vector, so one whose length misses by more than this came out of the wrong
/// element rather than out of rounding.
const NORMAL_LENGTH_TOLERANCE: f32 = 0.1;

/// What a level held, beside what the sweep found.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGeometryCensus {
  pub visuals: usize,
  pub drawable_visuals: usize,
  pub fastpath_visuals: usize,
  pub vertices: u64,
  pub indices: u64,
  pub shader_entries: usize,
  /// Visuals of each engine model type, by `MT_*` identifier.
  pub model_types: BTreeMap<String, usize>,
  /// Visuals stored in each kind of vertex declaration.
  pub layouts: BTreeMap<LevelGeometryLayoutKind, usize>,
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
  /// Visuals a sector reaches from its root, counting one reached by two sectors twice.
  pub sector_visuals: usize,
  /// Drawable visuals no sector reaches, which nothing culled by sector would ever draw.
  pub orphaned_drawables: usize,
  /// Drawable visuals reached from more than one sector, which a packer keyed by sector would pack twice.
  pub shared_drawables: usize,
  /// Whether the compiler wrote the directional light the loader keeps as the sun.
  pub has_sun: bool,
}

impl LevelGeometryCensus {
  /// Counts one visual by the kind its header declares.
  ///
  /// A level holds thousands of visuals and a dozen kinds, so a kind already counted is found by its static name and
  /// only the first of each allocates its key.
  pub(crate) fn record_visual(&mut self, model_type: u8) {
    self.visuals += 1;

    let known: Option<&mut usize> = OgfModelType::from_raw(model_type)
      .map(OgfModelType::name)
      .and_then(|name| self.model_types.get_mut(name));

    match known {
      Some(count) => *count += 1,
      None => *self.model_types.entry(OgfModelType::label(model_type)).or_default() += 1,
    }
  }

  /// Counts what one visual draws from the level's ordinary buffers.
  pub(crate) fn record_drawable(&mut self, container: &OgfGeometryContainerChunk) {
    self.drawable_visuals += 1;
    self.vertices += u64::from(container.vertex_count);
    self.indices += u64::from(container.index_count);
  }

  /// Counts the declaration a visual's geometry is stored in, and says whether it carries a normal to judge.
  pub(crate) fn record_layout(&mut self, buffer: &LevelGeomVertexBuffer) -> bool {
    *self.layouts.entry(LevelGeometryLayoutKind::of(buffer)).or_default() += 1;

    LevelVertexLayout::of(buffer).is_ok_and(|layout| layout.get_normal_offset().is_some())
  }

  /// Folds one decoded vertex into the extent, the normal count and the widest coordinate.
  pub(crate) fn record_vertex(&mut self, vertex: &LevelVertex, has_normal: bool) {
    self.bounds = Some(match self.bounds.take() {
      None => (vertex.position.clone(), vertex.position.clone()),
      Some((min, max)) => (min.min(&vertex.position), max.max(&vertex.position)),
    });

    if has_normal && (vertex.normal.length() - 1.0).abs() > NORMAL_LENGTH_TOLERANCE {
      self.stray_normals += 1;
    }

    self.widest_coordinate = self
      .widest_coordinate
      .max(vertex.texture_coordinate.0.abs())
      .max(vertex.texture_coordinate.1.abs());
  }
}
