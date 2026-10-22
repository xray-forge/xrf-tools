use crate::scene::level::grass_build_size::GrassBuildSize;
use crate::scene::level::grass_view::GrassView;

/// wgpu's default storage binding: 128 MiB.
const BINDING: u64 = 128 << 20;

#[test]
fn keeps_a_lens_spacing_and_a_small_flat_ring_spacing() {
  assert_eq!(GrassView::get_density(0.3, (220, false), BINDING), 0.3);
  assert_eq!(GrassView::get_density(0.3, (20, true), BINDING), 0.3);
}

#[test]
fn widens_a_flat_ring_until_every_slot_fits_the_lists() {
  let reach: i32 = 220;
  let density: f32 = GrassView::get_density(0.3, (reach, true), BINDING);
  let line: u64 = reach as u64 * 2 + 1;
  let steps: u64 = (2.0 / density).ceil() as u64;

  assert!(density > 0.3 && density <= 0.99, "{density}");
  assert!(line * line * (steps + 1) * (steps + 1) * GrassBuildSize::ITEM_BYTES <= BINDING);
}
