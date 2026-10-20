#import "common/noise"
#import "common/camera"
#import "common/fullscreen"
#import "common/lighting"
#import "common/occlusion"
#import "common/octahedral"
#import "common/present"
#import "common/debanding"
#import "common/reflection_end"
#import "common/debug_colours"
#import "common/luminance"
#import "common/srgb"

// The viewport's finished scene put into its rectangle of the window, moved where the water and the particles distort
// it; or, for a debug view, one of the targets the scene was built from.

// The scene, what the water and the particles move what is seen through them by, the targets a debug view shows, what
// the frame shows, the frame upscaled or the scene again (filtered for the debanding's reads), the bloom, finished in
// the engine's 256-square target or at half the frame's size for the enhanced bloom, the indirect light at half the
// frame's size, and the reflections and how their rays ended, at the size they are traced at.
#import "generated/frame/present"

const VIEW_ALBEDO: u32 = 1u;
const VIEW_GLOSS: u32 = 2u;
const VIEW_NORMAL: u32 = 3u;
const VIEW_HEMI: u32 = 4u;
const VIEW_SUN: u32 = 5u;
const VIEW_MATERIAL: u32 = 6u;
const VIEW_DEPTH: u32 = 7u;
const VIEW_LIGHT: u32 = 8u;
const VIEW_MOTION: u32 = 10u;
const VIEW_INDIRECT_LIGHT: u32 = 11u;
const VIEW_REFLECTIONS: u32 = 12u;
const VIEW_REFLECTION_ENDS: u32 = 13u;
const VIEW_PARALLAX: u32 = 14u;
const VIEW_BUMPS: u32 = 15u;
const VIEW_HEIGHTS: u32 = 16u;
const VIEW_LUMINANCE: u32 = 17u;
const VIEW_ALBEDO_RANGE: u32 = 18u;
const VIEW_OVERDRAW: u32 = 19u;
const VIEW_CASCADES: u32 = 20u;
const VIEW_LAMPS: u32 = 21u;
const VIEW_SAMPLED_MIP: u32 = 22u;
const VIEW_SPECULAR: u32 = 23u;
// The first of FSR's debug views, which FSR draws in place of the frame it resolved.
const VIEW_FSR_FIRST: u32 = 24u;
// What the reflections view shows where a ray met nothing and the cube stands.
const REFLECTION_MISS: vec3<f32> = vec3<f32>(0.05, 0.1, 0.45);
// What the reflections view multiplies what was met by before its shoulder.
const REFLECTION_VIEW_GAIN: f32 = 4.0;
// Drawn pixels of motion the motion view spans from black to full colour on each axis.
const MOTION_VIEW_RANGE: f32 = 16.0;

// `is_bloomed` for the engine's bloom and for the enhanced one.
const BLOOM_ENGINE: u32 = 1u;
const BLOOM_ENHANCED: u32 = 2u;

// Metres the depth view spreads over, logarithmically, so a metre up close and a kilometre away both read.
const DEPTH_VIEW_RANGE: f32 = 5000.0;


// The linear albedo the albedo range view takes as plausible, and what it shows below and above it.
const ALBEDO_LEAST: f32 = 0.03;
const ALBEDO_MOST: f32 = 0.9;
const ALBEDO_UNDER: vec3<f32> = vec3<f32>(0.1, 0.25, 0.95);
const ALBEDO_OVER: vec3<f32> = vec3<f32>(0.95, 0.15, 0.1);

// The counts the overdraw and the lamps views reach red at.
const OVERDRAW_MOST: f32 = 8.0;
const LAMPS_MOST: f32 = 16.0;

// How much of a cascade's tint stays where it shadows, and the drawn texels a stripe of the grass's reach repeats over
// and spans.
const CASCADE_SHADOWED: f32 = 0.35;
const GRASS_STRIPE_PERIOD: i32 = 8;
const GRASS_STRIPE_WIDTH: i32 = 2;

// One step of the eight-bit window the frame is written to.
const OUTPUT_STEP: f32 = 1.0 / 255.0;

// What the distortion target holds where nothing distorts it.
const NEUTRAL_DISTORTION: f32 = 127.0 / 255.0;

// Metres, and the share of its own depth, what a move reads may stand nearer than what the pixel shows behind it.
const NEARER_MARGIN: f32 = 0.25;
const NEARER_SHARE: f32 = 0.02;

