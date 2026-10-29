#import "common/lighting"
#import "common/srgb"

// How a viewport's scene is shown (`DisplayMode`): as it is, where the engine's passes tonemapped it; or, under the
// physical light, by the engine's curve in the engine's own domain (the light encoded for the display, as the engine's
// values are, then curved, its output the display value itself, so the authored levels' diffuse shows as the engine
// shows it), or by a display transform to the display's linear colours, then encoded for it. The transforms take
// scene-referred light, middle grey at 0.18.

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
// The most of white PBR Neutral's inverse takes a shown colour for: half an eight-bit step under it, the brightest a
// display tells from white. The shoulder reaches white only at endless light, so what shows white comes back as the
// light the shoulder shows that close to it, about 30, rather than as near endless light.
const NEUTRAL_INVERSE_TOP: f32 = 1.0 - 0.5 / 255.0;
// The matrices above inverted exactly (WGSL has no inverse), for the transforms' inverses.
const SRGB_TO_REC2020_INVERSE: mat3x3<f32> = mat3x3<f32>(
  vec3<f32>(1.66051228, -0.124563464, -0.0181675039), vec3<f32>(-0.587704679, 1.13294912, -0.100559587),
  vec3<f32>(-0.0728663674, -0.00827236161, 1.11871703),
);
const REC2020_TO_SRGB_INVERSE: mat3x3<f32> = mat3x3<f32>(
  vec3<f32>(0.627409440, 0.0691248390, 0.0164233580), vec3<f32>(0.329260252, 0.919548614, 0.0880478476),
  vec3<f32>(0.0432718936, 0.0113207668, 0.895616714),
);
const AGX_INSET_INVERSE: mat3x3<f32> = mat3x3<f32>(
  vec3<f32>(1.19744108, -0.196474626, -0.146557417), vec3<f32>(-0.144261513, 1.35409513, -0.108284059),
  vec3<f32>(-0.0531795642, -0.157620505, 1.25484148),
);
const AGX_OUTSET_INVERSE: mat3x3<f32> = mat3x3<f32>(
  vec3<f32>(0.899796956, 0.111420989, 0.111420989), vec3<f32>(0.0871996192, 0.875575586, 0.0871996192),
  vec3<f32>(0.0130034249, 0.0130034249, 0.801379392),
);
// The inverse of AgX's sigmoid: halvings that bracket the answer within a sixteenth of the unit range, then Newton's
// steps inside the bracket, which the sigmoid's slope, never under 0.1 there, takes within 2e-5 of the answer: half the
// sigmoid's evaluations of halving all the way, which a pass inverting every particle's texel affords.
const AGX_SIGMOID_HALVINGS: u32 = 4u;
const AGX_SIGMOID_NEWTON_STEPS: u32 = 3u;

// AgX's sigmoid, as a polynomial fit over the log-encoded unit range.
fn agx_contrast(x: vec3<f32>) -> vec3<f32> {
  let x2: vec3<f32> = x * x;
  let x4: vec3<f32> = x2 * x2;

  return 15.5 * x4 * x2 - 40.14 * x4 * x + 31.96 * x4 - 6.868 * x2 * x + 0.4298 * x2 + 0.1191 * x - 0.00232;
}

// The slope of AgX's sigmoid (`agx_contrast`), its polynomial's derivative.
fn agx_contrast_slope(x: vec3<f32>) -> vec3<f32> {
  let x2: vec3<f32> = x * x;
  let x4: vec3<f32> = x2 * x2;

  return 93.0 * x4 * x - 200.7 * x4 + 127.84 * x2 * x - 20.604 * x2 + 0.8596 * x + 0.1191;
}

// AgX's base look: scene light to the display's linear colours.
fn agx(light: vec3<f32>) -> vec3<f32> {
  let inset: vec3<f32> = AGX_INSET * (SRGB_TO_REC2020 * light);
  let encoded: vec3<f32> = saturate((log2(max(inset, vec3<f32>(1e-10))) - AGX_MIN_EV) / (AGX_MAX_EV - AGX_MIN_EV));
  let shown: vec3<f32> = pow(max(AGX_OUTSET * agx_contrast(encoded), vec3<f32>(0.0)), vec3<f32>(2.2));

  return saturate(REC2020_TO_SRGB * shown);
}

