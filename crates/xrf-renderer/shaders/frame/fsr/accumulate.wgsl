#import "common/fullscreen"
#import "frame/fsr/common"

// `Accumulate` (`ffx_fsr2_accumulate.h`, with `ffx_fsr2_reproject.h`, `ffx_fsr2_upsample.h` and
// `ffx_fsr2_postprocess_lock_status.h`), at the display's size: the history, the lock status and the luma history.

// YCoCg, and the depth clip in alpha.
@group(0) @binding(1) var prepared: texture_2d<f32>;
// The reactive and accumulation masks.
@group(0) @binding(2) var reactive_masks: texture_2d<f32>;
@group(0) @binding(3) var dilated_motion: texture_2d<f32>;
// Whether each drawn texel is a thin feature to lock.
@group(0) @binding(4) var locks: texture_2d<f32>;
// The mean log luma a 32nd a side.
@group(0) @binding(5) var shading_luma: texture_2d<f32>;
// The frame before, resolved: colour, and the temporal reactivity, negative where it was in motion.
@group(0) @binding(6) var history: texture_2d<f32>;
// Each pixel's lock: its lifetime remaining, and the shading luma it was locked at.
@group(0) @binding(7) var lock_status: texture_2d<f32>;
// Each pixel's four last lumas.
@group(0) @binding(8) var luma_history: texture_2d<f32>;
@group(0) @binding(9) var linear_sampler: sampler;

struct AccumulateOutput {
  @location(0) history: vec4<f32>,
  @location(1) lock_status: vec4<f32>,
  @location(2) luma_history: vec4<f32>,
};

// `RectificationBox`.
struct Box {
  centre: vec3<f32>,
  spread: vec3<f32>,
  least: vec3<f32>,
  most: vec3<f32>,
};

// This frame's colour gathered about a display pixel, its weight among the frames, and the box around it.
struct Upsampled {
  color: vec3<f32>,
  weight: f32,
  box: Box,
};

fn load_clamped(source: texture_2d<f32>, at: vec2<f32>, size: vec2<f32>) -> vec4<f32> {
  return textureLoad(source, clamped_texel(at, size), 0);
}

// `DeclareCustomTextureSample(..., Lanczos2, FetchBicubicSamples)`: the history at a coordinate through a separable
// Lanczos-2 over the sixteen texels around it, deringed to the four nearest.
fn lanczos2_history(uv: vec2<f32>, size: vec2<f32>) -> vec4<f32> {
  let position: vec2<f32> = clamp(uv * size - 0.5, vec2<f32>(0.0), size);
  let base: vec2<f32> = floor(position);
  let fraction: vec2<f32> = position - base;
  var across: vec4<f32>;
  var down: vec4<f32>;

  for (var index: i32 = 0; index < 4; index++) {
    across[index] = lanczos2(f32(index - 1) - fraction.x);
    down[index] = lanczos2(f32(index - 1) - fraction.y);
  }

  let across_total: f32 = across.x + across.y + across.z + across.w;
  let down_total: f32 = down.x + down.y + down.z + down.w;
  var filtered: vec4<f32> = vec4<f32>(0.0);
  var least: vec4<f32> = vec4<f32>(3.402823466e38);
  var most: vec4<f32> = vec4<f32>(-3.402823466e38);

  for (var y: i32 = 0; y < 4; y++) {
    var row: vec4<f32> = vec4<f32>(0.0);

    for (var x: i32 = 0; x < 4; x++) {
      let sample: vec4<f32> = load_clamped(history, base + vec2<f32>(f32(x - 1), f32(y - 1)), size);

      row += sample * across[x];

      if ((x == 1 || x == 2) && (y == 1 || y == 2)) {
        least = min(least, sample);
        most = max(most, sample);
      }
    }

    filtered += row / across_total * down[y];
  }

  return clamp(filtered / down_total, least, most);
}