// Half a step either way by interleaved gradient noise (Jimenez, 2014), so a flat gradient falls between two steps as
// a fine grain rather than as bands.
fn output_dither(pixel: vec2<f32>) -> vec3<f32> {
  let noise: f32 = gradient_noise(pixel);

  return vec3<f32>((noise - 0.5) * OUTPUT_STEP);
}

// How far along the view a texel of the viewport's targets lies, farther than anything where nothing was drawn.
fn view_distance(texel: vec2<i32>) -> f32 {
  let stored: f32 = textureLoad(depth_target, texel, 0);

  return select(-camera_view_position(vec2<f32>(texel) + 0.5, stored).z, 1e6, stored <= 0.0);
}

// One target of the frame at a texel, as a colour.
fn shown_target(texel: vec2<i32>) -> vec3<f32> {
  let stored: f32 = textureLoad(depth_target, texel, 0);
  let distance: f32 = -camera_view_position(vec2<f32>(texel) + 0.5, stored).z;

  switch (present.view) {
    case VIEW_ALBEDO: {
      return textureLoad(albedo_target, texel, 0).rgb;
    }
    case VIEW_GLOSS: {
      return vec3<f32>(textureLoad(albedo_target, texel, 0).a);
    }
    case VIEW_NORMAL: {
      return octahedral_decode(textureLoad(normal_target, texel, 0).xy) * 0.5 + 0.5;
    }
    case VIEW_HEMI: {
      return vec3<f32>(textureLoad(material_target, texel, 0).x);
    }
    case VIEW_SUN: {
      return vec3<f32>(textureLoad(material_target, texel, 0).y);
    }
    case VIEW_MATERIAL: {
      return vec3<f32>(textureLoad(material_target, texel, 0).z);
    }
    case VIEW_DEPTH: {
      return vec3<f32>(select(log(distance + 1.0) / log(DEPTH_VIEW_RANGE + 1.0), 1.0, stored <= 0.0));
    }
    case VIEW_LIGHT: {
      return textureLoad(light_target, texel, 0).rgb;
    }
    case VIEW_SPECULAR: {
      return vec3<f32>(textureLoad(light_target, texel, 0).a);
    }
    case VIEW_MOTION: {
      let pixels: vec2<f32> = textureLoad(motion_target, texel, 0).xy * camera.viewport.xy;

      return vec3<f32>(pixels / MOTION_VIEW_RANGE + 0.5, 0.5);
    }
    case VIEW_INDIRECT_LIGHT: {
      let is_gathered: bool = present.is_indirect != 0u && stored > 0.0;

      if (!is_gathered) {
        return vec3<f32>(0.0);
      }

      return upsampled_light(indirect_light, vec2<f32>(texel), distance);
    }
    case VIEW_REFLECTIONS: {
      if (present.is_reflected == 0u || stored <= 0.0) {
        return vec3<f32>(0.0);
      }

      let traced: vec4<f32> = textureLoad(reflections, min(texel, vec2<i32>(textureDimensions(reflections)) - 1), 0);

      if (traced.a < 0.0) {
        return vec3<f32>(0.0);
      }

      // What was met, brightened through a shoulder so a dim frame's reflections read, over the miss's colour by how
      // far the hit is trusted.
      let met: vec3<f32> = traced.rgb / max(traced.a, 1e-4) * REFLECTION_VIEW_GAIN;

      return mix(REFLECTION_MISS, met / (1.0 + met), saturate(traced.a));
    }
    // The G-buffer draws the parallax, the bumps, the heights or the sampled mip into the albedo for these views alone.
    case VIEW_PARALLAX, VIEW_BUMPS, VIEW_HEIGHTS, VIEW_SAMPLED_MIP: {
      return textureLoad(albedo_target, texel, 0).rgb;
    }
    case VIEW_LUMINANCE: {
      let shown: f32 = luminance(textureLoad(scene, texel, 0).rgb);

      return stop_colour(log2(max(shown, 1e-6) / MIDDLE_GREY));
    }
    case VIEW_ALBEDO_RANGE: {
      if (stored <= 0.0) {
        return vec3<f32>(0.0);
      }

      let albedo: vec3<f32> = textureLoad(albedo_target, texel, 0).rgb;
      let linear: f32 = luminance(srgb_decoded(albedo));
      let grey: vec3<f32> = vec3<f32>(luminance(albedo));

      return select(select(grey, ALBEDO_OVER, linear > ALBEDO_MOST), ALBEDO_UNDER, linear < ALBEDO_LEAST);
    }
    // The overdraw pass added one into the light target for each surface drawn over the pixel.
    case VIEW_OVERDRAW: {
      return count_colour(textureLoad(light_target, texel, 0).x, OVERDRAW_MOST);
    }
    // The lamps' pass wrote the pixel's cluster's count into the light target, the sun nought.
    case VIEW_LAMPS: {
      if (stored <= 0.0) {
        return vec3<f32>(0.0);
      }

      return count_colour(textureLoad(light_target, texel, 0).x, LAMPS_MOST);
    }
    // The sun's pass wrote the cascade the pixel reads, from one, and how much of the sun reaches it.
    case VIEW_CASCADES: {
      if (stored <= 0.0) {
        return vec3<f32>(0.0);
      }

      let sun: vec4<f32> = textureLoad(light_target, texel, 0);
      let shown: vec3<f32> = cascade_colour(sun.x) * mix(CASCADE_SHADOWED, 1.0, saturate(sun.y));
      let is_stripe: bool = (texel.x + texel.y) % GRASS_STRIPE_PERIOD < GRASS_STRIPE_WIDTH;

      return select(shown, mix(shown, vec3<f32>(1.0), 0.6), distance < present.inspect.x && is_stripe);
    }
    case VIEW_REFLECTION_ENDS: {
      if (present.is_reflected == 0u || stored <= 0.0) {
        return vec3<f32>(0.0);
      }

      let last: vec2<i32> = vec2<i32>(textureDimensions(reflection_ends)) - 1;
      let traced: vec2<i32> = clamp(vec2<i32>(vec2<f32>(texel) / present.reflection_ratio), vec2<i32>(0), last);

      return reflection_end_colour(reflection_end_code(textureLoad(reflection_ends, traced, 0).x));
    }
    default: {
      let is_searched: bool = present.is_occluded != 0u && stored > 0.0;

      return vec3<f32>(select(1.0, upsampled_occlusion(occlusion_target, vec2<f32>(texel), distance), is_searched));
    }
  }
}

