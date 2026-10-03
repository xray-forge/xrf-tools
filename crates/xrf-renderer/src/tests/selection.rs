use glam::{Mat4, Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::contract::render_rect::RenderRect;
use crate::pass::camera_uniform::CameraUniform;
use crate::scene::static_scene::static_selection::StaticSelection;

fn camera() -> CameraUniform {
  let view: CameraView = CameraView::new(Vec3::ZERO, Mat4::IDENTITY, 60.0, 1.0, 0.5, 100.0);

  CameraUniform::new(&view, RenderRect::default(), Vec4::ZERO)
}

#[test]
fn marks_nothing_without_a_selection() {
  let uniform: CameraUniform = camera().with_selection(None);

  assert_eq!(uniform.selection, [0; 4]);
}

#[test]
fn packs_a_place_alone_as_every_cluster_of_it() {
  let uniform: CameraUniform = camera().with_selection(Some(&StaticSelection {
    place: Some(7),
    runs: vec![StaticSelection::ANY_CLUSTER],
  }));

  assert_eq!(uniform.selection, [1, 7, 1, 0]);
  assert_eq!(uniform.selection_runs[0], [0, u32::MAX, 0, 0]);
}

#[test]
fn packs_runs_two_a_row() {
  let runs: Vec<[u32; 2]> = (0..10).map(|index| [index * 10, index * 10 + 5]).collect();
  let uniform: CameraUniform = camera().with_selection(Some(&StaticSelection { place: None, runs }));

  assert_eq!(uniform.selection, [1, u32::MAX, 10, 0]);
  assert_eq!(uniform.selection_runs[0], [0, 5, 10, 15]);
  assert_eq!(uniform.selection_runs[4], [80, 85, 90, 95]);
}

// A shader's baked geometry in a sector can be cut into many slots: past the runs the camera holds, the nearest join,
// rather than the last reaching over every cluster after it.
#[test]
fn joins_the_runs_nearest_each_other_past_what_it_holds() {
  let mut runs: Vec<[u32; 2]> = (0..64).map(|index| [index * 100, index * 100 + 10]).collect();

  runs.push([6_311, 6_320]);

  let uniform: CameraUniform = camera().with_selection(Some(&StaticSelection { place: None, runs }));

  assert_eq!(uniform.selection[2], 64);
  assert_eq!(uniform.selection_runs[31], [6_200, 6_210, 6_300, 6_320]);
}
