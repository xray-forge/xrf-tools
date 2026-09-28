use xrf_error::XrfResult;
use xrf_level::{
  DetailModel, DetailVertex, LevelCformFace, LevelCformGeometry, LevelDetailsFile, LevelDetailsHeader, LevelDetailsSlot,
};
use xrf_math::Vector3d;

use crate::data::details::details_description::DetailsDescription;
use crate::pack::details::details_package::DetailsPackage;
use crate::pack::details::details_packer::DetailsPacker;

/// The game material the fixture's passable faces are made of.
const PASSABLE: u16 = 9;

/// A slot's words, planting object 0 over a base at `base` metres, two metres high.
fn planted(base: f32) -> [u32; LevelDetailsSlot::WORDS] {
  let packed_base: u64 = ((base + 200.0) / 0.2).round() as u64;
  let word: u64 = packed_base | (20 << 12) | (0x3F << 26) | (0x3F << 32) | (0x3F << 38) | (5 << 44) | (7 << 48);

  [word as u32, (word >> 32) as u32, 0xFFFF, 0]
}

/// A slot planting nothing.
fn empty() -> [u32; LevelDetailsSlot::WORDS] {
  let word: u64 = (0x3F << 20) | (0x3F << 26) | (0x3F << 32) | (0x3F << 38);

  [word as u32, (word >> 32) as u32, 0, 0]
}

/// A three by one grid whose cells are world slots -1 to 1 along `x`: the first planted, the second empty, the third
/// planted over ground nothing reaches.
fn details() -> LevelDetailsFile {
  LevelDetailsFile {
    header: LevelDetailsHeader {
      version: 3,
      object_count: 1,
      offset_x: 1,
      offset_z: 0,
      size_x: 3,
      size_z: 1,
    },
    objects: vec![DetailModel {
      shader: String::from("details\\blend"),
      texture: String::from("detail\\grass"),
      flags: 0,
      min_scale: 0.5,
      max_scale: 1.5,
      vertices: vec![
        DetailVertex {
          position: Vector3d::new(-1.0, 0.0, 1.0),
          u: 0.0,
          v: 1.0,
        },
        DetailVertex {
          position: Vector3d::new(1.0, 0.0, 1.0),
          u: 1.0,
          v: 1.0,
        },
        DetailVertex {
          position: Vector3d::new(0.0, 2.0, -1.0),
          u: 0.5,
          v: 0.0,
        },
      ],
      indices: vec![0, 1, 2],
    }],
    slots: vec![planted(0.0), empty(), planted(50.0)],
  }
}

/// One collision face of the given corners and material.
fn face(vertices: [u32; 3], material: u16) -> LevelCformFace {
  LevelCformFace {
    is_shadow_suppressed: false,
    is_wallmark_suppressed: false,
    material,
    sector: 0,
    vertices,
  }
}

/// The corners of the ground at `y = 1` across world slots -1 and 0, wound to face up to a ray cast down.
fn ground() -> Vec<Vector3d> {
  vec![
    Vector3d::new(-2.0, 1.0, -1.0),
    Vector3d::new(-2.0, 1.0, 1.0),
    Vector3d::new(2.0, 1.0, 1.0),
  ]
}

/// That ground twice over: solid, and a passable copy.
fn collision() -> XrfResult<LevelCformGeometry> {
  LevelCformGeometry::new(ground(), vec![face([0, 1, 2], 1), face([0, 1, 2], PASSABLE)])
}

fn pack() -> (DetailsDescription, Vec<u32>) {
  let details: LevelDetailsFile = details();
  let collision: LevelCformGeometry = collision().expect("a valid form");
  let is_passable = |material: u16| material == PASSABLE;
  let package: DetailsPackage = DetailsPacker::new(&details, &collision, &is_passable).pack();
  let words: Vec<u32> = package
    .buffer
    .as_chunks::<4>()
    .0
    .iter()
    .map(|word| u32::from_le_bytes(*word))
    .collect();

  (package.description, words)
}

fn section(words: &[u32], offset: u32, length: u32) -> Vec<u32> {
  words[(offset / 4) as usize..((offset + length) / 4) as usize].to_vec()
}

