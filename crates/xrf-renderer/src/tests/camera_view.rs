use crate::camera::camera_view::CameraView;

#[test]
fn camera_view_reads_back_its_lens_depth_range() {
  let view: CameraView = CameraView::new(glam::Vec3::ZERO, glam::Mat4::IDENTITY, 67.5, 1.7, 0.2, 5000.0);
  let (near, far): (f32, f32) = view.get_depth_range();

  assert!((near - 0.2).abs() < 1e-4, "{near}");
  assert!((far - 5000.0).abs() < 1.0, "{far}");
}
