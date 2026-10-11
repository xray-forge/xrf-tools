use std::f32::consts::TAU;

use crate::scene::texture::height_field::{integrate_normals, to_unit_heights};

const SIZE: usize = 64;

/// A tiling surface of a few waves across and down, in texels.
fn create_surface() -> Vec<f32> {
  (0..SIZE * SIZE)
    .map(|index| {
      let (x, y) = ((index % SIZE) as f32, (index / SIZE) as f32);

      3.0 * (TAU * x / SIZE as f32).sin() * (TAU * 2.0 * y / SIZE as f32).cos()
        + 1.5 * (TAU * 3.0 * (x + y) / SIZE as f32).sin()
    })
    .collect()
}

/// A surface's normals by its forward differences, wrapped, `y` taken with `sign`.
fn to_normals(surface: &[f32], sign: f32) -> Vec<[f32; 3]> {
  (0..SIZE * SIZE)
    .map(|index| {
      let (x, y) = (index % SIZE, index / SIZE);
      let across: f32 = surface[y * SIZE + (x + 1) % SIZE] - surface[index];
      let down: f32 = surface[((y + 1) % SIZE) * SIZE + x] - surface[index];
      let length: f32 = (across * across + down * down + 1.0).sqrt();

      [-across / length, -down * sign / length, 1.0 / length]
    })
    .collect()
}

fn get_largest_error(surface: &[f32], heights: &[f32]) -> f32 {
  let mean: f32 = surface.iter().sum::<f32>() / surface.len() as f32;

  surface
    .iter()
    .zip(heights)
    .map(|(expected, found)| (expected - mean - found).abs())
    .fold(0.0, f32::max)
}

#[test]
fn height_field_gives_back_the_surface_its_normals_were_taken_from() {
  let surface: Vec<f32> = create_surface();
  let heights: Vec<f32> = integrate_normals(SIZE, SIZE, &to_normals(&surface, 1.0), 0.0);

  assert!(get_largest_error(&surface, &heights) < 1e-2);
}

#[test]
fn height_field_reads_a_normal_map_whose_y_runs_up_the_texture() {
  let surface: Vec<f32> = create_surface();
  let heights: Vec<f32> = integrate_normals(SIZE, SIZE, &to_normals(&surface, -1.0), 0.0);

  assert!(get_largest_error(&surface, &heights) < 1e-2);
}

#[test]
fn height_field_scales_heights_about_a_half_within_the_unit() {
  let unit: Vec<f32> = to_unit_heights(&create_surface());
  let mean: f32 = unit.iter().sum::<f32>() / unit.len() as f32;

  assert!((mean - 0.5).abs() < 1e-3);
  assert!(unit.iter().all(|it| (0.0..=1.0).contains(it)));
  assert!(unit.iter().any(|it| *it <= 0.01 || *it >= 0.99));
}

#[test]
fn height_field_fades_relief_broader_than_its_low_cut() {
  // One wave across and down, and eleven: the first faded nearly whole, the second all but kept.
  let broad = |index: usize| (TAU * (index % SIZE) as f32 / SIZE as f32).sin() * 4.0;
  let fine = |index: usize| (TAU * 11.0 * (index / SIZE) as f32 / SIZE as f32).sin();
  let surface: Vec<f32> = (0..SIZE * SIZE).map(|index| broad(index) + fine(index)).collect();
  let only_fine: Vec<f32> = (0..SIZE * SIZE).map(fine).collect();
  let heights: Vec<f32> = integrate_normals(SIZE, SIZE, &to_normals(&surface, 1.0), 8.0);

  assert!(get_largest_error(&only_fine, &heights) < 0.25);
}