#[test]
fn bins_each_planted_slot_with_the_solid_ground_under_it() {
  let (description, words) = pack();

  // Only the first cell plants over ground; the empty one and the one standing over nothing are left out.
  assert_eq!(description.slot_count, 1);
  assert_eq!(
    section(&words, description.grid.byte_offset, description.grid.byte_length),
    vec![1, 0, 0]
  );
  assert_eq!(description.get_triangle_count(), 1);
  assert_eq!(description.get_bin_length(), 1);

  let record: Vec<u32> = section(&words, description.slots.byte_offset, description.slots.byte_length);

  assert_eq!(record[..LevelDetailsSlot::WORDS], planted(0.0));
  // Its bin starts at the first entry and holds one.
  assert_eq!(record[4..], [0, 1]);
}

#[test]
fn packs_the_triangles_in_renderer_space_wound_for_it() {
  let (description, words) = pack();
  let triangle: Vec<f32> = section(
    &words,
    description.triangles.byte_offset,
    description.triangles.byte_length,
  )
  .into_iter()
  .map(f32::from_bits)
  .collect();

  // Each corner's z negated, and the last two corners swapped, which keeps the face turned up to a ray cast down.
  assert_eq!(triangle, vec![-2.0, 1.0, 1.0, 2.0, 1.0, -1.0, -2.0, 1.0, -1.0]);
}

#[test]
fn keeps_a_triangle_facing_up_through_the_space_change() {
  let (description, words) = pack();
  let corners: Vec<f32> = section(
    &words,
    description.triangles.byte_offset,
    description.triangles.byte_length,
  )
  .into_iter()
  .map(f32::from_bits)
  .collect();
  let corner = |index: usize| Vector3d::new(corners[index * 3], corners[index * 3 + 1], corners[index * 3 + 2]);
  let (first, second, third) = (corner(0), corner(1), corner(2));
  // `CDB::TestRayTri`'s determinant for a ray straight down, whose sign is which way the face is turned to it.
  let (edge1, edge2) = (
    Vector3d::new(second.x - first.x, second.y - first.y, second.z - first.z),
    Vector3d::new(third.x - first.x, third.y - first.y, third.z - first.z),
  );
  let determinant: f32 = edge1.z * edge2.x - edge1.x * edge2.z;
  let engine: Vec<Vector3d> = ground();
  let (engine_edge1, engine_edge2) = (
    Vector3d::new(engine[1].x - engine[0].x, 0.0, engine[1].z - engine[0].z),
    Vector3d::new(engine[2].x - engine[0].x, 0.0, engine[2].z - engine[0].z),
  );
  let engine_determinant: f32 = engine_edge1.z * engine_edge2.x - engine_edge1.x * engine_edge2.z;

  // Positive is what the culling test keeps, the engine's and the renderer's `toRayRange` alike.
  assert!(engine_determinant > 0.0, "the ground faces up to the engine's ray");
  assert!(determinant > 0.0, "and to the renderer's");
}

#[test]
fn packs_a_model_in_renderer_space_with_the_measures_its_planting_reads() {
  let (description, words) = pack();
  let model = &description.models[0];

  assert!(model.is_waving);
  assert_eq!((model.min_scale, model.max_scale), (0.5, 1.5));
  assert_eq!(model.height, 2.0);
  // From the centre of a box two metres every way to its corner.
  assert!((model.radius - 3.0_f32.sqrt()).abs() < 1e-6);

  let positions: Vec<f32> = section(&words, model.positions.byte_offset, model.positions.byte_length)
    .into_iter()
    .map(f32::from_bits)
    .collect();

  assert_eq!(positions[..3], [-1.0, 0.0, -1.0]);
  // One triangle of sixteen-bit indices.
  assert_eq!(model.indices.byte_length, 3 * 2);
}

#[test]
fn bins_a_triangle_reaching_far_past_the_grid_over_the_part_inside_it_and_skips_one_standing_nowhere() -> XrfResult {
  let details: LevelDetailsFile = details();
  // Ground from the first cell to the edge of what an `f32` holds, and a corner that is no number.
  let collision: LevelCformGeometry = LevelCformGeometry::new(
    vec![
      Vector3d::new(-2.0, 1.0, -1.0),
      Vector3d::new(f32::MAX, 1.0, 1.0),
      Vector3d::new(-2.0, 1.0, 1.0),
      Vector3d::new(f32::NAN, 1.0, 1.0),
    ],
    vec![face([0, 1, 2], 1), face([0, 3, 2], 1)],
  )?;

  let is_passable = |_: u16| false;
  let package: DetailsPackage = DetailsPacker::new(&details, &collision, &is_passable).pack();

  // The wide ground reaches the first cell, the only one planted over it; the unplaced one is binned nowhere.
  assert_eq!(package.description.slot_count, 1);
  assert_eq!(package.description.get_bin_length(), 1);

  Ok(())
}
