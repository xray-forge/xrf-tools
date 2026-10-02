use glam::Vec3;

use crate::lighting::material_lut::{MATERIAL_LUT_DEPTH, MATERIAL_LUT_HEIGHT, MATERIAL_LUT_WIDTH, create_material_lut};
use crate::lighting::render_lighting::RenderLighting;
use crate::lighting::sun_direction::to_renderer_sun_direction;

fn texel(data: &[u8], slice: u32, y: u32, x: u32) -> [u8; 2] {
  let at: usize = (((slice * MATERIAL_LUT_HEIGHT + y) * MATERIAL_LUT_WIDTH + x) * 2) as usize;

  [data[at], data[at + 1]]
}

fn fnv1a(data: &[u8]) -> u32 {
  data.iter().fold(0x811c_9dc5, |hash: u32, byte: &u8| {
    (hash ^ u32::from(*byte)).wrapping_mul(0x0100_0193)
  })
}

fn assert_near(actual: Vec3, expected: [f32; 3]) {
  assert!(
    actual.distance(Vec3::from_array(expected)) < 1e-6,
    "{actual:?} is not {expected:?}"
  );
}

#[test]
fn material_lut_holds_two_channels_a_texel() {
  let data: Vec<u8> = create_material_lut();

  assert_eq!(
    data.len(),
    (MATERIAL_LUT_WIDTH * MATERIAL_LUT_HEIGHT * MATERIAL_LUT_DEPTH * 2) as usize
  );
  assert_eq!(data.len(), 262_144);
}

#[test]
fn material_lut_matches_the_web_viewer_byte_for_byte() {
  assert_eq!(fnv1a(&create_material_lut()), 0xd9aa_6130);
}

#[test]
fn material_lut_samples_every_lighting_model() {
  let data: Vec<u8> = create_material_lut();
  let expected: [(u32, u32, u32, [u8; 2]); 19] = [
    (0, 0, 0, [0, 0]),
    (0, 128, 64, [152, 0]),
    (0, 255, 127, [255, 255]),
    (0, 255, 126, [253, 127]),
    (0, 200, 100, [213, 2]),
    (1, 0, 127, [255, 0]),
    (1, 240, 127, [255, 59]),
    (1, 100, 30, [69, 0]),
    (1, 254, 127, [255, 232]),
    (2, 0, 0, [0, 0]),
    (2, 250, 120, [241, 57]),
    (2, 255, 127, [255, 255]),
    (2, 253, 127, [255, 255]),
    (2, 64, 64, [128, 0]),
    (3, 0, 127, [255, 0]),
    (3, 128, 64, [128, 215]),
    (3, 255, 127, [255, 255]),
    (3, 17, 3, [6, 147]),
    (3, 254, 126, [253, 233]),
  ];

  for (slice, y, x, value) in expected {
    assert_eq!(texel(&data, slice, y, x), value, "slice {slice}, row {y}, column {x}");
  }
}

#[test]
fn default_lighting_is_noon_of_default_clear() {
  let lighting: RenderLighting = RenderLighting::default();

  assert_near(lighting.sun_direction, [0.808_504_3, -0.5, -0.310_356]);
  assert_near(lighting.get_sun_direction(), [0.808_504_3, -0.5, -0.310_356]);
  assert_near(lighting.sun_color, [0.905_882, 0.839_216, 0.694_118]);
  assert_near(lighting.sky_irradiance, [0.5, 0.511, 0.548]);
  assert!((lighting.get_sun_specular() - 3.484_544_3).abs() < 1e-6);
  assert_near(lighting.get_ambient(), [0.04, 0.04, 0.04]);
  assert_near(lighting.get_environment(), [1.882_372, 1.474_528, 1.317_668]);
}

#[test]
fn sun_specular_is_linear_past_one() {
  let lighting: RenderLighting = RenderLighting {
    sun_color: Vec3::new(2.0, 1.5, 1.0),
    ..RenderLighting::default()
  };

  assert_eq!(lighting.get_sun_specular(), 6.0);
}

#[test]
fn ambient_never_falls_below_its_floor() {
  let lighting: RenderLighting = RenderLighting {
    ambient_color: Vec3::ZERO,
    ..RenderLighting::default()
  };

  assert_eq!(lighting.get_ambient(), Vec3::splat(0.001));
}

#[test]
fn sun_direction_normalises_and_leaves_zero_alone() {
  let long: RenderLighting = RenderLighting {
    sun_direction: Vec3::new(0.0, -4.0, 0.0),
    ..RenderLighting::default()
  };
  let zero: RenderLighting = RenderLighting {
    sun_direction: Vec3::ZERO,
    ..RenderLighting::default()
  };

  assert_eq!(long.get_sun_direction(), Vec3::NEG_Y);
  assert_eq!(zero.get_sun_direction(), Vec3::ZERO);
}

#[test]
fn sun_direction_takes_heading_then_pitch_with_z_negated() {
  assert_near(to_renderer_sun_direction(0.0, 0.0), [0.0, 0.0, -1.0]);
  assert_near(to_renderer_sun_direction(90.0, 0.0), [-1.0, 0.0, 0.0]);
  assert_near(to_renderer_sun_direction(0.0, -90.0), [0.0, -1.0, 0.0]);
  assert_near(to_renderer_sun_direction(45.0, -45.0), [-0.5, -0.707_106_77, -0.5]);
}
