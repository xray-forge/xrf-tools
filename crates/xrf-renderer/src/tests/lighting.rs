use glam::Vec3;

use crate::contract::render_light_scales::RenderLightScales;
use crate::lighting::light_specular::to_light_specular;
use crate::lighting::material_slice::material_slice;
use crate::lighting::render_lighting::RenderLighting;
use crate::lighting::srgb::{authored_light, srgb_decoded, srgb_encoded};
use crate::lighting::sun_direction::to_renderer_sun_direction;
use crate::pass::material_table::MaterialTable;

fn texel(data: &[u8], slice: u32, y: u32, x: u32) -> [u8; 2] {
  let at: usize = (((slice * MaterialTable::LUT_HEIGHT + y) * MaterialTable::LUT_WIDTH + x) * 2) as usize;

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
  let data: Vec<u8> = MaterialTable::create_lut();

  assert_eq!(
    data.len(),
    (MaterialTable::LUT_WIDTH * MaterialTable::LUT_HEIGHT * MaterialTable::LUT_DEPTH * 2) as usize
  );
  assert_eq!(data.len(), 262_144);
}

#[test]
fn material_lut_matches_the_web_viewer_byte_for_byte() {
  assert_eq!(fnv1a(&MaterialTable::create_lut()), 0xd9aa_6130);
}

#[test]
fn material_lut_samples_every_lighting_model() {
  let data: Vec<u8> = MaterialTable::create_lut();
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
  assert!((lighting.get_sun_specular(&RenderLightScales::default()) - 3.484_544_3).abs() < 1e-6);
  assert_near(lighting.get_ambient(&RenderLightScales::default()), [0.04, 0.04, 0.04]);
  assert_near(
    lighting.get_environment(&RenderLightScales::default()),
    [1.882_372, 1.474_528, 1.317_668],
  );
}

#[test]
fn sun_specular_is_linear_past_one() {
  let lighting: RenderLighting = RenderLighting {
    sun_color: Vec3::new(2.0, 1.5, 1.0),
    ..RenderLighting::default()
  };

  assert_eq!(lighting.get_sun_specular(&RenderLightScales::default()), 6.0);
}

// Where the specular is linear (the physical levels) a light's specular follows it as its diffuse does: a moon of 0.04
// at `r2_sun_lumscale` 2.2 gives a specular of its strength, 0.088, where the engine's curve softens it to 0.2.
#[test]
fn a_linear_specular_follows_its_light() {
  let lighting: RenderLighting = RenderLighting {
    sun_color: Vec3::splat(0.04),
    ..RenderLighting::default()
  };
  let scales: RenderLightScales = RenderLightScales {
    sun: 2.2,
    ..RenderLightScales::default()
  };
  let linear: f32 = to_light_specular(authored_light(lighting.get_sun_color(&scales), true), true);

  assert!((linear - 4.0 * 0.088).abs() < 1e-6, "{linear}");
  assert!(lighting.get_sun_specular(&scales) > 2.0 * linear);
}

// Anomaly's `r2_sun_lumscale 2`: the sun twice as bright, its specular from the scaled colour (linear past one); the
// hemisphere and ambient by their own.
#[test]
fn the_games_console_scales_the_sun_hemisphere_and_ambient() {
  let lighting: RenderLighting = RenderLighting::default();
  let scales: RenderLightScales = RenderLightScales {
    sun: 2.0,
    hemi: 0.5,
    ambient: 3.0,
  };

  assert_near(lighting.get_sun_color(&scales), [1.811_764, 1.678_432, 1.388_236]);
  assert!((lighting.get_sun_specular(&scales) - 6.504_576).abs() < 1e-4);
  assert_near(lighting.get_environment(&scales), [0.941_186, 0.737_264, 0.658_834]);
  assert_near(lighting.get_ambient(&scales), [0.12, 0.12, 0.12]);
}

#[test]
fn ambient_never_falls_below_its_floor() {
  let lighting: RenderLighting = RenderLighting {
    ambient_color: Vec3::ZERO,
    ..RenderLighting::default()
  };

  assert_eq!(lighting.get_ambient(&RenderLightScales::default()), Vec3::splat(0.001));
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

#[test]
fn srgb_encodes_linear_light_as_a_colour_texture_holds_it() {
  // Middle grey, 0.18, is stored as about 117 of 255; the reference display's power holds to black, with no linear toe,
  // so a product of two encoded values decodes to the product of their decodes.
  assert!((srgb_encoded(0.18) - 0.458_659).abs() < 1e-5);
  assert!((srgb_encoded(0.002) - 0.002_f32.powf(1.0 / 2.2)).abs() < 1e-7);
  assert!((srgb_decoded(0.1 * 0.15) - srgb_decoded(0.1) * srgb_decoded(0.15)).abs() < 1e-9);
  assert_eq!(srgb_encoded(0.0), 0.0);
  assert!((srgb_encoded(1.0) - 1.0).abs() < 1e-6);
}

#[test]
fn material_slice_is_what_the_g_buffer_keeps_of_a_texture_material() {
  // Blinn, the default, packed down from 0.375 to 8 of 31 steps over four thirds.
  assert!((material_slice(1.0) - 32.0 / 93.0).abs() < 1e-6);
  assert!((material_slice(0.0) - 8.0 / 93.0).abs() < 1e-6);
  // A tree's `MAT_FLORA` six passes as it stands and wraps in the packing to between Blinn and Phong.
  assert!((material_slice(6.0) - 44.0 / 93.0).abs() < 1e-6);
  // Metal with a heavy weight passes the last slice, which the table's sampler wraps.
  assert!(material_slice(3.8) > 1.0);
}
