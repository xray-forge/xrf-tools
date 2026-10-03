use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::frame::temporal_jitter::{TemporalJitter, halton};

#[test]
fn halton_places_samples_as_the_radical_inverse_of_their_index() {
  assert_eq!(halton(1, 2), 0.5);
  assert_eq!(halton(2, 2), 0.25);
  assert_eq!(halton(3, 2), 0.75);
  assert!((halton(1, 3) - 1.0 / 3.0).abs() < 1e-6);
  assert!((halton(2, 3) - 2.0 / 3.0).abs() < 1e-6);
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
