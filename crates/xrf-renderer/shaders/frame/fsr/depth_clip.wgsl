#import "common/fullscreen"
#import "frame/fsr/common"

// `ffx_fsr2_depth_clip.h`: the prepared colour in YCoCg with how far its surface was disoccluded, and the reactive and
// accumulation masks.

@group(0) @binding(1) var color: texture_2d<f32>;
@group(0) @binding(2) var motion_target: texture_2d<f32>;
// The reconstructed depth of the frame before, a depth's bits a drawn texel.
@group(0) @binding(3) var<storage, read> reconstructed: array<u32>;
@group(0) @binding(4) var dilated_depth: texture_2d<f32>;
@group(0) @binding(5) var dilated_motion: texture_2d<f32>;
@group(0) @binding(6) var previous_dilated_motion: texture_2d<f32>;
@group(0) @binding(7) var reactive_mask: texture_2d<f32>;
@group(0) @binding(8) var linear_sampler: sampler;

// `Ksep`: the depth separation a pixel of the view can tell, per unit of distance.
const DEPTH_SEPARATION: f32 = 1.37e-5;
// The diagonal of a 1080p view, which the clip's power is scaled up to.
const FULL_HD_DIAGONAL: f32 = 2202.9071;

struct DepthClipOutput {
  @location(0) prepared: vec4<f32>,
  @location(1) masks: vec4<f32>,
};

fn load_reconstructed(at: vec2<f32>) -> f32 {
  let texel: vec2<i32> = clamped_texel(at, fsr.render_size);

  return bitcast<f32>(reconstructed[u32(texel.y) * u32(fsr.render_size.x) + u32(texel.x)]);
}

fn load_dilated_depth(at: vec2<f32>) -> f32 {
  return textureLoad(dilated_depth, clamped_texel(at, fsr.render_size), 0).x;
}

fn load_color(at: vec2<f32>) -> vec3<f32> {
  return textureLoad(color, clamped_texel(at, fsr.render_size), 0).rgb;
}

// `LoadInputMotionVector`: FSR's motion is the renderer's turned, from now to the frame before.
fn load_fsr_motion(at: vec2<f32>) -> vec2<f32> {
  return -textureLoad(motion_target, clamped_texel(at, fsr.render_size), 0).xy;
}

// `GetViewSpacePosition`.
fn to_view_position(at: vec2<f32>, depth: f32) -> vec3<f32> {
  let z: f32 = to_view_depth(depth);
  let ndc: vec2<f32> = at / fsr.render_size * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0);

  return vec3<f32>(fsr.device_to_view.z * ndc.x * z, fsr.device_to_view.w * ndc.y * z, z);
}

// `ComputeDepthClip`: how far the surface stands behind what stood there the frame before, against the separation the
// view's resolution and field can tell.
fn depth_clip(position: vec2<f32>, moved: vec2<f32>, current: f32) -> f32 {
  let current_view: f32 = to_view_depth(current);
  let footprint: BilinearFootprint = bilinear_footprint((position + 0.5) / fsr.render_size + moved, fsr.render_size);
  let half_viewport: f32 = length(fsr.render_size);
  let power: f32 = mix(1.0, 3.0, saturate(half_viewport / FULL_HD_DIAGONAL));
  var clipped: f32 = 0.0;
  var weights: f32 = 0.0;

  for (var corner: i32 = 0; corner < 4; corner++) {
    let at: vec2<f32> = footprint.base + footprint_corner(corner);
    let weight: f32 = footprint.weights[corner];
    let previous: f32 = load_reconstructed(at);
    let previous_view: f32 = to_view_depth(previous);
    let difference: f32 = current_view - previous_view;

    if (is_on_screen(at, fsr.render_size) && weight > RECONSTRUCTED_DEPTH_WEIGHT_THRESHOLD && difference > 0.0) {
      let plane: f32 = min(previous, current);
      let centre: vec3<f32> = to_view_position(floor(fsr.render_size * 0.5), plane);
      let edge: vec3<f32> = to_view_position(vec2<f32>(0.0), plane);
      let required: f32 = DEPTH_SEPARATION * (length(edge) / length(centre)) * half_viewport
        * max(current_view, previous_view);

      clipped += pow(saturate(required / max(difference, 1e-10)), power) * weight;
      weights += weight;
    }
  }

  return select(0.0, saturate(1.0 - clipped / max(weights, 1e-10)), weights > 0.0);
}

