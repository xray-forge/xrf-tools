//! The engine's bloom weights and offsets, as `phase_bloom` sets them.

use glam::Vec4;

use crate::pass::bloom_uniform::BloomUniform;

/// All fifteen taps a filter reads: the centre once, every other weight on either side.
fn get_total(weights: &[Vec4; 2]) -> f32 {
  let [near, far] = weights;

  far.w + 2.0 * (near.element_sum() + far.truncate().element_sum())
}

#[test]
fn a_blur_weighs_twice_its_strength_over_both_kernels() {
  // OpenXRay's defaults: `kernel_g 3`, `kernel_scale 0.7`; each of `CalcGauss_wave`'s two kernels sums to the strength.
  let weights: [Vec4; 2] = BloomUniform::get_wave(3.0, 0.7);

  assert!((get_total(&weights) - 1.4).abs() < 1e-5);
  // The centre weighs most, and the taps fall away from it.
  assert!(weights[1][3] > weights[0][0]);
  assert!(weights[0][0] > weights[0][3] && weights[0][3] > weights[1][2]);
}

#[test]
fn the_blur_down_takes_the_radius_by_the_frames_height_over_its_width() {
  let across: BloomUniform = BloomUniform::filter(true, (3.0, 0.7), 0.5, false);
  let down: BloomUniform = BloomUniform::filter(false, (3.0, 0.7), 0.5, false);

  assert_eq!(across.params, Vec4::new(1.0 / 256.0, 0.0, 0.0, 0.0));
  assert_eq!(down.params, Vec4::new(0.0, 1.0 / 256.0, 0.0, 0.0));
  assert_eq!(across.weights, BloomUniform::get_wave(3.0, 0.7));
  assert_eq!(down.weights, BloomUniform::get_wave(1.5, 0.7));
}

#[test]
fn monolith_reads_one_side_and_the_build_offsets_half_a_texel_of_the_frame() {
  assert_eq!(BloomUniform::filter(true, (1.0, 0.05), 1.0, true).params[2], 1.0);
  assert_eq!(
    BloomUniform::build((1000, 500), 0.25).params,
    Vec4::new(0.0005, 0.001, 0.25, 0.0)
  );
}
