//! The display transforms as their published definitions write them, on the CPU: Khronos PBR Neutral's reference,
//! three.js's AgX and the engine's curve in the engine's own domain, colours of linear light to the display's linear
//! colours. `common/display_transform.wgsl` runs them on the GPU; `DisplayMode::get_anchor_stops` solves its anchors
//! from them, and the transforms' GPU test holds the shader to them.

use glam::{Mat3, Vec3};

use crate::lighting::srgb::{srgb_decoded, srgb_encoded};

/// The engine's curve's white, as `tonemap` squares it.
const WHITE: f32 = 1.7;
/// The luminance the grading's contrast turns about, as `graded` turns it.
const MIDDLE_GREY: f32 = 0.18;

/// The engine's curve a channel, `x (1 + x / W²) / (1 + x)`.
fn engine_curve(light: Vec3) -> Vec3 {
  light * (Vec3::ONE + light / (WHITE * WHITE)) / (Vec3::ONE + light)
}

/// The engine's curve in the engine's domain: over the light encoded for the display, its output the display value,
/// given back as the display's linear colour.
pub fn engine_display(light: Vec3) -> Vec3 {
  let encoded: Vec3 = light.to_array().map(srgb_encoded).into();

  engine_curve(encoded).to_array().map(srgb_decoded).into()
}

/// Khronos PBR Neutral, as the reference GLSL of the Khronos PBR Neutral specification writes it.
pub fn pbr_neutral(colour: Vec3) -> Vec3 {
  let start: f32 = 0.8 - 0.04;
  let desaturation: f32 = 0.15;
  let least: f32 = colour.min_element();
  let offset: f32 = if least < 0.08 {
    least - 6.25 * least * least
  } else {
    0.04
  };
  let toed: Vec3 = colour - Vec3::splat(offset);
  let peak: f32 = toed.max_element();

  if peak < start {
    return toed.clamp(Vec3::ZERO, Vec3::ONE);
  }

  let d: f32 = 1.0 - start;
  let compressed: f32 = 1.0 - d * d / (peak + d - start);
  let grey: f32 = 1.0 - 1.0 / (desaturation * (peak - compressed) + 1.0);

  (toed * (compressed / peak))
    .lerp(Vec3::splat(compressed), grey)
    .clamp(Vec3::ZERO, Vec3::ONE)
}

/// AgX's base look as three.js's `AgXToneMapping` writes it: Rec. 2020, the inset, a log encoding over −12.47 to +4.03
/// stops, the polynomial sigmoid, the outset, the 2.2 power, back to sRGB.
pub fn agx(colour: Vec3) -> Vec3 {
  let to_rec2020: Mat3 = Mat3::from_cols(
    Vec3::new(0.6274, 0.0691, 0.0164),
    Vec3::new(0.3293, 0.9195, 0.0880),
    Vec3::new(0.0433, 0.0113, 0.8956),
  );
  let from_rec2020: Mat3 = Mat3::from_cols(
    Vec3::new(1.6605, -0.1246, -0.0182),
    Vec3::new(-0.5876, 1.1329, -0.1006),
    Vec3::new(-0.0728, -0.0083, 1.1187),
  );
  let inset: Mat3 = Mat3::from_cols(
    Vec3::new(0.85662715, 0.13731897, 0.11189821),
    Vec3::new(0.09512124, 0.761242, 0.07679942),
    Vec3::new(0.048251606, 0.10143904, 0.81130237),
  );
  let outset: Mat3 = Mat3::from_cols(
    Vec3::new(1.1271006, -0.14132976, -0.14132976),
    Vec3::new(-0.11060664, 1.1578237, -0.11060664),
    Vec3::new(-0.016493939, -0.016493939, 1.2519364),
  );
  let (least, most): (f32, f32) = (-12.47393, 4.026069);
  let encoded: Vec3 = (inset * (to_rec2020 * colour))
    .max(Vec3::splat(1e-10))
    .to_array()
    .map(|channel| ((channel.log2() - least) / (most - least)).clamp(0.0, 1.0))
    .into();
  let sigmoid = |x: f32| {
    let x2: f32 = x * x;
    let x4: f32 = x2 * x2;

    15.5 * x4 * x2 - 40.14 * x4 * x + 31.96 * x4 - 6.868 * x2 * x + 0.4298 * x2 + 0.1191 * x - 0.00232
  };
  let curved: Vec3 = encoded.to_array().map(sigmoid).into();
  let shown: Vec3 = (outset * curved).max(Vec3::ZERO).powf(2.2);

  (from_rec2020 * shown).clamp(Vec3::ZERO, Vec3::ONE)
}

/// A luminance as the grading's `contrast` in stops about middle grey shows it, as `graded` does.
pub fn get_graded(luminance: f32, contrast: f32) -> f32 {
  MIDDLE_GREY * (luminance / MIDDLE_GREY).powf(contrast)
}

/// The stops a colour of `hue` at luminance `key` is lifted by for `transform` to show it as bright as the engine's curve
/// shows it, held to `bounds`: the transform's response solved by bisection, as it rises with the light.
pub fn get_key_match(transform: fn(Vec3) -> Vec3, (key, hue): (f32, Vec3), (least, most): (f32, f32)) -> f32 {
  let luminance = |colour: Vec3| colour.dot(Vec3::new(0.2126, 0.7152, 0.0722));
  let colour: Vec3 = hue * (key / luminance(hue).max(1e-6));
  let wanted: f32 = luminance(engine_display(colour));
  let (mut low, mut high): (f32, f32) = (least, most);

  for _ in 0..40 {
    let middle: f32 = 0.5 * (low + high);

    if luminance(transform(colour * middle.exp2())) < wanted {
      low = middle;
    } else {
      high = middle;
    }
  }

  0.5 * (low + high)
}
