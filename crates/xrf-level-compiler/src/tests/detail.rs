//! How the detail lighter meets and lights each slot of a small grid.

use glam::Vec3;
use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_geometry::BvhTracer;
use xrf_job::JobHandle;
use xrf_level::{
  LevelDetailsFile, LevelDetailsHeader, LevelDetailsLight, LevelDetailsSlot, LevelLight, LevelLightRole,
  LevelLightsChunk, LevelLightsFile,
};
use xrf_math::Vector3d;

use crate::shadow::shadow_materials::ShadowMaterials;
use crate::{DetailLightRequest, DetailLighter, DetailLighting, DetailSlotCounts};

/// A row of five slots, 2 m each from the origin along x, each 1 m tall from its base at 0 m:
///
/// - over ground at 0.5 m, open to the sky;
/// - over the same ground, under a roof at 3 m;
/// - over nothing;
/// - planting an object at no density;
/// - planting nothing.
fn details() -> LevelDetailsFile {
  // Base 1000 units of 0.2 m from -200 m, height 10 units of 0.1 m, no corner planting.
  let empty: [u32; 4] = (0..LevelDetailsSlot::CORNERS).fold([1000 | (10 << 12), 0, 0, 0], |words, corner| {
    LevelDetailsSlot::with_object(words, corner, LevelDetailsSlot::EMPTY_ID)
  });
  let planted: [u32; 4] = LevelDetailsSlot::with_object(empty, 0, 0);
  let dense: [u32; 4] = [planted[0], planted[1], 0x0000_0011, 0];

  LevelDetailsFile {
    header: LevelDetailsHeader {
      version: LevelDetailsFile::CURRENT_VERSION,
      object_count: 0,
      offset_x: 0,
      offset_z: 0,
      size_x: 5,
      size_z: 1,
    },
    objects: Vec::new(),
    slots: vec![dense, dense, dense, planted, empty],
  }
}

/// A quad from `low` to `high` across x and z at height `y`, facing up or down.
fn quad(low: [f32; 2], high: [f32; 2], y: f32, is_up: bool) -> [[Vec3; 3]; 2] {
  let corner = |x: f32, z: f32| Vec3::new(x, y, z);
  let [a, b, c, d] = [
    corner(low[0], low[1]),
    corner(low[0], high[1]),
    corner(high[0], low[1]),
    corner(high[0], high[1]),
  ];

  if is_up {
    [[a, b, c], [c, b, d]]
  } else {
    [[a, c, b], [c, d, b]]
  }
}

/// Ground under the first two slots, ending short of the third, and a roof over the second.
fn tracer() -> BvhTracer {
  let triangles: Vec<[Vec3; 3]> = [
    quad([-1.0, -1.0], [3.9, 3.0], 0.5, true),
    quad([2.0, -1.0], [4.0, 3.0], 3.0, false),
  ]
  .into_iter()
  .flatten()
  .collect();
  let vertices: Vec<Vec3> = triangles.iter().flatten().copied().collect();
  let faces: Vec<[u32; 3]> = (0..triangles.len() as u32)
    .map(|face| [face * 3, face * 3 + 1, face * 3 + 2])
    .collect();

  BvhTracer::new(&vertices, &faces)
}

fn light(kind: u16, position: [f32; 3], direction: [f32; 3], diffuse: [f32; 3]) -> LevelLight {
  let vector = |[x, y, z]: [f32; 3]| Vector3d::new(x, y, z);

  LevelLight {
    kind,
    level: 0,
    diffuse: vector(diffuse),
    position: vector(position),
    direction: vector(direction),
    range: 2.5,
    range_squared: 6.25,
    falloff: 0.0,
    attenuation_constant: 1.0,
    attenuation_linear: 0.0,
    attenuation_quadratic: 0.0,
    energy: 1.0,
    triangle: [Vector3d::new(0.0, 0.0, 0.0); 3],
  }
}

/// The sun straight overhead, and a bright red point light beside the first slot, reaching a corner of its points and
/// none of the second's.
fn lights() -> LevelLightsFile {
  LevelLightsFile {
    chunks: vec![
      LevelLightsChunk::Lights {
        id: LevelLightRole::Rgb.get_chunk_id(),
        lights: vec![light(
          LevelLight::KIND_POINT,
          [-1.0, 2.5, 1.0],
          [0.0; 3],
          [20.0, 0.0, 0.0],
        )],
      },
      LevelLightsChunk::Lights {
        id: LevelLightRole::Hemi.get_chunk_id(),
        lights: Vec::new(),
      },
      LevelLightsChunk::Lights {
        id: LevelLightRole::Sun.get_chunk_id(),
        lights: vec![light(
          LevelLight::KIND_DIRECTIONAL,
          [0.0; 3],
          [0.0, -1.0, 0.0],
          [1.0; 3],
        )],
      },
    ],
  }
}

fn lighting() -> XrfResult<DetailLighting> {
  let details: LevelDetailsFile = details();
  let tracer: BvhTracer = tracer();

  DetailLighter::light(
    &DetailLightRequest {
      details: &details,
      tracer: &tracer,
      shadows: &ShadowMaterials::new_opaque(4),
      lights: &lights(),
    },
    &JobHandle::inert(),
  )
}

#[test]
fn meets_each_slot_as_the_sdk_s_lighter_does() -> XrfResult {
  let lighting: DetailLighting = lighting()?;
  let original: LevelDetailsFile = details();

  assert_eq!(
    lighting.counts,
    DetailSlotCounts {
      unplanted: 1,
      emptied: 1,
      untouched: 1,
      lit: 2,
    }
  );
  // The bare corner is cleared; the slots the lighter leaves keep their words and their zero light.
  assert!(!LevelDetailsSlot::of(&lighting.details.slots[3]).is_planted());
  assert_eq!(lighting.details.slots[2], original.slots[2]);
  assert_eq!(lighting.details.slots[4], original.slots[4]);

  Ok(())
}

#[test]
fn lights_a_slot_by_what_reaches_its_points() -> XrfResult {
  let lighting: DetailLighting = lighting()?;
  let open: LevelDetailsLight = LevelDetailsSlot::get_light(&lighting.details.slots[0]);
  let roofed: LevelDetailsLight = LevelDetailsSlot::get_light(&lighting.details.slots[1]);

  // The open slot's every point sees the whole sun, which the lighter halves: 0.5, seven steps of fifteen.
  assert_eq!(open.sun, 7);
  assert_eq!(open.hemi, 0);
  assert!(open.red > 0 && open.green == 0 && open.blue == 0, "{open:?}");
  // The roof blocks the sun, and the point light reaches none of the roofed slot's points.
  assert_eq!(roofed, LevelDetailsLight::UNLIT);

  Ok(())
}

#[test]
fn writes_the_lit_details_back_in_the_layout_the_engine_reads() -> XrfResult {
  let lighting: DetailLighting = lighting()?;
  let mut writer: ChunkWriter = ChunkWriter::new();

  lighting.details.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(
    LevelDetailsFile::read_from_bytes::<XRayByteOrder>(writer.buffer)?.slots,
    lighting.details.slots
  );

  Ok(())
}