// `LoadRwNewLocks`, gathered: the lock the lock pass found for the one drawn sample landing in this display pixel, the
// texel `m` whose `floor((m + 0.5 - jitter) / render * display)` is this pixel.
fn new_lock(position: vec2<f32>) -> f32 {
  // The candidate at or just past this pixel's own edge, and the ones before it, lest rounding put it a texel over.
  let candidate: vec2<f32> = ceil(position * fsr.downscale - 0.5 + fsr.jitter);
  var lock: f32 = 0.0;

  for (var index: i32 = 0; index < 4; index++) {
    let at: vec2<f32> = candidate - footprint_corner(index);

    if (all(to_display_position(at) == position) && is_on_screen(at, fsr.render_size)) {
      lock = max(lock, load_clamped(locks, at, fsr.render_size).x);
    }
  }

  return lock;
}

// `ComputeUpsampledColorAndWeight`: the nine prepared texels nearest the pixel through FSR 1's Lanczos approximation,
// the kernel wider the less the history is to be trusted, deringed to their box, and the box itself, weighed by
// nearness sharper the faster the view moves.
fn upsampled(position: vec2<f32>, depth_clip: f32, is_new: bool, reactive: f32, velocity: f32) -> Upsampled {
  let output: vec2<f32> = (position + 0.5) * fsr.downscale;
  let input: vec2<f32> = floor(output);
  let unjittered: vec2<f32> = input + 0.5 - fsr.jitter;
  let is_flipped: vec2<bool> = unjittered > output;
  // `offsetTL`: two back where the sample stands past the pixel, else one.
  let top_left: vec2<f32> = select(vec2<f32>(-1.0), vec2<f32>(-2.0), is_flipped);
  let base_offset: vec2<f32> = unjittered - output;
  let kernel_reactive: f32 = max(reactive, f32(is_new));
  let max_kernel: f32 = min(1.99, 1.0 / fsr.downscale.x);
  let bias_max: f32 = max_kernel * (1.0 - kernel_reactive);
  let bias_min: f32 = max(1.0, (bias_max + 1.0) * 0.3);
  let bias_factor: f32 = max(0.0, max(depth_clip * 0.25, kernel_reactive));
  let kernel_bias: f32 = mix(bias_max, bias_min, bias_factor);
  let curve_bias: f32 = mix(-2.0, -3.0, saturate(velocity / 50.0));
  var color: vec3<f32> = vec3<f32>(0.0);
  var weight: f32 = 0.0;
  var centre: vec3<f32> = vec3<f32>(0.0);
  var moment: vec3<f32> = vec3<f32>(0.0);
  var box_weight: f32 = 0.0;
  var least: vec3<f32> = vec3<f32>(3.402823466e38);
  var most: vec3<f32> = vec3<f32>(-3.402823466e38);

  for (var row: i32 = 0; row < 3; row++) {
    for (var column: i32 = 0; column < 3; column++) {
      // Flipped, so the first three rows and columns are always the box's.
      let place: vec2<f32> = select(vec2<f32>(f32(column), f32(row)), vec2<f32>(f32(3 - column), f32(3 - row)),
        is_flipped);
      let at: vec2<f32> = input + top_left + place;
      let sample: vec3<f32> = load_clamped(prepared, at, fsr.render_size).xyz;
      let offset: vec2<f32> = base_offset + top_left + place;
      let biased: vec2<f32> = offset * kernel_bias;
      let tap: f32 = f32(is_on_screen(at, fsr.render_size)) * lanczos2_approx_sq(dot(biased, biased));
      let nearness: f32 = exp(curve_bias * dot(offset, offset));

      color += sample * tap;
      weight += tap;
      least = min(least, sample);
      most = max(most, sample);
      centre += sample * nearness;
      moment += sample * sample * nearness;
      box_weight += nearness;
    }
  }

  // `RectificationBoxComputeVarianceBoxData`.
  let normalizer: f32 = select(1.0, box_weight, abs(box_weight) > FSR2_EPSILON);
  let mean: vec3<f32> = centre / normalizer;
  let spread: vec3<f32> = sqrt(abs(moment / normalizer - mean * mean));
  let is_weighed: bool = weight > FSR2_EPSILON;

  // Normalized and deringed where anything was weighed, the weight scaled to a frame's share.
  return Upsampled(select(color, clamp(color / max(weight, FSR2_EPSILON), least, most), is_weighed),
    select(0.0, weight * UPSAMPLE_LANCZOS_WEIGHT_SCALE, is_weighed), Box(mean, spread, least, most));
}

