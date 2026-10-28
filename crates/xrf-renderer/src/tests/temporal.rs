use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_view_options::RenderViewOptions;
use crate::scene::level::feature::resolve_feature::ResolveFeature;
use crate::scene::level::feature::temporal_jitter::TemporalJitter;

#[test]
fn halton_places_samples_as_the_radical_inverse_of_their_index() {
  assert_eq!(TemporalJitter::halton(1, 2), 0.5);
  assert_eq!(TemporalJitter::halton(2, 2), 0.25);
  assert_eq!(TemporalJitter::halton(3, 2), 0.75);
  assert!((TemporalJitter::halton(1, 3) - 1.0 / 3.0).abs() < 1e-6);
  assert!((TemporalJitter::halton(2, 3) - 2.0 / 3.0).abs() < 1e-6);
}

// Eight places a cycle, each within half a pixel of the centre, none repeated.
#[test]
fn jitter_cycles_through_eight_places_within_the_pixel() {
  let mut jitter: TemporalJitter = TemporalJitter::default();
  let places: Vec<Vec2> = (0..8).map(|_| jitter.next(1.0)).collect();

  assert!(places.iter().all(|it| it.abs().max_element() < 0.5));
  assert_eq!(jitter.next(1.0), places[0]);

  for (index, place) in places.iter().enumerate() {
    assert!(places[index + 1..].iter().all(|other| other != place));
  }
}

// Upscaled twice across, a pixel's samples cycle through four times the places, so each output pixel still gets eight.
#[test]
fn jitter_cycles_through_more_places_the_more_a_frame_is_upscaled() {
  let mut jitter: TemporalJitter = TemporalJitter::default();
  let first: Vec2 = jitter.next(2.0);

  assert!((1..32).all(|_| jitter.next(2.0) != first));
  assert_eq!(jitter.next(2.0), first);
}

// Each texel samples the scene the jitter's pixels from its centre, `y` down, so a point lands as far the other way, at
// any depth.
#[test]
fn a_jittered_view_samples_each_pixel_the_jitter_from_its_centre() {
  let view: CameraView = CameraView::new(Vec3::ZERO, Mat4::IDENTITY, 60.0, 2.0, 0.1, 100.0);
  let size: Vec2 = Vec2::new(800.0, 400.0);
  let jittered: CameraView = view.jittered(Vec2::new(0.25, -0.5), size);

  for depth in [1.0, 10.0, 90.0] {
    let point: Vec4 = Vec4::new(3.0, 1.0, -depth, 1.0);
    let still: Vec4 = view.get_view_projection() * point;
    let moved: Vec4 = jittered.get_view_projection() * point;
    let ndc: Vec2 = moved.truncate().truncate() / moved.w - still.truncate().truncate() / still.w;
    let shift: Vec2 = Vec2::new(ndc.x, -ndc.y) * size / 2.0;

    assert!((shift.x + 0.25).abs() < 1e-3, "{shift}");
    assert!((shift.y - 0.5).abs() < 1e-3, "{shift}");
  }
}

// The prepared view is unjittered and NGX takes the jitter apart, in FSR's sense (a drawn texel `m` stands at
// `m + 0.5 - jitter`): its camera puts each point where the drawn frame's jitter, taken out once, says it stands.
#[test]
fn ray_reconstruction_places_hits_through_the_drawn_camera_less_its_jitter() {
  let view: CameraView = CameraView::new(Vec3::ZERO, Mat4::IDENTITY, 60.0, 2.0, 0.1, 100.0);
  let size: Vec2 = Vec2::new(800.0, 400.0);
  let offset: Vec2 = Vec2::new(0.25, -0.5);
  let drawn: CameraView = view.jittered(offset, size);
  let jitter: Vec2 = -offset;
  let (world_to_view, view_to_clip): (Mat4, Mat4) = ResolveFeature::get_ray_camera(&view);
  let to_pixel = |clip: Vec4| Vec2::new(clip.x / clip.w + 1.0, 1.0 - clip.y / clip.w) * size / 2.0;

  for depth in [1.0, 10.0, 90.0] {
    let point: Vec4 = Vec4::new(3.0, 1.0, -depth, 1.0);
    let placed: Vec2 = to_pixel(view_to_clip * world_to_view * point);
    let stood: Vec2 = to_pixel(drawn.get_view_projection() * point) - jitter;

    assert!((placed - stood).abs().max_element() < 1e-3, "{placed} {stood}");
  }
}

