use glam::Vec3;

use crate::scene::static_scene::trace::static_trace_geometry::StaticTraceGeometry;

/// A top-level instance of a tree's geometry, which the wind sways: its index in the top level, its place, where it
/// stands, in renderer space, its instance's transform and its geometry.
#[derive(Clone, Copy, Debug)]
pub struct StaticTraceSwaying {
  pub index: u32,
  pub place: u32,
  pub position: Vec3,
  pub transform: [f32; 12],
  pub geometry: StaticTraceGeometry,
}
