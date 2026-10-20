use glam::Vec3;

use crate::camera::camera_frustum::CameraFrustum;
use crate::camera::camera_view::CameraView;
use crate::contract::render_shadow_settings::{RENDER_MAX_SHADOW_FILTER, RenderShadowSettings};
use crate::scene::level::level_shadows::LevelShadows;

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
fn samples_the_grass_a_narrow_view_sees() {
  let view: CameraView = camera(Vec3::new(64.0, -2.4, -232.8), (157.4, 5.9), 8.0);
  let points: Vec<Vec3> = LevelShadows::sample_seen_grass(&view, 111.0);

  let frustum: CameraFrustum = view.get_frustum();

  assert!(!points.is_empty());
  assert!(points.iter().all(|point| frustum.reaches(point.extend(0.01))));
  assert!(points.iter().any(|point| point.distance(view.position) < 20.0));
  assert!(points.iter().all(|point| point.distance(view.position) <= 111.0 + 1e-3));
}

#[test]
fn samples_the_grass_a_wide_view_sees_near_and_at_its_reach() {
  let view: CameraView = camera(Vec3::ZERO, (0.0, 10.0), 67.5);
  let points: Vec<Vec3> = LevelShadows::sample_seen_grass(&view, 50.0);
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
  assert_eq!(LevelShadows::get_grass_reach(&settings, 2), Some(1.0));
}