// Khronos PBR Neutral's shoulder: scene light to the display's linear colours, as they are below it, compressed and
// desaturated towards white past it. Its toe, an offset of 0.04 off the light faded to nothing at black, is left out:
// its fade squares the dark, which a night lives in whole, doubling a night's contrast into its sky's eight-bit steps,
// where the engine's curve is near linear; the anchor brings the frame's key to the engine's showing without it.
fn pbr_neutral(light: vec3<f32>) -> vec3<f32> {
  let peak: f32 = max(light.r, max(light.g, light.b));

  if (peak < NEUTRAL_START) {
    return saturate(light);
  }

  let reach: f32 = 1.0 - NEUTRAL_START;
  let compressed: f32 = 1.0 - reach * reach / (peak + reach - NEUTRAL_START);
  let scaled: vec3<f32> = light * (compressed / peak);
  let grey: f32 = 1.0 - 1.0 / (NEUTRAL_DESATURATION * (peak - compressed) + 1.0);

  return saturate(mix(scaled, vec3<f32>(compressed), grey));
}

// AgX's sigmoid inverted over its unit range: the log-encoded value each channel's `curved` came from, bracketed by
// halving, the sigmoid rising throughout it, then found by Newton's steps held within the bracket.
fn agx_contrast_inverse(curved: vec3<f32>) -> vec3<f32> {
  var low: vec3<f32> = vec3<f32>(0.0);
  var high: vec3<f32> = vec3<f32>(1.0);

  for (var halving: u32 = 0u; halving < AGX_SIGMOID_HALVINGS; halving++) {
    let middle: vec3<f32> = (low + high) * 0.5;
    let is_under: vec3<bool> = agx_contrast(middle) < curved;

    low = select(low, middle, is_under);
    high = select(middle, high, is_under);
  }

  var found: vec3<f32> = (low + high) * 0.5;

  for (var step: u32 = 0u; step < AGX_SIGMOID_NEWTON_STEPS; step++) {
    found = clamp(found - (agx_contrast(found) - curved) / agx_contrast_slope(found), low, high);
  }

  return found;
}

