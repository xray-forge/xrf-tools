use xrf_math::glam;
use xrf_math::glam::Vec3;

use crate::camera::camera_frustum::CameraFrustum;
use crate::camera::camera_view::CameraView;
use crate::contract::feature::lighting::render_shadow_settings::{RENDER_MAX_SHADOW_FILTER, RenderShadowSettings};
use crate::feature::lighting::shadows::cascades::cascade_draw::CascadeDraw;
use crate::feature::lighting::shadows::cascades::cascade_split_state::CascadeSplitState;
use crate::feature::lighting::shadows::level_shadows::LevelShadows;

/// A camera at `position` looking along `heading` degrees about the up axis and `pitch` degrees down, through `fov`.
fn camera(position: Vec3, (heading, pitch): (f32, f32), fov: f32) -> CameraView {
  let (heading, pitch): (f32, f32) = (heading.to_radians(), pitch.to_radians());
  let look: Vec3 = Vec3::new(heading.sin() * pitch.cos(), -pitch.sin(), -heading.cos() * pitch.cos());

  CameraView::new(
    position,
    glam::camera::rh::view::look_at_mat4(position, position + look, Vec3::Y),
    fov,
    1.7,
    0.2,
    5000.0,
  )
}

#[test]
fn samples_the_reach_a_narrow_view_sees() {
  let view: CameraView = camera(Vec3::new(64.0, -2.4, -232.8), (157.4, 5.9), 8.0);
  let points: Vec<Vec3> = LevelShadows::sample_seen(&view, 111.0);

  let frustum: CameraFrustum = view.get_frustum();

  assert!(!points.is_empty());
  assert!(points.iter().all(|point| frustum.reaches(point.extend(0.01))));
  assert!(points.iter().any(|point| point.distance(view.position) < 20.0));
  assert!(points.iter().all(|point| point.distance(view.position) <= 111.0 + 1e-3));
}

#[test]
fn samples_the_reach_a_wide_view_sees_near_and_at_its_end() {
  let view: CameraView = camera(Vec3::ZERO, (0.0, 10.0), 67.5);
  let points: Vec<Vec3> = LevelShadows::sample_seen(&view, 50.0);
  let frustum: CameraFrustum = view.get_frustum();

  assert!(points.iter().all(|point| frustum.reaches(point.extend(0.01))));
  assert!(points.iter().any(|point| point.distance(view.position) < 5.0));
  assert!(
    points
      .iter()
      .any(|point| (point.distance(view.position) - 50.0).abs() < 1e-3)
  );
}

#[test]
fn widens_the_nearest_cascades_bands_alone() {
  assert_eq!(
    LevelShadows::get_cascade_blends(0.08).to_array(),
    [0.2, 0.2, 0.08, 0.08]
  );
  assert_eq!(LevelShadows::get_cascade_blends(0.3).to_array(), [0.3, 0.3, 0.3, 0.3]);
}

/// A far cascade keeping a static layer that still holds, drawn and nothing due.
fn held(index: usize) -> CascadeSplitState {
  CascadeSplitState {
    index,
    is_held: true,
    can_wait: true,
    ..CascadeSplitState::default()
  }
}

fn choose(state: CascadeSplitState) -> Option<(CascadeDraw, bool)> {
  state.choose(&mut false)
}

#[test]
fn draws_nothing_while_the_static_layer_holds_and_nothing_sways_or_moves() {
  assert_eq!(choose(held(3)), None);
}

#[test]
fn draws_a_new_static_layer_where_the_held_one_went_stale() {
  let stale: CascadeSplitState = CascadeSplitState {
    is_held: false,
    is_stale: true,
    ..held(3)
  };

  assert_eq!(choose(stale), Some((CascadeDraw::Static, false)));
  // One swapped for the layer drawn ahead still holds: the trees are drawn over it.
  assert_eq!(
    choose(CascadeSplitState {
      is_stale: true,
      ..held(3)
    }),
    Some((CascadeDraw::Swaying, false))
  );
}

#[test]
fn sweeps_the_sway_band_by_band_at_once_on_the_near_cascades_alone() {
  let swayed = |index: usize| CascadeSplitState {
    is_swayed: true,
    ..held(index)
  };

  assert_eq!(choose(swayed(0)), Some((CascadeDraw::Banded(0), true)));
  assert_eq!(choose(swayed(3)), Some((CascadeDraw::Banded(0), false)));
  // A sweep under way finishes though the sway no longer shows.
  assert_eq!(
    choose(CascadeSplitState {
      sweep: Some(2),
      ..held(3)
    }),
    Some((CascadeDraw::Banded(2), false))
  );
}

#[test]
fn draws_what_moves_every_frame_over_the_kept_layer_while_the_trees_wait() {
  let moving: CascadeSplitState = CascadeSplitState {
    has_moving: true,
    is_kept: true,
    ..held(1)
  };

  assert_eq!(choose(moving), Some((CascadeDraw::Kept, true)));
  // Not kept yet: its trees drawn over its static layer with what moves, at once.
  assert_eq!(
    choose(CascadeSplitState {
      is_kept: false,
      ..moving
    }),
    Some((CascadeDraw::Swaying, true))
  );
}

// Two kept cascades whose trees are due in one frame: the first redraws them, the second keeps its layer a frame,
// unless it has waited its most.
#[test]
fn spreads_two_kept_cascades_tree_redraws_over_frames() {
  let swayed = |index: usize| CascadeSplitState {
    has_moving: true,
    is_kept: true,
    is_swayed: true,
    ..held(index)
  };
  let mut is_trees_taken: bool = false;

  assert_eq!(
    swayed(0).choose(&mut is_trees_taken),
    Some((CascadeDraw::Swaying, true))
  );
  assert!(is_trees_taken);
  assert_eq!(swayed(1).choose(&mut is_trees_taken), Some((CascadeDraw::Kept, true)));
  assert_eq!(
    CascadeSplitState {
      can_wait: false,
      ..swayed(1)
    }
    .choose(&mut is_trees_taken),
    Some((CascadeDraw::Swaying, true))
  );
}

// What moved every frame left the cascade: drawn once more without it, at once, so its shadow does not stay behind.
#[test]
fn clears_what_moved_every_frame_once_it_leaves_the_cascade() {
  assert_eq!(
    choose(CascadeSplitState {
      has_drawn_moving: true,
      ..held(2)
    }),
    Some((CascadeDraw::Swaying, true))
  );
}

// Widths and a filter from a stored setting or a page are bounded where they are read: a width of nought made a NaN
// cascade that redrew every frame.
#[test]
fn reads_the_cascades_and_the_filter_within_their_bounds() {
  let settings: RenderShadowSettings = RenderShadowSettings {
    cascades: vec![0.0, f32::NAN, f32::INFINITY, 20.0],
    filter: 9,
    ..RenderShadowSettings::default()
  };

  assert_eq!(
    (0..5)
      .map(|index| settings.get_cascade_width(index))
      .collect::<Vec<f32>>(),
    [1.0, 1.0, 10_000.0, 20.0, 1.0]
  );
  assert_eq!(settings.get_filter(), RENDER_MAX_SHADOW_FILTER);
  assert_eq!(settings.get_reach(2), Some(1.0));
}