@fragment
fn fs_accumulate(in: FullscreenVarying) -> AccumulateOutput {
  let position: vec2<f32> = floor(in.clip.xy);

  // `InitParams`.
  let uv: vec2<f32> = (position + 0.5) / fsr.display_size;
  let drawn_uv: vec2<f32> = clamped_uv(uv + fsr.jitter / fsr.render_size, fsr.render_size);
  let motion: vec2<f32> = load_clamped(dilated_motion, floor(uv * fsr.render_size), fsr.render_size).xy;
  let velocity: f32 = length(motion * fsr.display_size);
  let reprojected: vec2<f32> = uv + motion;
  let is_existing: bool = all(reprojected >= vec2<f32>(0.0)) && all(reprojected <= vec2<f32>(1.0));
  let depth_clip: f32 = saturate(textureSampleLevel(prepared, linear_sampler, drawn_uv, 0.0).w);
  let masks: vec4<f32> = textureSampleLevel(reactive_masks, linear_sampler, drawn_uv, 0.0);
  let dilated_reactive: f32 = masks.x;
  let accumulation_mask: f32 = masks.y;
  let is_reset: bool = fsr.frame_index < 0.5;
  let is_new: bool = !is_existing || is_reset;

  // `ReprojectHistoryColor` and `ReprojectHistoryLockStatus`.
  var previous_color: vec3<f32> = vec3<f32>(0.0);
  var status: vec2<f32> = vec2<f32>(0.0);
  var temporal_reactive: f32 = 0.0;
  var was_moving: bool = false;
  var is_new_lock: bool = false;

  if (is_existing && !is_reset) {
    let previous: vec4<f32> = lanczos2_history(reprojected, fsr.display_size);

    previous_color = to_ycocg(clamp(previous.rgb, vec3<f32>(0.0), vec3<f32>(FSR2_FP16_MAX)));
    temporal_reactive = saturate(abs(previous.w));
    was_moving = previous.w < 0.0;
    is_new_lock = new_lock(position) > 127.0 / 255.0;
    status = textureSampleLevel(lock_status, linear_sampler, reprojected, 0.0).xy;
  }

  var reactive: f32 = max(dilated_reactive, temporal_reactive);

  // `UpdateLockStatus`.
  let shading_uv: vec2<f32> = clamped_uv(uv, fsr.luma_mip_size);
  let shading: f32 = pow(exp(textureSampleLevel(shading_luma, linear_sampler, shading_uv, 0.0).x), 1.0 / 6.0);

  status.y = select(status.y, shading, status.y == 0.0);

  let luminance_diff: f32 = 1.0 - min_over_max(status.y, shading);

  if (is_new_lock) {
    status.y = shading;
    status.x = select(1.0, 2.0, status.x != 0.0);
  } else if (status.x <= 1.0) {
    status.y = mix(status.y, shading, 0.5);
  } else if (luminance_diff > 0.1) {
    status.x = 0.0;
  }

  reactive = max(reactive, saturate((luminance_diff - 0.1) * 10.0));
  status.x = status.x * (1.0 - reactive) * saturate(1.0 - accumulation_mask) * f32(depth_clip < 0.1);

  let lock_contribution: f32 = saturate(saturate(saturate(status.x - 1.0) * 4.0)
    * saturate(min_over_max(status.y, shading)));

  // `ComputeUpsampledColorAndWeight`.
  let gathered: Upsampled = upsampled(position, depth_clip, is_new, reactive, velocity);
  let box: Box = gathered.box;
  let weight: f32 = gathered.weight;

  // `ComputeLumaInstabilityFactor`.
  let frame_luma: f32 = round(box.centre.x * 255.0) / 255.0;
  let is_luma_sampled: bool = max(max(depth_clip, accumulation_mask), luminance_diff) < 0.1 && !is_new;
  let lumas: vec4<f32> = select(vec4<f32>(0.0), textureSampleLevel(luma_history, linear_sampler, reprojected, 0.0),
    is_luma_sampled);
  let first_diff: f32 = frame_luma - lumas.x;
  var smallest: f32 = abs(first_diff);

  for (var index: i32 = 1; index < 4; index++) {
    let diff: f32 = frame_luma - lumas[index];

    if (sign(first_diff) == sign(diff)) {
      smallest = min(smallest, abs(diff));
    }
  }

  let box_size_factor: f32 = pow(saturate(box.spread.x / 0.1), 6.0);
  let is_unstable: f32 = f32(f32(smallest != abs(first_diff)) * box_size_factor > 1.0 / 255.0)
    * (1.0 - max(accumulation_mask, pow(max(reactive, 0.0), 1.0 / 6.0)));
  // Nothing yet, until four lumas stand in the history.
  let instability: f32 = select(0.0, is_unstable, abs(first_diff) >= 1.0 / 255.0) * f32(lumas.z != 0.0);

  // `ComputeBaseAccumulationWeight`.
  var accumulation: f32 = MAX_ACCUMULATION_LANCZOS_WEIGHT * f32(is_existing) * (1.0 - reactive) * (1.0 - depth_clip);

  accumulation = min(accumulation, mix(accumulation, weight * 10.0, max(f32(was_moving), saturate(velocity * 10.0))));
  accumulation = min(accumulation, mix(accumulation, weight, saturate(velocity / 20.0)));

  var resolved: vec3<f32>;

  if (is_new) {
    resolved = to_rgb(gathered.color);
  } else {
    // `RectifyHistory`.
    let influence: f32 = min(20.0, pow(1.0 / (fsr.downscale.x * fsr.downscale.y), 3.0));
    let box_scale_t: f32 = max(depth_clip, max(accumulation_mask, saturate(velocity / 20.0)));
    let scaled_spread: vec3<f32> = box.spread * mix(influence, 1.0, box_scale_t);
    let box_min: vec3<f32> = max(box.least, box.centre - scaled_spread);
    let box_max: vec3<f32> = min(box.most, box.centre + scaled_spread);

    if (any(box_min > previous_color) || any(previous_color > box_max)) {
      let contribution: f32 = saturate(max(instability, lock_contribution) * (1.0 - sqrt(max(dilated_reactive, 0.0))));

      previous_color = mix(clamp(previous_color, box_min, box_max), previous_color, contribution);
      accumulation = mix(min(accumulation, 0.1), accumulation, contribution);
    }

    // `Accumulate`, for colour in the display's range: no tonemap around the blend.
    let total: f32 = max(FSR2_EPSILON, accumulation + weight);

    resolved = to_rgb(mix(previous_color, gathered.color, weight / total));
  }

  // `FinalizeLockStatus`: a lock whose surface leaves the screen next frame dies; else it wears down by this frame's
  // weight.
  let next: vec2<f32> = uv - motion;
  let is_staying: bool = all(next >= vec2<f32>(0.0)) && all(next <= vec2<f32>(1.0));
  let decrease: f32 = weight / (fsr.jitter_phase_count * AVERAGE_LANCZOS_WEIGHT_PER_FRAME);
  let lifetime: f32 = select(0.0, max(0.0, status.x - decrease), is_staying);

  // `ComputeTemporalReactiveFactor`: negative where the pixel moved enough to be counted as in motion.
  var settled: f32 = min(0.99, reactive);

  settled = max(settled, mix(settled, 0.4, saturate(velocity)));
  settled = max(settled * settled, max(depth_clip * 0.1, dilated_reactive));
  settled = select(settled, 1.0, is_new);

  let next_reactive: f32 = select(settled, -max(FSR2_EPSILON, settled), saturate(velocity * 10.0) >= 1.0);
  var out: AccumulateOutput;

  out.history = vec4<f32>(resolved, next_reactive);
  out.lock_status = vec4<f32>(lifetime, status.y, 0.0, 1.0);
  out.luma_history = vec4<f32>(frame_luma, lumas.x, lumas.y, lumas.z);

  return out;
}
