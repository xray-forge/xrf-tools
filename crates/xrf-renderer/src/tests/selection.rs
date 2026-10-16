use glam::{Mat4, UVec4, Vec3, Vec4};

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

  assert_eq!(uniform.selection, UVec4::ZERO);
}

#[test]
fn packs_a_place_alone_as_every_cluster_of_it() {
  let uniform: CameraUniform = camera().with_selection(Some(&StaticSelection {
    place: Some(7),
    runs: vec![StaticSelection::ANY_CLUSTER],
  }));

  assert_eq!(uniform.selection, UVec4::new(1, 7, 1, 0));
  assert_eq!(uniform.selection_runs[0], UVec4::new(0, u32::MAX, 0, 0));
}

#[test]
fn packs_runs_two_a_row() {
  let runs: Vec<[u32; 2]> = (0..10).map(|index| [index * 10, index * 10 + 5]).collect();
  let uniform: CameraUniform = camera().with_selection(Some(&StaticSelection { place: None, runs }));

  assert_eq!(uniform.selection, UVec4::new(1, u32::MAX, 10, 0));
  assert_eq!(uniform.selection_runs[0], UVec4::new(0, 5, 10, 15));
  assert_eq!(uniform.selection_runs[4], UVec4::new(80, 85, 90, 95));
}

// A shader's baked geometry in a sector can be cut into many slots: past the runs the camera holds, the nearest join,
// rather than the last reaching over every cluster after it.
#[test]
fn joins_the_runs_nearest_each_other_past_what_it_holds() {
  let mut runs: Vec<[u32; 2]> = (0..64).map(|index| [index * 100, index * 100 + 10]).collect();

  runs.push([6_311, 6_320]);

  let uniform: CameraUniform = camera().with_selection(Some(&StaticSelection { place: None, runs }));

  assert_eq!(uniform.selection[2], 64);
  assert_eq!(uniform.selection_runs[31], UVec4::new(6_200, 6_210, 6_300, 6_320));
}

// Joined at once, the runs part where joining the narrowest gap one at a time, the earlier among equals, leaves them.
#[test]
fn joins_as_the_narrowest_gap_joined_one_at_a_time_would() {
  let mut at: u32 = 0;
  let runs: Vec<[u32; 2]> = (0..300_u32)
    .map(|index| {
      let start: u32 = at + (index * 7_919) % 13;

      at = start + 1 + index % 5;
      [start, at]
    })
    .collect();
  let mut expected: Vec<[u32; 2]> = runs.clone();

  while expected.len() > 64 {
    let narrowest: usize = (1..expected.len())
      .min_by_key(|index| expected[*index][0].saturating_sub(expected[index - 1][1]))
      .unwrap();

    expected[narrowest - 1][1] = expected[narrowest][1];
    expected.remove(narrowest);
  }

  let uniform: CameraUniform = camera().with_selection(Some(&StaticSelection { place: None, runs }));
  let joined: Vec<[u32; 2]> = (0..64)
    .map(|index| {
      let row: UVec4 = uniform.selection_runs[index / 2];

      if index % 2 == 0 { [row.x, row.y] } else { [row.z, row.w] }
    })
    .collect();

  assert_eq!(uniform.selection[2], 64);
  assert_eq!(joined, expected);
}
