#import "common/lighting"
#import "common/srgb"

// How a viewport's scene is shown (`DisplayMode`): as it is, where the engine's passes tonemapped it; by the engine's
// curve over linear light at the engine's display values; or, under the physical light, by a display transform to the
// display's linear colours, then encoded for it. The transforms take scene-referred light, middle grey at 0.18.

const DISPLAY_AS_IS: u32 = 0u;
const DISPLAY_TONEMAPPED: u32 = 1u;
const DISPLAY_ENGINE_ENCODED: u32 = 2u;
const DISPLAY_AGX: u32 = 3u;
const DISPLAY_PBR_NEUTRAL: u32 = 4u;

// Linear sRGB to linear Rec. 2020 and back, the gamut AgX's matrices are fitted in.
const SRGB_TO_REC2020: mat3x3<f32> = mat3x3<f32>(
  vec3<f32>(0.6274, 0.0691, 0.0164), vec3<f32>(0.3293, 0.9195, 0.0880), vec3<f32>(0.0433, 0.0113, 0.8956),
);
const REC2020_TO_SRGB: mat3x3<f32> = mat3x3<f32>(
  vec3<f32>(1.6605, -0.1246, -0.0182), vec3<f32>(-0.5876, 1.1329, -0.1006), vec3<f32>(-0.0728, -0.0083, 1.1187),
);
// AgX's inset, which pulls the primaries in so a saturated colour desaturates towards white as it brightens, and its
// outset, which restores what the inset took below the shoulder.
const AGX_INSET: mat3x3<f32> = mat3x3<f32>(
  vec3<f32>(0.856627153315983, 0.137318972929847, 0.11189821299995),
  vec3<f32>(0.0951212405381588, 0.761241990602591, 0.0767994186031903),
  vec3<f32>(0.0482516061458583, 0.101439036467562, 0.811302368396859),
);
const AGX_OUTSET: mat3x3<f32> = mat3x3<f32>(
  vec3<f32>(1.1271005818144368, -0.1413297634984383, -0.14132976349843826),
  vec3<f32>(-0.11060664309660323, 1.157823702216272, -0.11060664309660294),
  vec3<f32>(-0.016493938717834573, -0.016493938717834257, 1.2519364065950405),
);
// The stops AgX's log encoding spans, about middle grey.
const AGX_MIN_EV: f32 = -12.47393;
const AGX_MAX_EV: f32 = 4.026069;
// PBR Neutral's shoulder start (0.8 less its toe's 0.04) and how far the compressed part desaturates.
const NEUTRAL_START: f32 = 0.76;
const NEUTRAL_DESATURATION: f32 = 0.15;

// AgX's sigmoid, as a polynomial fit over the log-encoded unit range.
fn agx_contrast(x: vec3<f32>) -> vec3<f32> {
  let x2: vec3<f32> = x * x;
  let x4: vec3<f32> = x2 * x2;

  return 15.5 * x4 * x2 - 40.14 * x4 * x + 31.96 * x4 - 6.868 * x2 * x + 0.4298 * x2 + 0.1191 * x - 0.00232;
}

// AgX's base look: scene light to the display's linear colours.
fn agx(light: vec3<f32>) -> vec3<f32> {
  let inset: vec3<f32> = AGX_INSET * (SRGB_TO_REC2020 * light);
  let encoded: vec3<f32> = saturate((log2(max(inset, vec3<f32>(1e-10))) - AGX_MIN_EV) / (AGX_MAX_EV - AGX_MIN_EV));
  let shown: vec3<f32> = pow(max(AGX_OUTSET * agx_contrast(encoded), vec3<f32>(0.0)), vec3<f32>(2.2));

  return saturate(REC2020_TO_SRGB * shown);
}

// Khronos PBR Neutral: scene light to the display's linear colours, as they are below the shoulder.
fn pbr_neutral(light: vec3<f32>) -> vec3<f32> {
  let least: f32 = min(light.r, min(light.g, light.b));
  let offset: f32 = select(0.04, least - 6.25 * least * least, least < 0.08);
  let toed: vec3<f32> = light - offset;
  let peak: f32 = max(toed.r, max(toed.g, toed.b));

  if (peak < NEUTRAL_START) {
    return saturate(toed);
  }

  let reach: f32 = 1.0 - NEUTRAL_START;
  let compressed: f32 = 1.0 - reach * reach / (peak + reach - NEUTRAL_START);
  let scaled: vec3<f32> = toed * (compressed / peak);
  let grey: f32 = 1.0 - 1.0 / (NEUTRAL_DESATURATION * (peak - compressed) + 1.0);

  return saturate(mix(scaled, vec3<f32>(compressed), grey));
}

// A colour of the scene as the display shows it under `mode`.
fn shown_for_display(colour: vec3<f32>, mode: u32) -> vec3<f32> {
  switch (mode) {
    case DISPLAY_TONEMAPPED: {
      return tonemap(colour, 1.0);
    }
    case DISPLAY_ENGINE_ENCODED: {
      return srgb_encoded(tonemap(colour, 1.0));
    }
    case DISPLAY_AGX: {
      return srgb_encoded(agx(colour));
    }
    case DISPLAY_PBR_NEUTRAL: {
      return srgb_encoded(pbr_neutral(colour));
    }
    default: {
      return colour;
    }
  }
}
