//! Which trees a view sways in its own top level.

use glam::Vec3;

use crate::feature::tracing::view_trace::view_trace_feature::ViewTraceFeature;
use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;
use crate::scene::static_scene::trace::static_trace_geometry::StaticTraceGeometry;
use crate::scene::static_scene::trace::static_trace_swaying::StaticTraceSwaying;

/// A tree standing `x` metres along from the camera, its instance `index`.
fn create_tree(index: u32, x: f32) -> StaticTraceSwaying {
  StaticTraceSwaying {
    index,
    place: index,
    position: Vec3::new(x, 0.0, 0.0),
    transform: [0.0; 12],
    geometry: StaticTraceGeometry {
      layout: StaticLayout::Tree,
      first_index: 0,
      index_count: 3,
      vertex_start: 0,
      vertex_count: 3,
      surface: 0,
      is_cut_out: true,
      first_density: 0,
    },
  }
}

/// The wind sways the near trees, nearest first, and none past the reach; with no wind none is swayed, each standing
/// as the scene's trace holds it, so the view keeps the scene's shared structures.
#[test]
fn sways_the_near_trees_only_while_the_wind_blows() {
  let trees: [StaticTraceSwaying; 3] = [create_tree(0, 30.0), create_tree(1, 10.0), create_tree(2, 500.0)];
  let chosen = |amplitude: f32| -> Vec<u32> {
    ViewTraceFeature::choose(&trees, Vec3::ZERO, amplitude)
      .iter()
      .map(|tree| tree.index)
      .collect()
  };

  assert_eq!(chosen(0.2), vec![1, 0]);
  assert!(chosen(0.0).is_empty());
}