// `ComputeMotionDivergence`: how far the motion around turns from this texel's.
fn motion_divergence(position: vec2<f32>) -> f32 {
  let nucleus: vec2<f32> = load_fsr_motion(position);
  var max_velocity: f32 = length(nucleus);
  var convergence: f32 = 1.0;

  for (var index: i32 = 0; index < 9; index++) {
    let around: vec2<f32> = load_fsr_motion(position + neighbour(index));
    let velocity: f32 = length(around);

    max_velocity = max(velocity, max_velocity);

    let scale: f32 = max(max(velocity, max_velocity), 1e-10);

    convergence = min(convergence, dot(around / scale, nucleus / scale));
  }

  return select(0.0, saturate(1.0 - convergence) * saturate(max_velocity / 0.01),
    length(nucleus * fsr.render_size) > 0.01);
}

@fragment
fn fs_depth_clip(in: FullscreenVarying) -> DepthClipOutput {
  let position: vec2<f32> = floor(in.clip.xy);
  let motion: vec2<f32> = textureLoad(dilated_motion, clamped_texel(position, fsr.render_size), 0).xy;
  // Motion under a hundredth of a display pixel is taken for none.
  let moved: vec2<f32> = motion * f32(length(motion * fsr.display_size) > 0.01);
  let clip: f32 = depth_clip(position, moved, load_dilated_depth(position));

  // `EvaluateSurface`: a surface sloping away down the view counts for nothing.
  let d0: f32 = to_view_depth(load_reconstructed(position + vec2<f32>(0.0, -1.0)));
  let d1: f32 = to_view_depth(load_reconstructed(position));
  let d2: f32 = to_view_depth(load_reconstructed(position + vec2<f32>(0.0, 1.0)));
  let surface: f32 = 1.0 - f32(d0 - d1 > d1 * 0.01 && d1 - d2 > d2 * 0.01);

  // `ComputePreparedInputColor`: an exposure of one.
  let rgb: vec3<f32> = min(max(load_color(position), vec3<f32>(0.0)), vec3<f32>(FSR2_FP16_MAX));

  // `ComputeTemporalMotionDivergence`: how far this motion departs from the motion found where it came from.
  let reprojected: vec2<f32> = clamped_uv((position + 0.5) / fsr.render_size + motion, fsr.render_size);
  let previous_motion: vec2<f32> = textureSampleLevel(previous_dilated_motion, linear_sampler, reprojected, 0.0).xy;
  let distance: f32 = length(motion * fsr.display_size);
  let temporal_divergence: f32 = select(0.0,
    mix(0.0, saturate(1.0 - length(previous_motion) / max(length(motion), 1e-10)), saturate(pow(distance / 20.0, 3.0))),
    distance > 1.0);

  // `ComputeDepthDivergence`: how far the depths around spread, none where the sky shows.
  let farthest: f32 = to_view_depth(0.0);
  var depth_min: f32 = farthest;
  var depth_max: f32 = 0.0;
  var is_sky: bool = false;

  for (var index: i32 = 0; index < 9; index++) {
    let at: vec2<f32> = position + neighbour(index);
    let depth: f32 = to_view_depth(load_dilated_depth(at)) * f32(is_on_screen(at, fsr.render_size));

    is_sky = is_sky || depth == farthest;
    depth_min = min(depth_min, depth);
    depth_max = max(depth_max, depth);
  }

  let depth_divergence: f32 = select(1.0 - depth_min / max(depth_max, 1e-10), 0.0, is_sky);
  let accumulation_mask: f32 = max(saturate(temporal_divergence - depth_divergence), motion_divergence(position));

  // `PreProcessReactiveMasks`: the reactive mask dilated to the similar colours around, the more similar the more.
  let reference: vec3<f32> = load_color(position);
  var reactive: f32 = 0.0;

  for (var index: i32 = 0; index < 9; index++) {
    let at: vec2<f32> = position + neighbour(index);
    let around: vec3<f32> = load_color(at);
    let mask: f32 = textureLoad(reactive_mask, clamped_texel(at, fsr.render_size), 0).x;
    let similarity: f32 = dot(reference, around)
      / max(max(dot(reference, reference), dot(around, around)), 1e-10);

    reactive = max(reactive, pow(max(mask, 0.0), 1.0 + (6.0 - similarity * 6.0)));
  }

  var out: DepthClipOutput;

  out.prepared = vec4<f32>(to_ycocg(rgb), clip * surface);
  out.masks = vec4<f32>(reactive, accumulation_mask, 0.0, 1.0);

  return out;
}
