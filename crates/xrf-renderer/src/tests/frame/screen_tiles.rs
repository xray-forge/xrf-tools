use glam::{Mat4, Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::contract::viewport::render_rect::RenderRect;
use crate::frame::targets::screen_tiles::ScreenTiles;

const SIZE: (u32, u32) = (3730, 1930);

/// A camera at the origin looking down `-z`, as wide as the frame.
fn new_view_projection() -> Mat4 {
  let view: Mat4 = glam::camera::rh::view::look_at_mat4(Vec3::ZERO, Vec3::NEG_Z, Vec3::Y);

  CameraView::new(Vec3::ZERO, view, 80.0, SIZE.0 as f32 / SIZE.1 as f32, 0.2, 1000.0).get_view_projection()
}

/// A square quad `half` across either way of `center`, facing the camera.
fn new_quad(center: Vec3, half: f32) -> [Vec3; 4] {
  [
    center + Vec3::new(-half, -half, 0.0),
    center + Vec3::new(half, -half, 0.0),
    center + Vec3::new(-half, half, 0.0),
    center + Vec3::new(half, half, 0.0),
  ]
}

fn cover_quad(quad: &[Vec3]) -> ScreenTiles {
  let mut tiles: ScreenTiles = ScreenTiles::default();

  tiles.cover_hull(new_view_projection(), quad);

  tiles
}

#[test]
fn marks_nothing_until_a_writer_covers_a_tile() {
  assert!(ScreenTiles::default().is_empty());
  assert!(ScreenTiles::default().get_rects(SIZE).is_empty());
}

#[test]
fn lays_every_tile_over_the_frame_as_one_rect() {
  assert_eq!(
    ScreenTiles::new_whole().get_rects(SIZE),
    vec![RenderRect {
      x: 0,
      y: 0,
      width: SIZE.0,
      height: SIZE.1,
    }]
  );
}

#[test]
fn bounds_a_small_quad_ahead_to_the_tiles_about_the_frames_centre() {
  let rects: Vec<RenderRect> = cover_quad(&new_quad(Vec3::new(0.0, 0.0, -10.0), 0.1)).get_rects(SIZE);

  assert_eq!(rects.len(), 1);

  let rect: RenderRect = rects[0];
  let (center_x, center_y) = (SIZE.0.cast_signed() / 2, SIZE.1.cast_signed() / 2);

  assert!(rect.x < center_x && rect.x + rect.width.cast_signed() > center_x);
  assert!(rect.y < center_y && rect.y + rect.height.cast_signed() > center_y);
  assert!(rect.width * rect.height < SIZE.0 * SIZE.1 / 50);
}

#[test]
fn marks_nothing_for_a_quad_behind_the_eye_or_off_the_screen() {
  assert!(cover_quad(&new_quad(Vec3::new(0.0, 0.0, 10.0), 1.0)).is_empty());
  assert!(cover_quad(&new_quad(Vec3::new(100.0, 0.0, -10.0), 1.0)).is_empty());
}

#[test]
fn bounds_a_quad_across_the_eye_by_its_part_before_it() {
  // Lying on the ground from behind the eye to ahead of it, a little to the right: the right half's lower part.
  let quad: [Vec3; 4] = [
    Vec3::new(0.5, -1.0, 5.0),
    Vec3::new(1.5, -1.0, 5.0),
    Vec3::new(0.5, -1.0, -5.0),
    Vec3::new(1.5, -1.0, -5.0),
  ];
  let rects: Vec<RenderRect> = cover_quad(&quad).get_rects(SIZE);

  assert!(!rects.is_empty());
  assert!(
    rects
      .iter()
      .all(|rect| rect.x >= SIZE.0.cast_signed() / 2 && rect.y >= SIZE.1.cast_signed() / 2)
  );
}

#[test]
fn takes_a_sphere_about_the_eye_as_the_whole_frame() {
  let mut tiles: ScreenTiles = ScreenTiles::default();

  tiles.cover_sphere(new_view_projection(), Vec4::new(0.0, 0.0, -1.0, 3.0));

  assert_eq!(tiles, ScreenTiles::new_whole());
}

#[test]
fn grows_a_tile_into_its_neighbours_within_the_frame() {
  let tile: ScreenTiles = cover_quad(&new_quad(Vec3::new(0.0, 0.0, -10.0), 0.01));
  let rect: RenderRect = tile.get_rects(SIZE)[0];
  let grown: RenderRect = tile.grown(1, 1).get_rects(SIZE)[0];
  let (tile_width, tile_height) = (SIZE.0 / ScreenTiles::COLUMNS, SIZE.1 / ScreenTiles::ROWS);

  assert!(grown.x <= rect.x - tile_width.cast_signed() && grown.y <= rect.y - tile_height.cast_signed());
  assert!(grown.width >= rect.width + 2 * tile_width && grown.height >= rect.height + 2 * tile_height);
  assert_eq!(ScreenTiles::new_whole().grown(3, 3), ScreenTiles::new_whole());
}

#[test]
fn keeps_apart_quads_apart_as_rects_of_their_own() {
  let mut tiles: ScreenTiles = cover_quad(&new_quad(Vec3::new(-6.0, 3.0, -10.0), 0.05));

  tiles.cover_tiles(&cover_quad(&new_quad(Vec3::new(6.0, -3.0, -10.0), 0.05)));

  let rects: Vec<RenderRect> = tiles.get_rects(SIZE);
  let covered: u32 = rects.iter().map(|rect| rect.width * rect.height).sum();

  assert_eq!(rects.len(), 2);
  assert!(covered < SIZE.0 * SIZE.1 / 50);
}