// AgX's base look inverted: the scene light it shows as the display's linear colours `shown`.
fn agx_inverse(shown: vec3<f32>) -> vec3<f32> {
  let outset: vec3<f32> = pow(max(REC2020_TO_SRGB_INVERSE * shown, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
  let encoded: vec3<f32> = agx_contrast_inverse(AGX_OUTSET_INVERSE * outset);
  let inset: vec3<f32> = exp2(encoded * (AGX_MAX_EV - AGX_MIN_EV) + AGX_MIN_EV);

  return max(SRGB_TO_REC2020_INVERSE * (AGX_INSET_INVERSE * inset), vec3<f32>(0.0));
}

// PBR Neutral's shoulder inverted: the scene light it shows as `shown`, each channel held to `NEUTRAL_INVERSE_TOP`.
// Past the shoulder the shown colour's top channel is the compressed peak, which gives the peak back, then the share of
// white it was mixed with.
fn pbr_neutral_inverse(shown: vec3<f32>) -> vec3<f32> {
  let within: vec3<f32> = min(shown, vec3<f32>(NEUTRAL_INVERSE_TOP));
  let compressed: f32 = max(within.r, max(within.g, within.b));

  if (compressed < NEUTRAL_START) {
    return within;
  }

  let reach: f32 = 1.0 - NEUTRAL_START;
  let peak: f32 = reach * reach / (1.0 - compressed) - reach + NEUTRAL_START;
  let grey: f32 = 1.0 - 1.0 / (NEUTRAL_DESATURATION * (peak - compressed) + 1.0);
  let scaled: vec3<f32> = (within - grey * compressed) / max(1.0 - grey, 1e-6);

  return max(scaled * (peak / compressed), vec3<f32>(0.0));
}

// `graded` undone: the light that `contrast`, `saturation` and `stops` grade into `graded_light`.
fn ungraded(graded_light: vec3<f32>, contrast: f32, saturation: f32, stops: f32) -> vec3<f32> {
  let light: vec3<f32> = graded_light * exp2(-stops);
  let luminance_is: f32 = max(luminance(light), 1e-6);
  let scaled: vec3<f32> = (light - (1.0 - saturation) * luminance_is) / max(saturation, 1e-3);
  let luminance_was: f32 = MIDDLE_GREY * pow(luminance_is / MIDDLE_GREY, 1.0 / max(contrast, 1e-3));

  return max(scaled * (luminance_was / luminance_is), vec3<f32>(0.0));
}

// What a `display` shows a frame as without grading it: as it is (`DISPLAY_AS_IS`), contrast and saturation one, no
// stops.
const DISPLAY_UNGRADED: vec4<f32> = vec4<f32>(0.0, 1.0, 1.0, 0.0);

// Scene-referred light graded before the physical light's transform: its luminance raised to `contrast` in stops about
// middle grey, its colour mixed from that luminance by `saturation`, then lifted by `stops`; ones and nought stops
// leave it as it is.
fn graded(light: vec3<f32>, contrast: f32, saturation: f32, stops: f32) -> vec3<f32> {
  let luminance_was: f32 = max(luminance(light), 1e-6);
  let luminance_is: f32 = MIDDLE_GREY * pow(luminance_was / MIDDLE_GREY, contrast);
  let scaled: vec3<f32> = light * (luminance_is / luminance_was);

  return max(mix(vec3<f32>(luminance_is), scaled, saturation), vec3<f32>(0.0)) * exp2(stops);
}

// A colour of the scene as the display shows it under a `display` (`PresentUniform::display`): its mode, then the
// physical light's contrast and saturation and the transform's lift (`DisplayMode::get_lift`), which grade it before
// its transform.
fn shown_for_display(colour: vec3<f32>, display: vec4<f32>) -> vec3<f32> {
  let mode: u32 = u32(display.x);
  let light: vec3<f32> =
    select(colour, graded(colour, display.y, display.z, display.w), mode >= DISPLAY_ENGINE_ENCODED);

  switch (mode) {
    case DISPLAY_TONEMAPPED: {
      return tonemap(colour, 1.0);
    }
    case DISPLAY_ENGINE_ENCODED: {
      return tonemap(srgb_encoded(light), 1.0);
    }
    case DISPLAY_AGX: {
      return srgb_encoded(agx(light));
    }
    case DISPLAY_PBR_NEUTRAL: {
      return srgb_encoded(pbr_neutral(light));
    }
    default: {
      return colour;
    }
  }
}

// `shown_for_display` undone: the colour of the scene a `display` shows as `shown`, within the display's range.
fn scene_for_display(shown: vec3<f32>, display: vec4<f32>) -> vec3<f32> {
  let mode: u32 = u32(display.x);
  var light: vec3<f32> = shown;

  switch (mode) {
    case DISPLAY_TONEMAPPED: {
      return untonemapped(shown, 1.0);
    }
    case DISPLAY_ENGINE_ENCODED: {
      light = srgb_decoded(untonemapped(shown, 1.0));
    }
    case DISPLAY_AGX: {
      light = agx_inverse(srgb_decoded(shown));
    }
    case DISPLAY_PBR_NEUTRAL: {
      light = pbr_neutral_inverse(srgb_decoded(shown));
    }
    default: {
      return shown;
    }
  }

  return ungraded(light, display.y, display.z, display.w);
}

// A picture of the game's (the sky, its clouds and sun, as the physical light writes them for `scene_from_shown`) as a
// `display` shows it as the engine's curve does: the game paints its pictures for the engine's display, so under AgX
// and PBR Neutral, scene-referred transforms that would show a display-referred picture with their own contrast and
// colour, the picture enters the scene as the light the transform shows as the engine's curve shows the picture's
// light, as a film's pipeline brings a display-referred plate in through the inverse of its view. Elsewhere as it is.
fn picture_for_display(shown: vec3<f32>, display: vec4<f32>) -> vec3<f32> {
  let mode: u32 = u32(display.x);

  if (mode != DISPLAY_AGX && mode != DISPLAY_PBR_NEUTRAL) {
    return shown;
  }

  let light: vec3<f32> = untonemapped(srgb_decoded(shown), 1.0);
  let held: vec3<f32> = scene_for_display(tonemap(srgb_encoded(light), 1.0), display);

  return srgb_encoded(tonemap(held, 1.0));
}

// `picture_for_display` undone: the picture a `display` brought in as `entered`, for a reader of the seen sky that
// takes it as light, which no view transform enters (`sky_light`).
fn picture_from_display(entered: vec3<f32>, display: vec4<f32>) -> vec3<f32> {
  let mode: u32 = u32(display.x);

  if (mode != DISPLAY_AGX && mode != DISPLAY_PBR_NEUTRAL) {
    return entered;
  }

  let held: vec3<f32> = untonemapped(srgb_decoded(entered), 1.0);
  let light: vec3<f32> = srgb_decoded(untonemapped(shown_for_display(held, display), 1.0));

  return srgb_encoded(tonemap(light, 1.0));
}
