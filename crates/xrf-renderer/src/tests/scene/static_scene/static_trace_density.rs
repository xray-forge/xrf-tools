//! A cut-out triangle's density word, which the scene trace tests its holes by.

use glam::{Vec2, Vec3};

use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;
use crate::scene::static_scene::item::static_scene_writer::StaticSceneWriter;
use crate::scene::static_scene::trace::static_trace_density::StaticTraceDensity;
use crate::scene::static_scene::trace::static_trace_geometry::StaticTraceGeometry;

/// A triangle's vertices in a layout's words: its positions, and each base coordinate's words as the layout packs it.
fn create_words(positions: [Vec3; 3], uvs: [[u32; 4]; 3]) -> Vec<u32> {
  let mut words: Vec<u32> = vec![0; StaticLayout::STRIDE as usize * 3];

  for corner in 0..3 {
    let vertex: &mut [u32] = &mut words[corner * StaticLayout::STRIDE as usize..][..StaticLayout::STRIDE as usize];
    let [uv, extra, tangent, binormal]: [u32; 4] = uvs[corner];

    for axis in 0..3 {
      vertex[StaticLayout::POSITION + axis] = positions[corner][axis].to_bits();
    }

    vertex[StaticLayout::UV] = uv;
    vertex[StaticLayout::EXTRA] = extra;
    vertex[StaticLayout::TANGENT] = tangent;
    vertex[StaticLayout::BINORMAL] = binormal;
  }

  words
}

/// Two signed shorts in a word, the first low, as the arenas pack a base coordinate.
fn pack_shorts(x: i16, y: i16) -> u32 {
  u32::from(x.cast_unsigned()) | (u32::from(y.cast_unsigned()) << 16)
}

/// A word's logarithm and direction, as `cut_out_lod` in `shaders/common/scene_trace.wgsl` reads them.
fn read(word: u32) -> (f32, Vec2) {
  let byte = |shift: u32| f32::from((word >> shift) as u8) / 255.0 * 2.0 - 1.0;

  (
    (word & 0xffff) as f32 / StaticTraceDensity::STEPS - StaticTraceDensity::BIAS,
    Vec2::new(byte(16), byte(24)),
  )
}

/// A spawned model's triangle two metres by one, its base coordinate spanning a unit by a half: a quarter of a unit of
/// area a metre of its own, half a unit a metre each way, and the direction of its corners' cross straight down `y`.
#[test]
fn measures_a_model_triangle_s_density_and_its_cross() {
  let uv = |x: f32, y: f32| [x.to_bits(), y.to_bits(), 0, 0];
  let words: Vec<u32> = create_words(
    [Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)],
    [uv(0.0, 0.0), uv(1.0, 0.0), uv(0.0, 0.5)],
  );
  let measured: Vec<u32> = StaticTraceDensity::measure(StaticLayout::Model, &words, 0, &[0, 1, 2]);
  let (logarithm, across): (f32, Vec2) = read(measured[0]);

  assert_eq!(measured.len(), 1);
  assert!((logarithm + 1.0).abs() < 1.0 / 512.0, "{logarithm}");
  // Within a byte's step over -1..1, which nought falls between.
  assert!(
    (across - Vec2::new(0.0, -1.0)).abs().max_element() <= 2.0 / 255.0,
    "{across}"
  );
}