// `img_corrections` (Anomaly's `combine_2`): the finished frame exposed, graded towards a colour in its mid tones,
// saturated and raised to its gamma.
fn corrected(color: vec3<f32>) -> vec3<f32> {
  let luminance_vector: vec3<f32> = vec3<f32>(0.2125, 0.7154, 0.0721);
  var image: vec3<f32> = color * present.corrections.x;
  let luminance: f32 = dot(image, luminance_vector);

  if (any(present.grading.rgb > vec3<f32>(0.0))) {
    let graded: vec3<f32> = mix(mix(vec3<f32>(0.0), present.grading.rgb, saturate(luminance * 2.0)), vec3<f32>(1.0),
      saturate(luminance - 0.5) * 2.0);

    image = saturate(mix(image, graded, saturate(luminance * 0.15)));
  }

  image = mix(image, vec3<f32>(dot(image, luminance_vector)), 1.0 - present.corrections.z);

  return pow(max(image, vec3<f32>(0.0)), vec3<f32>(1.0 / present.corrections.y));
}

// Drawn texels around a selection its outline reaches, and how much of its colour tints what is selected.
const SELECTION_REACH: i32 = 2;
const SELECTION_TINT: f32 = 0.15;

// Whether the G-buffer marked a drawn texel selected, held to the targets' edges.
fn is_marked(texel: vec2<i32>) -> bool {
  let last: vec2<i32> = vec2<i32>(camera.viewport.xy) - 1;

  return has_mark(textureLoad(material_target, clamp(texel, vec2<i32>(0), last), 0).a, MARK_SELECTED);
}

// How much of the selection's colour a texel shows: its outline outside what is selected, a tint over it, or none.
fn selection_share(texel: vec2<i32>) -> f32 {
  if (present.selection.w == 0.0) {
    return 0.0;
  }

  if (is_marked(texel)) {
    return SELECTION_TINT;
  }

  for (var y: i32 = -SELECTION_REACH; y <= SELECTION_REACH; y += 1) {
    for (var x: i32 = -SELECTION_REACH; x <= SELECTION_REACH; x += 1) {
      if (is_marked(texel + vec2<i32>(x, y))) {
        return 1.0;
      }
    }
  }

  return 0.0;
}

// The drawn texel under a point of the viewport, in its pixels.
fn drawn_texel(pixel: vec2<f32>) -> vec2<i32> {
  return to_drawn_texel(pixel, camera.viewport.xy, present.size);
}

