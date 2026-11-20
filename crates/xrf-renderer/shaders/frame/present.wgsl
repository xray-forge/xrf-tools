#import "common/noise"
#import "common/camera"
#import "common/fullscreen"
#import "common/lighting"
#import "common/occlusion"
#import "common/octahedral"
#import "common/present"
#import "common/reflection_end"
#import "common/debug_colours"
#import "common/luminance"
#import "common/srgb"
#import "common/display_transform"
#import "common/lens"

// The viewport's finished scene put into its rectangle of the window; or, for a debug view, one of the targets the
// scene was built from.

// The scene, the targets a debug view shows, what
// the frame shows, the frame upscaled or the scene again, the bloom, finished in
// the engine's 256-square target or at half the frame's size for the enhanced bloom, the indirect light gathered at
// half the frame's size or traced at its size, and the reflections and how their rays ended, at the size they are
// traced at.
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
const VIEW_DETAIL_SUN: u32 = 24u;
const VIEW_DETAIL_HEMI: u32 = 25u;
const VIEW_DETAIL_LIGHT: u32 = 26u;
const VIEW_AI_HIGH_COVER: u32 = 27u;
const VIEW_AI_LOW_COVER: u32 = 28u;
const VIEW_AI_LINKS: u32 = 29u;
// The first of FSR's debug views, which FSR draws in place of the frame it resolved.
const VIEW_FSR_FIRST: u32 = 30u;
// What the reflections view shows where no ray was traced and the cube stands.
const REFLECTION_MISS: vec3<f32> = vec3<f32>(0.05, 0.1, 0.45);
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

// How bright the detail slots' and the AI map's views show the albedo where they show no slot or node.
const UNSLOTTED_DIM: f32 = 0.25;

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

// Half a step either way by interleaved gradient noise (Jimenez, 2014), so a flat gradient falls between two steps as
// a fine grain rather than as bands.
fn output_dither(pixel: vec2<f32>) -> vec3<f32> {
  let noise: f32 = gradient_noise(pixel);

  return vec3<f32>((noise - 0.5) * OUTPUT_STEP);
}

// How far the grain at its strongest moves a displayed value either way.
const GRAIN_REACH: f32 = 0.15;

