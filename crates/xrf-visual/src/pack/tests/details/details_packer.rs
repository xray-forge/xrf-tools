use xrf_level::{DetailModel, DetailVertex, LevelCformFace, LevelCformGeometry, LevelDetailsFile, LevelDetailsHeader};
use xrf_math::Vector3d;

use crate::data::details::details_description::DetailsDescription;
use crate::pack::details::details_package::DetailsPackage;
use crate::pack::details::details_packer::DetailsPacker;

/// The game material the fixture's passable faces are made of.
const PASSABLE: u16 = 9;

/// A slot's sixteen stored bytes, planting object 0 over a base at `base` metres, two metres high.
fn planted(base: f32) -> [u8; 16] {
  let packed_base: u64 = ((base + 200.0) / 0.2).round() as u64;
  let word: u64 = packed_base | (20 << 12) | (0x3F << 26) | (0x3F << 32) | (0x3F << 38) | (5 << 44) | (7 << 48);
  let mut bytes: [u8; 16] = [0; 16];

  bytes[..8].copy_from_slice(&word.to_le_bytes());
  bytes[8..10].copy_from_slice(&0xFFFF_u16.to_le_bytes());

  bytes
}

/// A slot planting nothing.
fn empty() -> [u8; 16] {
  let word: u64 = (0x3F << 20) | (0x3F << 26) | (0x3F << 32) | (0x3F << 38);
  let mut bytes: [u8; 16] = [0; 16];

  bytes[..8].copy_from_slice(&word.to_le_bytes());

  bytes
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
    slots: [planted(0.0), empty(), planted(50.0)].concat(),
  }
}

/// Ground at `y = 1` across world slots -1 and 0, twice over: solid, and a passable copy.
fn collision() -> LevelCformGeometry {
  let face = |vertices: [u32; 3], material: u16| LevelCformFace {
    is_shadow_suppressed: false,
    is_wallmark_suppressed: false,
    material,
    sector: 0,
    vertices,
  };

  LevelCformGeometry {
    faces: vec![face([0, 1, 2], 1), face([0, 1, 2], PASSABLE)],
    vertices: vec![
      Vector3d::new(-2.0, 1.0, -1.0),
      Vector3d::new(2.0, 1.0, 1.0),
      Vector3d::new(-2.0, 1.0, 1.0),
    ],
  }
}

fn pack() -> (DetailsDescription, Vec<u32>) {
  let details: LevelDetailsFile = details();
  let collision: LevelCformGeometry = collision();
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
  assert_eq!(description.triangle_count, 1);
  assert_eq!(description.bin_length, 1);

  let record: Vec<u32> = section(&words, description.slots.byte_offset, description.slots.byte_length);
  let stored: [u8; 16] = planted(0.0);

  assert_eq!(
    record[..4],
    stored
      .as_chunks::<4>()
      .0
      .iter()
      .map(|word| u32::from_le_bytes(*word))
      .collect::<Vec<u32>>()
  );
  // Its bin starts at the first entry and holds one, and it is world slot (-1, 0).
  assert_eq!(record[4..], [0, 1, (-1_i32) as u32, 0]);
}

#[test]
fn keeps_the_triangles_in_the_engine_space_the_planting_is_cast_in() {
  let (description, words) = pack();
  let triangle: Vec<f32> = section(
    &words,
    description.triangles.byte_offset,
    description.triangles.byte_length,
  )
  .into_iter()
  .map(f32::from_bits)
  .collect();

  assert_eq!(triangle, vec![-2.0, 1.0, -1.0, 2.0, 1.0, 1.0, -2.0, 1.0, 1.0]);
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
  assert_eq!(model.index_count, 3);
}