@fragment
fn fs_present(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let pixel: vec2<f32> = floor(in.clip.xy - present.origin);
  let texel: vec2<i32> = drawn_texel(pixel);

  // FSR's debug views: what FSR drew in place of the frame, as it stands.
  if (present.view >= VIEW_FSR_FIRST) {
    let drawn: vec3<f32> = select(textureLoad(scene, texel, 0).rgb, textureLoad(upscaled, vec2<i32>(pixel), 0).rgb,
      present.is_upscaled != 0u);

    return vec4<f32>(drawn, 1.0);
  }

  if (present.view != 0u) {
    return vec4<f32>(saturate(shown_target(texel)), 1.0);
  }

  var read: vec2<f32> = pixel;
  let strength: f32 = present.distortion;

  // `combine_2`'s `USE_DISTORT`: the scene read where the distortion target moves each pixel, `(distort.xy - .5) *
  // def_distort`. A move that would read something standing nearer than what the pixel shows reads the pixel itself:
  // a departure from the engine, whose water copies a railing standing in it into the water beside it.
  if (strength > 0.0) {
    let offset: vec2<f32> = (textureLoad(distortion, texel, 0).xy - NEUTRAL_DISTORTION) * strength;

    // A pixel nothing moves reads itself, with no depths to compare.
    if (any(offset != vec2<f32>(0.0))) {
      let moved: vec2<f32> = clamp(pixel + offset * present.size, vec2<f32>(0.0), present.size - 1.0);
      let here: f32 = view_distance(texel);
      let there: f32 = view_distance(drawn_texel(moved));

      if (there >= here - max(NEARER_MARGIN, here * NEARER_SHARE)) {
        read = moved;
      }
    }
  }

  var color: vec3<f32> = select(textureLoad(scene, drawn_texel(read), 0).rgb,
    textureLoad(upscaled, vec2<i32>(read), 0).rgb, present.is_upscaled != 0u);

  let is_linear: bool = present.tonemap.x > 0.5;

  // A scene of linear light at its exposure is tonemapped here, by the engine's curve, before everything the engine
  // lays over its tonemapped frame.
  if (is_linear) {
    color = tonemap(color, 1.0);
  }

  // Linear light, tonemapped, is encoded for the display once, before what the engine lays over its displayed frame.
  if (present.tonemap.y > 0.5) {
    color = srgb_encoded(color);
  }

  // The sky debanding, where nothing was drawn, over the shown steps it bands in: its neighbours read about the pixel
  // itself, not where the distortion moved its read.
  if (present.deband.x > 0.0 && textureLoad(depth_target, texel, 0) <= 0.0) {
    color = debanded(upscaled, bloom_sampler, color, (pixel + 0.5) / present.size, 1.0 / present.size,
      u32(present.deband.x), present.deband.y, present.deband.z, vec2<bool>(is_linear, present.tonemap.y > 0.5));
  }

  // The enhanced bloom screened over the frame, `1 - (1 - img)(1 - bloom)`, before its corrections; where the
  // distortion blurs, the corrected frame then goes towards the bloom by the distortion's `z`, as the engine's does.
  if (present.is_bloomed == BLOOM_ENHANCED) {
    let bloom: vec3<f32> = textureSampleLevel(bloom_target, bloom_sampler, (read + 0.5) / present.size, 0.0).rgb;
    var finished: vec3<f32> = corrected(1.0 - (1.0 - saturate(color)) * (1.0 - bloom));

    if (strength > 0.0) {
      finished = mix(finished, bloom * DEF_HDR, textureLoad(distortion, texel, 0).z);
    }

    let shown: vec3<f32> = mix(finished, present.selection.rgb, selection_share(texel));

    return vec4<f32>(shown + output_dither(in.clip.xy), 1.0);
  }

  // `combine_2`: the bloom read where the scene is; where the distortion blurs, the scene first goes towards it by the
  // distortion's `z`, `lerp(img, bloom * def_hdr, distort.z)`; then the bloom added, `combine_bloom`.
  if (present.is_bloomed == BLOOM_ENGINE) {
    let bloom: vec4<f32> = textureSampleLevel(bloom_target, bloom_sampler, (read + 0.5) / present.size, 0.0);

    if (strength > 0.0) {
      color = mix(color, bloom.rgb * DEF_HDR, textureLoad(distortion, texel, 0).z);
    }

    color += bloom.rgb * bloom.a;
  }

  let shown: vec3<f32> = mix(corrected(color), present.selection.rgb, selection_share(texel));

  return vec4<f32>(shown + output_dither(in.clip.xy), 1.0);
}