// A displayed colour on the camera's film (`Present::grain`): its grain at the pixel, by its brightness, after the
// display transform as film's grain is seen, and before the dither the window rounds by.
fn on_film(color: vec3<f32>, pixel: vec2<f32>) -> vec3<f32> {
  if (present.grain.x <= 0.0) {
    return color;
  }

  let grain: f32 = lens_grain(pixel, present.grain.y, u32(present.grain.z));

  return color + grain * present.grain.x * GRAIN_REACH * lens_grain_response(luminance(color));
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
        return REFLECTION_MISS;
      }

      // What the rays met as the display value combine adds it, in the engine's units: the light by the light's scale,
      // encoded where the light is summed (`PresentUniform::inspect`). Its alpha is the distance met at.
      let met: vec3<f32> = max(traced.rgb, vec3<f32>(0.0)) / max(present.inspect.y, 1e-12);

      return select(met, srgb_encoded(met), present.inspect.z > 0.5);
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
    // The grass wrote the slot's value, or the AI map the node's, over the light target, with one in alpha where it
    // shows one.
    case VIEW_DETAIL_SUN, VIEW_DETAIL_HEMI, VIEW_DETAIL_LIGHT, VIEW_AI_HIGH_COVER, VIEW_AI_LOW_COVER, VIEW_AI_LINKS: {
      let slot: vec4<f32> = textureLoad(light_target, texel, 0);

      if (slot.a > 0.5) {
        return slot.rgb;
      }

      return vec3<f32>(select(0.0, luminance(textureLoad(albedo_target, texel, 0).rgb) * UNSLOTTED_DIM, stored > 0.0));
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

// Drawn texels around a marked model its outline reaches, how much of its colour tints it, and how a faded texel shows:
// greyed by this share and dimmed to this much of its light.
const MARK_REACH: i32 = 1;
const MARK_TINT: f32 = 0.15;
const REST_GREY: f32 = 0.85;
const REST_DIM: f32 = 0.35;

// The colour the helpers marked a drawn texel in, with one in alpha where they marked it, held to the targets' edges.
fn mark_at(texel: vec2<i32>) -> vec4<f32> {
  let last: vec2<i32> = vec2<i32>(camera.viewport.xy) - 1;

  return textureLoad(mark_target, clamp(texel, vec2<i32>(0), last), 0);
}

// A shown colour with what the helpers and the selection lay over it: a marked model tinted in its helper's colour and
// outlined a texel round in it, what neither a mark nor the selection with its outline covers greyed and dimmed where
// the rest fades, then the selection's outline and tint over all.
fn laid_over(color: vec3<f32>, texel: vec2<i32>) -> vec3<f32> {
  let selected: f32 = selection_share(texel);
  var shown: vec3<f32> = color;
  var is_covered: bool = selected > 0.0;

  if (present.marks.x != 0.0) {
    let own: vec4<f32> = mark_at(texel);

    if (own.a > 0.5) {
      shown = mix(shown, own.rgb, MARK_TINT);
      is_covered = true;
    } else {
      for (var y: i32 = -MARK_REACH; y <= MARK_REACH && !is_covered; y += 1) {
        for (var x: i32 = -MARK_REACH; x <= MARK_REACH; x += 1) {
          let near: vec4<f32> = mark_at(texel + vec2<i32>(x, y));

          if (near.a > 0.5) {
            shown = near.rgb;
            is_covered = true;
            break;
          }
        }
      }
    }
  }

  if (present.marks.y != 0.0 && !is_covered) {
    shown = mix(shown, vec3<f32>(luminance(shown)), REST_GREY) * REST_DIM;
  }

  return mix(shown, present.selection.rgb, selected);
}

// The drawn texel under a point of the viewport, in its pixels.
fn drawn_texel(pixel: vec2<f32>) -> vec2<i32> {
  return to_drawn_texel(pixel, camera.viewport.xy, present.size);
}

// `combine_2`'s bloom added over a colour at a pixel, `combine_bloom`; its blur by the distortion's `z` came before the
// resolve (`frame/distort.wgsl`).
fn engine_bloomed(color: vec3<f32>, pixel: vec2<f32>) -> vec3<f32> {
  let bloom: vec4<f32> = textureSampleLevel(bloom_target, bloom_sampler, (pixel + 0.5) / present.size, 0.0);

  return color + bloom.rgb * bloom.a;
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

  // `combine_2`'s `USE_DISTORT`, its move and its blur, came before the resolve (`frame/distort.wgsl`).
  let drawn: vec3<f32> = select(textureLoad(scene, texel, 0).rgb, textureLoad(upscaled, vec2<i32>(pixel), 0).rgb,
    present.is_upscaled != 0u);

  let display: vec4<f32> = present.display;
  // The physical light's engine bloom is light, built from its linear scene, so it joins the scene as light before the
  // display transform; the engine's joins its shown frame after, as `combine_2` adds it.
  let is_bloom_light: bool = present.is_bloomed == BLOOM_ENGINE && u32(display.x) >= DISPLAY_ENGINE_ENCODED;
  var light: vec3<f32> = drawn;

  if (is_bloom_light) {
    light = engine_bloomed(drawn, pixel);
  }

  // A scene of linear light at its exposure is shown here, by the engine's curve or the physical light's display
  // transform and encoding, before everything the engine lays over its displayed frame.
  var color: vec3<f32> = shown_for_display(light, display);

  // The enhanced bloom screened over the frame, `1 - (1 - img)(1 - bloom)`, before its corrections.
  if (present.is_bloomed == BLOOM_ENHANCED) {
    let bloom: vec3<f32> = textureSampleLevel(bloom_target, bloom_sampler, (pixel + 0.5) / present.size, 0.0).rgb;
    let finished: vec3<f32> = on_film(corrected(1.0 - (1.0 - saturate(color)) * (1.0 - bloom)), pixel);
    return vec4<f32>(laid_over(finished, texel) + output_dither(in.clip.xy), 1.0);
  }

  // The engine's bloom over its shown frame, as `combine_2` adds it.
  if (present.is_bloomed == BLOOM_ENGINE && !is_bloom_light) {
    color = engine_bloomed(color, pixel);
  }

  return vec4<f32>(laid_over(on_film(corrected(color), pixel), texel) + output_dither(in.clip.xy), 1.0);
}
