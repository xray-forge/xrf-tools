use image::imageops::{self, FilterType};
use xrf_dds::{Rgba, RgbaImage};

use crate::alignment::{BumpAlignment, BumpAlignmentVerdict};

/// A tiling relief of soft mounds `side` texels a side, laid out by `seed`, from nought to one.
fn relief(side: u32, seed: u32) -> Vec<f32> {
  let mut state: u32 = seed;
  let mut next = || {
    state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);

    (state >> 8) as f32 / (1u32 << 24) as f32
  };
  let mounds: Vec<(f32, f32, f32)> = (0..24).map(|_| (next(), next(), 0.03 + 0.07 * next())).collect();
  let heights: Vec<f32> = (0..side * side)
    .map(|index| {
      let (x, y): (f32, f32) = ((index % side) as f32 / side as f32, (index / side) as f32 / side as f32);

      mounds
        .iter()
        .map(|(cx, cy, radius)| {
          let wrapped = |distance: f32| distance.abs().min(1.0 - distance.abs());
          let (dx, dy): (f32, f32) = (wrapped(x - cx), wrapped(y - cy));

          (-(dx * dx + dy * dy) / (radius * radius)).exp()
        })
        .sum()
    })
    .collect();
  let peak: f32 = heights.iter().copied().fold(f32::MIN, f32::max);

  heights.iter().map(|height| height / peak).collect()
}

/// The relief painted as a diffuse: higher is lighter, as a photo lit from above reads.
fn diffuse_of(side: u32, heights: &[f32]) -> RgbaImage {
  RgbaImage::from_fn(side, side, |x, y| {
    let value: u8 = (40.0 + 180.0 * heights[(y * side + x) as usize]) as u8;

    Rgba([value, value, value, u8::MAX])
  })
}

/// The relief as a bump: its normal reversed behind a constant gloss, `x` in alpha and `y` in blue, as the SDK packs it.
fn bump_of(side: u32, heights: &[f32], strength: f32) -> RgbaImage {
  let at = |x: i64, y: i64| {
    heights[(y.rem_euclid(i64::from(side)) * i64::from(side) + x.rem_euclid(i64::from(side))) as usize]
  };
  let byte = |unit: f32| ((unit * 0.5 + 0.5) * 255.0).round() as u8;

  RgbaImage::from_fn(side, side, |x, y| {
    let (x, y): (i64, i64) = (i64::from(x), i64::from(y));
    let (across, down): (f32, f32) = (
      (at(x + 1, y) - at(x - 1, y)) * strength,
      (at(x, y + 1) - at(x, y - 1)) * strength,
    );
    let length: f32 = (across * across + down * down + 1.0).sqrt();

    Rgba([128, byte(1.0 / length), byte(-down / length), byte(-across / length)])
  })
}

#[test]
fn a_bump_drawn_from_its_diffuse_is_aligned() {
  let heights: Vec<f32> = relief(64, 7);
  let alignment: BumpAlignment = BumpAlignment::measure(
    [64, 64],
    &diffuse_of(64, &heights),
    [64, 64],
    &bump_of(64, &heights, 8.0),
  );

  assert_eq!(alignment.verdict, BumpAlignmentVerdict::Aligned, "{alignment:?}");
  assert!(alignment.correlation > alignment.chance, "{alignment:?}");
}

#[test]
fn a_bump_of_another_set_is_mismatched() {
  let alignment: BumpAlignment = BumpAlignment::measure(
    [64, 64],
    &diffuse_of(64, &relief(64, 7)),
    [64, 64],
    &bump_of(64, &relief(64, 91), 8.0),
  );

  assert_eq!(alignment.verdict, BumpAlignmentVerdict::Mismatched, "{alignment:?}");
}

#[test]
fn a_bump_at_a_coarser_resolution_is_still_its_diffuses() {
  // An upscaled diffuse over its original bump, as a high resolution pack leaves the bumps it did not redraw.
  let fine: Vec<f32> = relief(128, 7);
  let coarse: Vec<f32> = relief(64, 7);
  let alignment: BumpAlignment = BumpAlignment::measure(
    [128, 128],
    &diffuse_of(128, &fine),
    [64, 64],
    &bump_of(64, &coarse, 8.0),
  );

  assert_eq!(alignment.verdict, BumpAlignmentVerdict::Aligned, "{alignment:?}");
}

#[test]
fn textures_of_different_proportions_are_compared_over_the_same_coordinates() {
  // Half as tall, and still the same picture: both are sampled across the whole of the surface's coordinates.
  let heights: Vec<f32> = relief(64, 7);
  let squeezed: RgbaImage = imageops::resize(&diffuse_of(64, &heights), 64, 32, FilterType::Triangle);
  let alignment: BumpAlignment = BumpAlignment::measure([64, 32], &squeezed, [64, 64], &bump_of(64, &heights, 8.0));

  assert_eq!(alignment.verdict, BumpAlignmentVerdict::Aligned, "{alignment:?}");
}

#[test]
fn an_atlas_left_black_around_its_islands_is_compared_by_its_islands() {
  // One island of relief in the middle, the rest of the bump unpainted black and the diffuse one flat grey.
  let heights: Vec<f32> = relief(64, 7);
  let inside = |x: u32, y: u32| (16..48).contains(&x) && (16..48).contains(&y);
  let painted: RgbaImage = diffuse_of(64, &heights);
  let relief: RgbaImage = bump_of(64, &heights, 8.0);
  let diffuse: RgbaImage = RgbaImage::from_fn(64, 64, |x, y| {
    if inside(x, y) {
      *painted.get_pixel(x, y)
    } else {
      Rgba([90, 90, 90, 255])
    }
  });
  let bump: RgbaImage = RgbaImage::from_fn(64, 64, |x, y| {
    if inside(x, y) {
      *relief.get_pixel(x, y)
    } else {
      Rgba([0, 0, 0, 0])
    }
  });

  let alignment: BumpAlignment = BumpAlignment::measure([64, 64], &diffuse, [64, 64], &bump);

  assert_eq!(alignment.verdict, BumpAlignmentVerdict::Aligned, "{alignment:?}");
}

#[test]
fn a_flat_bump_has_nothing_to_compare() {
  let heights: Vec<f32> = relief(64, 7);
  let alignment: BumpAlignment = BumpAlignment::measure(
    [64, 64],
    &diffuse_of(64, &heights),
    [64, 64],
    &bump_of(64, &heights, 0.0),
  );

  assert_eq!(alignment.verdict, BumpAlignmentVerdict::Featureless, "{alignment:?}");
}
