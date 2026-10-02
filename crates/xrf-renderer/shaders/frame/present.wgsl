#import "common/camera"
#import "common/fullscreen"
#import "common/occlusion"
#import "common/octahedral"
#import "common/present"

// The viewport's finished scene put into its rectangle of the window, moved where the water distorts it; or, for a
// debug view, one of the targets the scene was built from.

@group(1) @binding(0) var scene: texture_2d<f32>;
// How far the water moves what is seen through it, around what the target is cleared to.
@group(1) @binding(1) var distortion: texture_2d<f32>;
@group(1) @binding(2) var depth_target: texture_depth_2d;
@group(1) @binding(3) var albedo_target: texture_2d<f32>;
@group(1) @binding(4) var normal_target: texture_2d<f32>;
@group(1) @binding(5) var material_target: texture_2d<f32>;
@group(1) @binding(6) var light_target: texture_2d<f32>;
@group(1) @binding(7) var occlusion_target: texture_2d<f32>;
@group(1) @binding(8) var<uniform> present: Present;
// The frame upscaled to the viewport's size, or the scene again where it is drawn at that size.
@group(1) @binding(9) var upscaled: texture_2d<f32>;
@group(1) @binding(10) var motion_target: texture_2d<f32>;

const VIEW_ALBEDO: u32 = 1u;
const VIEW_GLOSS: u32 = 2u;
const VIEW_NORMAL: u32 = 3u;
const VIEW_HEMI: u32 = 4u;
const VIEW_SUN: u32 = 5u;
const VIEW_MATERIAL: u32 = 6u;
const VIEW_DEPTH: u32 = 7u;
const VIEW_LIGHT: u32 = 8u;
const VIEW_MOTION: u32 = 10u;
// Drawn pixels of motion the motion view spans from black to full colour on each axis.
const MOTION_VIEW_RANGE: f32 = 16.0;

// Metres the depth view spreads over, logarithmically, so a metre up close and a kilometre away both read.
const DEPTH_VIEW_RANGE: f32 = 5000.0;

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
  let noise: f32 = fract(fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))) * 52.9829189);

  return vec3<f32>((noise - 0.5) * OUTPUT_STEP);
}

// How far along the view a texel of the viewport's targets lies, farther than anything where nothing was drawn.
fn view_distance(texel: vec2<i32>) -> f32 {
  let stored: f32 = textureLoad(depth_target, texel, 0);

  return select(-camera_view_position(vec2<f32>(texel) + 0.5, stored).z, 1e6, stored <= 0.0);
}

// `combine_2`'s `USE_DISTORT`: the scene read where the distortion target moves each pixel, `(distort.xy - .5) *
// def_distort`. A move that would read something standing nearer than what the pixel shows reads the pixel itself: a
// departure from the engine, whose water copies a railing standing in it into the water beside it.
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
    case VIEW_MOTION: {
      let pixels: vec2<f32> = textureLoad(motion_target, texel, 0).xy * camera.viewport.xy;

      return vec3<f32>(pixels / MOTION_VIEW_RANGE + 0.5, 0.5);
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

// The drawn texel under a point of the viewport, in its pixels.
fn drawn_texel(pixel: vec2<f32>) -> vec2<i32> {
  return to_drawn_texel(pixel, camera.viewport.xy, present.size);
}

@fragment
fn fs_present(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let pixel: vec2<f32> = floor(in.clip.xy - present.origin);
  let texel: vec2<i32> = drawn_texel(pixel);

  if (present.view != 0u) {
    return vec4<f32>(saturate(shown_target(texel)), 1.0);
  }

  var read: vec2<f32> = pixel;
  let strength: f32 = camera.switches.w;

  if (strength > 0.0) {
    let offset: vec2<f32> = (textureLoad(distortion, texel, 0).xy - NEUTRAL_DISTORTION) * strength;
    let moved: vec2<f32> = clamp(pixel + offset * present.size, vec2<f32>(0.0), present.size - 1.0);
    let here: f32 = view_distance(texel);
    let there: f32 = view_distance(drawn_texel(moved));

    if (there >= here - max(NEARER_MARGIN, here * NEARER_SHARE)) {
      read = moved;
    }
  }

  let color: vec3<f32> = select(textureLoad(scene, drawn_texel(read), 0).rgb,
    textureLoad(upscaled, vec2<i32>(read), 0).rgb, present.is_upscaled != 0u);

  return vec4<f32>(corrected(color) + output_dither(in.clip.xy), 1.0);
}
