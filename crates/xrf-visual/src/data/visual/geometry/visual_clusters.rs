use serde::Serialize;

use crate::data::visual::geometry::visual_section::VisualSection;

/// Where a geometry's clusters sit in its buffer: runs of up to [`VisualClusters::MAX_TRIANGLES`] consecutive
/// triangles of its index order, never crossing from one drawable into the next, which a renderer culls and draws each
/// on its own.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualClusters {
  pub count: u32,
  /// Four unsigned integers a cluster: its first index, in the geometry's indices; its triangles; the drawable it is
  /// cut from, by its index in the visuals run, or [`VisualClusters::NO_DRAWABLE`]; then nothing.
  pub ranges: VisualSection,
  /// Four floats a cluster: the centre and radius of a sphere holding its vertices, in the space of the positions.
  pub spheres: VisualSection,
}

impl VisualClusters {
  /// Triangles a cluster holds at most: a draw of a cluster is this many triangles, those past its own discarded.
  pub const MAX_TRIANGLES: u32 = 128;
  /// Unsigned integers one cluster's range takes.
  pub const RANGE_WORDS: usize = 4;
  /// Floats one cluster's sphere takes.
  pub const SPHERE_FLOATS: usize = 4;
  /// What a cluster names for its drawable where it is cut from a mesh many drawables stand in.
  pub const NO_DRAWABLE: u32 = u32::MAX;
}