// FSR and DLSS read textures at AMD's `log2(render / display) - 1`, TAA at the size lost alone, nothing else biased.
#[test]
fn reads_textures_sharper_by_what_the_upscale_loses() {
  let fsr: f32 = ResolveFeature::get_bias(Some(RenderAntialiasing::Fsr), 1.5);

  assert!((fsr - (1.5f32.recip().log2() - 1.0)).abs() < 1e-6);
  assert_eq!(ResolveFeature::get_bias(Some(RenderAntialiasing::Dlss), 2.0), -2.0);
  assert_eq!(ResolveFeature::get_bias(Some(RenderAntialiasing::Taa), 2.0), -1.0);
  assert_eq!(ResolveFeature::get_bias(Some(RenderAntialiasing::Taa), 1.0), 0.0);
  assert_eq!(ResolveFeature::get_bias(Some(RenderAntialiasing::Fsr), 1.0), -1.0);
  assert_eq!(ResolveFeature::get_bias(None, 2.0), 0.0);
}

// A reference's samples stand at their own places of the sequence, unresolved, read as sharp as DLSS reads them, the
// screen-space effects' noise turned by the sample, so every instant averages the same turns.
#[test]
fn a_reference_sample_is_drawn_unresolved_at_its_place_of_the_jitter() {
  let mut options: RenderViewOptions = RenderViewOptions::default();
  let mut resolve: ResolveFeature = ResolveFeature::default();

  options.features.antialiasing = RenderAntialiasing::Taa;
  resolve.set_reference(Some((5, 64)));

  assert_eq!(resolve.next_jitter(&options, 2.0, false), TemporalJitter::at(5));
  assert_eq!(resolve.get_temporal(), None);
  assert_eq!(resolve.get_texture_bias(), -1.0);
  assert_eq!(resolve.get_reference_sample(), Some(5));
  assert_eq!(resolve.get_noise_frame(), 5);

  resolve.set_reference(Some((2, 64)));
  resolve.next_jitter(&options, 2.0, false);

  assert_eq!(resolve.get_noise_frame(), 2);

  resolve.set_reference(None);

  assert_eq!(resolve.get_reference_sample(), None);

  assert_eq!(resolve.get_temporal(), None);
  assert_ne!(resolve.next_jitter(&options, 1.0, false), TemporalJitter::at(5));
  assert_eq!(resolve.get_temporal(), Some(RenderAntialiasing::Taa));
}

// A restarted resolve's jitter starts its sequence again, so two runs from a restart sample alike.
#[test]
fn a_restarted_resolve_jitters_from_the_first_place_again() {
  let mut options: RenderViewOptions = RenderViewOptions::default();
  let mut resolve: ResolveFeature = ResolveFeature::default();

  options.features.antialiasing = RenderAntialiasing::Taa;

  let first: Vec<Vec2> = (0..3).map(|_| resolve.next_jitter(&options, 1.0, false)).collect();

  resolve.restart();

  let again: Vec<Vec2> = (0..3).map(|_| resolve.next_jitter(&options, 1.0, false)).collect();

  assert_eq!(first, again);
  assert_eq!(resolve.get_noise_frame(), 0);
}

// A camera that walks or turns a little a frame keeps its history; one that jumps further than a frame's travel or
// turns past a third of a turn cut away, and the next frame's camera is measured from where it landed.
#[test]
fn a_jump_or_a_sharp_turn_is_a_cut() {
  let mut resolve: ResolveFeature = ResolveFeature::default();
  let at = |position: Vec3, yaw: f32| {
    CameraView::new(
      position,
      Mat4::from_rotation_y(-yaw) * Mat4::from_translation(-position),
      60.0,
      1.0,
      0.1,
      100.0,
    )
  };

  assert!(!resolve.is_cut(&at(Vec3::ZERO, 0.0)));
  assert!(!resolve.is_cut(&at(Vec3::new(0.5, 0.0, 0.0), 0.2)));
  assert!(resolve.is_cut(&at(Vec3::new(40.0, 0.0, 0.0), 0.2)));
  assert!(!resolve.is_cut(&at(Vec3::new(40.5, 0.0, 0.0), 0.3)));
  assert!(resolve.is_cut(&at(Vec3::new(40.5, 0.0, 0.0), 2.0)));

  resolve.restart();

  assert!(!resolve.is_cut(&at(Vec3::new(-100.0, 0.0, 0.0), 0.0)));
}
