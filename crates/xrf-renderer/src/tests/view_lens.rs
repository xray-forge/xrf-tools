use std::borrow::Cow;

use glam::{Mat4, Vec3};

use crate::camera::camera_view::CameraView;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_scale::RenderScale;
use crate::contract::render_view_options::RenderViewOptions;
use crate::scene::level::view_lens::ViewLens;

/// Options asking for everything a parallel lens cannot draw.
fn asking_everything() -> RenderViewOptions {
  let mut options: RenderViewOptions = RenderViewOptions::default();

  options.show.is_sky_visible = true;
  options.show.is_lens_flared = true;
  options.show.is_sun_shafted = true;
  options.features.is_occlusion_culled = true;
  options.features.lod.is_impostors = true;
  options.features.antialiasing = RenderAntialiasing::Dlss;
  options.output.upscaling.scale = RenderScale::Quality;
  options.features.shadows.cascades = vec![20.0, 60.0, 200.0];

  options
}

#[test]
fn a_perspective_draws_what_its_view_asks() {
  let options: RenderViewOptions = asking_everything();
  let view: CameraView = CameraView::new(Vec3::ZERO, Mat4::IDENTITY, 67.5, 1.7, 0.2, 5000.0);

  assert!(matches!(ViewLens::fit(&options, &view), Cow::Borrowed(_)));
}

#[test]
fn parallel_rays_draw_without_what_needs_an_eye_whichever_view_they_are() {
  let options: RenderViewOptions = asking_everything();
  let view: CameraView = CameraView::new_orthographic(Vec3::ZERO, Mat4::IDENTITY, (300.0, 200.0), 1.0, 251.0);
  let fitted: RenderViewOptions = ViewLens::fit(&options, &view).into_owned();

  assert!(!fitted.show.is_sky_visible && !fitted.show.is_lens_flared && !fitted.show.is_sun_shafted);
  assert!(!fitted.features.is_occlusion_culled && !fitted.features.lod.is_impostors);
  assert!(!fitted.features.reflections.is_drawn() && !fitted.features.shadows.contact.is_drawn());
  assert_eq!(fitted.features.antialiasing, RenderAntialiasing::Taa);
  assert_eq!(fitted.output.upscaling.scale, RenderScale::Native);
  // One cascade holding the whole box, and the grass ring out to its corners.
  assert_eq!(fitted.features.shadows.cascades.len(), 1);
  assert!((fitted.features.shadows.cascades[0] - Vec3::new(600.0, 400.0, 250.0).length()).abs() < 1e-2);
  assert_eq!(fitted.features.grass.radius, (600.0f32.hypot(400.0) * 0.5).ceil());
  // What the lens has no say in stands as asked.
  assert_eq!(fitted.mode, options.mode);
  assert_eq!(fitted.show.is_fogged, options.show.is_fogged);
}