/// A sector's base coordinate is read with its fraction from its tangent's and binormal's fourth bytes, over 1024, and
/// a tree's over 2048, as the trace reads them.
#[test]
fn reads_a_sector_s_and_a_tree_s_base_coordinates_as_the_trace_does() {
  let positions: [Vec3; 3] = [Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)];
  let half: u32 = 128 << 24;
  // Corners at (0, 0), (2.5, 0) and (0, 2.5) units of 1024, each half from its fourth bytes.
  let baked: Vec<u32> = create_words(
    positions,
    [
      [pack_shorts(0, 0), 0, 0, 0],
      [pack_shorts(2, 0), 0, half, 0],
      [pack_shorts(0, 2), 0, 0, half],
    ],
  );
  let tree: Vec<u32> = create_words(
    positions,
    [
      [pack_shorts(0, 0), 0, 0, 0],
      [pack_shorts(1024, 0), 0, 0, 0],
      [pack_shorts(0, -1024), 0, 0, 0],
    ],
  );
  let expected = |side: f32| 0.5 * (side * side).log2();
  let (sector, _): (f32, Vec2) = read(StaticTraceDensity::measure(StaticLayout::Baked, &baked, 0, &[0, 1, 2])[0]);
  let (leaf, _): (f32, Vec2) = read(StaticTraceDensity::measure(StaticLayout::Tree, &tree, 0, &[0, 1, 2])[0]);
  let fraction: f32 = 128.0 / 255.0;

  assert!(
    (sector - expected((2.0 + fraction) / 1024.0)).abs() < 1.0 / 512.0,
    "{sector}"
  );
  assert!((leaf - expected(0.5)).abs() < 1.0 / 512.0, "{leaf}");
}

/// A triangle of no area is stored densest, read at its coarsest, and one whose base coordinates span none sparsest,
/// read at its finest, as the trace reads them from its corners.
#[test]
fn stores_a_triangle_of_no_area_densest_and_one_of_no_coordinates_sparsest() {
  let uv = |x: f32, y: f32| [x.to_bits(), y.to_bits(), 0, 0];
  let flat: Vec<u32> = create_words(
    [Vec3::ZERO, Vec3::X, Vec3::new(2.0, 0.0, 0.0)],
    [uv(0.0, 0.0), uv(1.0, 0.0), uv(0.0, 1.0)],
  );
  let bare: Vec<u32> = create_words(
    [Vec3::ZERO, Vec3::X, Vec3::Y],
    [uv(0.0, 0.0), uv(0.0, 0.0), uv(0.0, 0.0)],
  );

  assert_eq!(
    StaticTraceDensity::measure(StaticLayout::Model, &flat, 0, &[0, 1, 2])[0] & 0xffff,
    0xffff
  );
  assert_eq!(
    StaticTraceDensity::measure(StaticLayout::Model, &bare, 0, &[0, 1, 2])[0] & 0xffff,
    0
  );
}

/// A writer measures a cut-out geometry's triangles after the words it put, starting it where they start, and none of
/// an opaque one's.
#[test]
fn puts_the_density_words_of_cut_out_geometry_alone() {
  let uv = |x: f32, y: f32| [x.to_bits(), y.to_bits(), 0, 0];
  let mut writer: StaticSceneWriter = StaticSceneWriter::default();
  let words: Vec<u32> = create_words(
    [Vec3::ZERO, Vec3::X, Vec3::Y],
    [uv(0.0, 0.0), uv(1.0, 0.0), uv(0.0, 1.0)],
  );

  writer.put_words(StaticLayout::Model, &words, &[0, 1, 2, 0, 2, 1]);

  let geometry = |is_cut_out: bool, first_index: u32| StaticTraceGeometry {
    layout: StaticLayout::Model,
    first_index,
    index_count: 3,
    vertex_start: 0,
    vertex_count: 3,
    surface: 0,
    is_cut_out,
    first_density: 7,
  };
  let opaque: StaticTraceGeometry = writer.put_densities(geometry(false, 0));
  let first: StaticTraceGeometry = writer.put_densities(geometry(true, 0));
  let second: StaticTraceGeometry = writer.put_densities(geometry(true, 3));

  assert_eq!(
    (opaque.first_density, first.first_density, second.first_density),
    (0, 0, 1)
  );
  assert_eq!(writer.trace_densities.len(), 2);
  assert_eq!(
    writer.trace_densities[0] & 0xffff,
    writer.trace_densities[1] & 0xffff,
    "a triangle turned over spans as much"
  );
}
