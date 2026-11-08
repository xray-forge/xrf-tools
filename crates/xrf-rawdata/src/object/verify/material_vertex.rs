use std::collections::BTreeMap;

use xrf_math::{Vector2d, Vector3d};
use xrf_ogf::{OgfStaticVertex, OgfVertex};

use crate::object::compile::geometry::export_vertex_bone::ExportVertexBone;

/// A vertex as the comparison reads it, from a skinned child or a static one: where it stands, its normal, its texture
/// coordinate, and the bones it follows with their weights, none for a static vertex.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MaterialVertex {
  pub(crate) position: Vector3d,
  pub(crate) normal: Vector3d,
  pub(crate) texture_coordinate: Vector2d,
  /// By ascending bone, each vertex's padding links summed away: a layout pads a narrower vertex with repeated or
  /// empty links, which the engine blends to the same pose.
  pub(crate) bones: Vec<ExportVertexBone>,
}

impl MaterialVertex {
  /// A summed weight at or below which a bone is padding.
  const NEGLIGIBLE_WEIGHT: f32 = 1e-6;
}

impl From<&OgfVertex> for MaterialVertex {
  fn from(vertex: &OgfVertex) -> Self {
    let mut summed: BTreeMap<u16, f32> = BTreeMap::new();

    for link in &vertex.links {
      *summed.entry(link.bone).or_default() += link.weight;
    }

    Self {
      position: vertex.position,
      normal: vertex.normal,
      texture_coordinate: Vector2d::new(vertex.texture_u, vertex.texture_v),
      bones: summed
        .into_iter()
        .filter(|(_, weight)| *weight > Self::NEGLIGIBLE_WEIGHT)
        .map(|(bone, weight)| ExportVertexBone { bone, weight })
        .collect(),
    }
  }
}

impl From<&OgfStaticVertex> for MaterialVertex {
  /// A layout without a normal or a texture coordinate set reads as a zero one, which no exported vertex carries.
  fn from(vertex: &OgfStaticVertex) -> Self {
    Self {
      position: vertex.position,
      normal: vertex.normal.unwrap_or(Vector3d::ZERO),
      texture_coordinate: match vertex.texture_coordinates.first().map(Vec::as_slice) {
        Some([u, v, ..]) => Vector2d::new(*u, *v),
        _ => Vector2d::ZERO,
      },
      bones: Vec::new(),
    }
  }
}
