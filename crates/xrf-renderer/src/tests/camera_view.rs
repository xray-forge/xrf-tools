use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::camera::camera_view::CameraView;

#[test]
fn camera_view_reads_back_its_lens_depth_range() {
  let view: CameraView = CameraView::new(glam::Vec3::ZERO, glam::Mat4::IDENTITY, 67.5, 1.7, 0.2, 5000.0);
  let (near, far): (f32, f32) = view.get_depth_range();

  assert!((near - 0.2).abs() < 1e-4, "{near}");
  assert!((far - 5000.0).abs() < 1.0, "{far}");
  assert!(!view.is_orthographic());
}

#[test]
fn orthographic_view_reads_back_its_depth_range() {
  let view: CameraView = CameraView::new_orthographic(Vec3::ZERO, Mat4::IDENTITY, (300.0, 200.0), 1.0, 251.0);
  let (near, far): (f32, f32) = view.get_depth_range();

  assert!(view.is_orthographic());
  assert!((near - 1.0).abs() < 1e-3, "{near}");
  assert!((far - 251.0).abs() < 1e-3, "{far}");
}

#[test]
fn orthographic_view_reverses_depth_and_spans_its_half_size() {
  let view: CameraView = CameraView::new_orthographic(Vec3::ZERO, Mat4::IDENTITY, (300.0, 200.0), 1.0, 251.0);
  let project = |point: Vec3| -> Vec3 {
    let clip: Vec4 = view.get_view_projection() * point.extend(1.0);

    clip.truncate() / clip.w
  };

  // One at the near plane, zero at the far one, as the perspective's.
  assert!((project(Vec3::new(0.0, 0.0, -1.0)).z - 1.0).abs() < 1e-5);
  assert!(project(Vec3::new(0.0, 0.0, -251.0)).z.abs() < 1e-5);
  // Its edges at the half size whatever the depth, as parallel rays meet them.
  assert!((project(Vec3::new(300.0, 200.0, -10.0)).truncate() - Vec2::ONE).length() < 1e-5);
  assert!((project(Vec3::new(-300.0, -200.0, -200.0)).truncate() + Vec2::ONE).length() < 1e-5);
}

#[test]
fn jitter_moves_samples_alike_through_either_lens() {
  let size: Vec2 = Vec2::new(200.0, 100.0);
  let offset: Vec2 = Vec2::new(0.25, -0.5);
  let shift = |view: &CameraView, point: Vec3| -> Vec2 {
    let to_ndc = |matrix: Mat4| -> Vec2 {
      let clip: Vec4 = matrix * point.extend(1.0);

      clip.truncate().truncate() / clip.w
    };

    to_ndc(view.jittered(offset, size).get_view_projection()) - to_ndc(view.get_view_projection())
  };
  let perspective: CameraView = CameraView::new(Vec3::ZERO, Mat4::IDENTITY, 60.0, 2.0, 0.1, 1000.0);
  let orthographic: CameraView = CameraView::new_orthographic(Vec3::ZERO, Mat4::IDENTITY, (10.0, 5.0), 1.0, 100.0);

  // The same shift in normalized device coordinates at every depth, whichever lens.
  for depth in [2.0, 50.0] {
    let point: Vec3 = Vec3::new(0.5, 0.25, -depth);

    assert!((shift(&perspective, point) - shift(&orthographic, point)).length() < 1e-5);
  }
}

#[test]
fn a_view_reads_back_its_lens_field_of_view_or_its_box() {
  let perspective: CameraView = CameraView::new(Vec3::ZERO, Mat4::IDENTITY, 67.5, 1.7, 0.2, 5000.0);
  let orthographic: CameraView = CameraView::new_orthographic(Vec3::ZERO, Mat4::IDENTITY, (300.0, 200.0), 1.0, 251.0);

  assert!(
    perspective
      .get_field_of_view()
      .is_some_and(|degrees| (degrees - 67.5).abs() < 1e-3)
  );
  assert_eq!(perspective.get_orthographic_extent(), None);
  // Parallel rays do not spread; their box is twice the half size across and up, near to far deep.
  assert_eq!(orthographic.get_field_of_view(), None);
  assert!(
    orthographic
      .get_orthographic_extent()
      .is_some_and(|extent| (extent - Vec3::new(600.0, 400.0, 250.0)).length() < 1e-2)
  );
}
