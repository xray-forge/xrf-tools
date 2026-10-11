#import "common/fullscreen"
#import "frame/fsr/common"

// `ffx_fsr3upscaler_accumulate.h`, with `ffx_fsr3upscaler_reproject.h` and `ffx_fsr3upscaler_upsample.h`, at the
// display's size: the history, its colour and each pixel's lock in alpha.

@group(0) @binding(1) var color: texture_2d<f32>;
@group(0) @binding(2) var dilated_motion: texture_2d<f32>;
// The reactive, disocclusion, shading change and accumulation masks.
@group(0) @binding(3) var masks: texture_2d<f32>;
@group(0) @binding(4) var luma_instability: texture_2d<f32>;
// The farthest depth in metres, at half the drawn size.
@group(0) @binding(5) var farthest_depth: texture_2d<f32>;
// How strongly each drawn texel is a thin feature to lock.
@group(0) @binding(6) var new_locks: texture_2d<f32>;
// The frame before's history: colour, and each pixel's lock.
@group(0) @binding(7) var history: texture_2d<f32>;
@group(0) @binding(8) var linear_sampler: sampler;

// `fRectificationCurveBias`: how sharply the box's weights fall off with distance.
const RECTIFICATION_CURVE_BIAS: f32 = -2.3;

// `RectificationBox`, its spread the deviation once computed.
struct Box {
  centre: vec3<f32>,
  spread: vec3<f32>,
  least: vec3<f32>,
  most: vec3<f32>,
};

// This frame's colour gathered about a display pixel, its weight among the frames, the box around it, and the weight
// the history keeps, which an initial sample takes away.
struct Upsampled {
  color: vec3<f32>,
  weight: f32,
  box: Box,
  history_weight: f32,
};

fn load_clamped(source: texture_2d<f32>, at: vec2<f32>, size: vec2<f32>) -> vec4<f32> {
  return textureLoad(source, clamped_texel(at, size), 0);
}

// `DeclareCustomTextureSample(..., Lanczos2, FetchBicubicSamples)`: the history at a coordinate through a separable
// Lanczos-2 over the sixteen texels around it, deringed to the four nearest.
fn lanczos2_history(uv: vec2<f32>, size: vec2<f32>) -> vec4<f32> {
  let position: vec2<f32> = uv * size - 0.5;
  let fraction: vec2<f32> = fract(position);
  let base: vec2<f32> = floor(clamp(position, vec2<f32>(0.0), size - 1.0));
  var across: vec4<f32>;
  var down: vec4<f32>;

  for (var index: i32 = 0; index < 4; index++) {
    across[index] = lanczos2(f32(index - 1) - fraction.x);
    down[index] = lanczos2(f32(index - 1) - fraction.y);
  }

  let across_total: f32 = across.x + across.y + across.z + across.w;
  let down_total: f32 = down.x + down.y + down.z + down.w;
  var filtered: vec4<f32> = vec4<f32>(0.0);
  var least: vec4<f32> = vec4<f32>(FSR_FP32_MAX);
  var most: vec4<f32> = vec4<f32>(-FSR_FP32_MAX);

  for (var y: i32 = 0; y < 4; y++) {
    var row: vec4<f32> = vec4<f32>(0.0);

    for (var x: i32 = 0; x < 4; x++) {
      // `ClampCoord`: the taps kept a texel inside the border.
      let tap: vec2<f32> = clamp(base + vec2<f32>(f32(x - 1), f32(y - 1)), vec2<f32>(1.0), size - 2.0);
      let sample: vec4<f32> = textureLoad(history, vec2<i32>(tap), 0);

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

// `LoadRwNewLocks`, gathered: the lock the reactivity pass found for the one drawn sample landing in this display pixel,
// the texel `m` whose `floor((m + 0.5 - jitter) / render * display)` is this pixel.
fn new_lock(position: vec2<f32>) -> f32 {
  // The candidate at or just past this pixel's own edge, and the ones before it, lest rounding put it a texel over.
  let candidate: vec2<f32> = ceil(position * fsr.downscale - 0.5 + fsr.jitter);
  var lock: f32 = 0.0;

  for (var index: i32 = 0; index < 4; index++) {
    let at: vec2<f32> = candidate - footprint_corner(index);

    if (all(to_display_position(at) == position) && is_on_screen(at, fsr.render_size)) {
      lock = max(lock, load_clamped(new_locks, at, fsr.render_size).x);
    }
  }

  return lock;
}

// `ComputeUpsampledColorAndWeight`: the nine texels nearest the pixel through FSR 1's Lanczos approximation, the kernel
// narrower the less the history is trusted, deringed to their box, and the box itself, weighed by nearness. Where
// nothing is accumulated yet (`bIsInitialSample`) the box's centre stands for the colour, and the history counts for
// nothing.
fn upsampled(position: vec2<f32>, reactive_masks: vec4<f32>, history_weight: f32) -> Upsampled {
  let output: vec2<f32> = (position + 0.5) * fsr.downscale;
  let input: vec2<f32> = floor(output);
  let unjittered: vec2<f32> = input + 0.5 - fsr.jitter;
  let is_flipped: vec2<bool> = unjittered > output;
  // `offsetTL`: two back where the sample stands past the pixel, else one.
  let top_left: vec2<f32> = select(vec2<f32>(-1.0), vec2<f32>(-2.0), is_flipped);
  let base_offset: vec2<f32> = unjittered - output;
  let is_initial: bool = reactive_masks[MASK_ACCUMULATION] == 0.0;
  let bias_max: f32 = min(1.99, 1.0 / fsr.downscale.x);
  let bias_min: f32 = max(1.0, (1.0 + bias_max) * 0.3);
  let bias_weight: f32 = min(1.0 - reactive_masks[MASK_DISOCCLUSION] * 0.5,
    min(1.0 - reactive_masks[MASK_SHADING_CHANGE], saturate(history_weight * 5.0)));
  let kernel_bias: f32 = mix(bias_min, bias_max, bias_weight);
  var weighted: vec3<f32> = vec3<f32>(0.0);
  var weight: f32 = 0.0;
  var centre: vec3<f32> = vec3<f32>(0.0);
  var moment: vec3<f32> = vec3<f32>(0.0);
  var box_weight: f32 = 0.0;
  var least: vec3<f32> = vec3<f32>(FSR_FP32_MAX);
  var most: vec3<f32> = vec3<f32>(-FSR_FP32_MAX);

  for (var row: i32 = 0; row < 3; row++) {
    for (var column: i32 = 0; column < 3; column++) {
      // Flipped, so the first three rows and columns are always the box's.
      let place: vec2<f32> = select(vec2<f32>(f32(column), f32(row)), vec2<f32>(f32(3 - column), f32(3 - row)),
        is_flipped);
      let at: vec2<f32> = input + top_left + place;
      // `LoadPreparedColor`: linear colour, kept off negative, in YCoCg; tonemapped for an initial sample of linear
      // light.
      let rgb: vec3<f32> = max(load_clamped(color, at, fsr.render_size).rgb, vec3<f32>(0.0));
      let sample: vec3<f32> = to_ycocg(select(rgb, tonemapped(rgb), is_initial && is_hdr()));
      let offset: vec2<f32> = base_offset + top_left + place;
      let on_screen: f32 = f32(is_on_screen(at, fsr.render_size));

      if (!is_initial) {
        let biased: vec2<f32> = offset * kernel_bias;
        let tap: f32 = on_screen * lanczos2_approx_sq(dot(biased, biased));

        weighted += sample * tap;
        weight += tap;
      }

      let nearness: f32 = exp(RECTIFICATION_CURVE_BIAS * dot(offset, offset)) * on_screen;

      least = min(least, sample);
      most = max(most, sample);
      centre += sample * nearness;
      moment += sample * sample * nearness;
      box_weight += nearness;
    }
  }

  // `RectificationBoxComputeVarianceBoxData`.
  let normalizer: f32 = select(1.0, box_weight, abs(box_weight) > FSR_FP32_MIN);
  let mean: vec3<f32> = centre / normalizer;
  let box: Box = Box(mean, sqrt(abs(moment / normalizer - mean * mean)), least, most);

  if (is_initial) {
    let initial: vec3<f32> = select(mean, to_ycocg(inverse_tonemapped(to_rgb(mean))), is_hdr());

    return Upsampled(initial, 1.0, box, 0.0);
  }

  // Normalized and deringed where anything was weighed, the weight scaled to a frame's share.
  if (weight > FSR_EPSILON) {
    return Upsampled(clamp(weighted / weight, least, most), weight * AVERAGE_LANCZOS_WEIGHT_PER_FRAME, box,
      history_weight);
  }

  return Upsampled(weighted, 0.0, box, history_weight);
}

// A pixel's accumulation: the history it leaves, its colour and its lock, and how much of it this frame gave.
struct Accumulated {
  history: vec4<f32>,
  frame_share: f32,
};

fn accumulate(position: vec2<f32>) -> Accumulated {
  // `InitPassData`.
  let uv: vec2<f32> = (position + 0.5) / fsr.display_size;
  let drawn_uv: vec2<f32> = uv + fsr.jitter / fsr.render_size;
  let motion: vec2<f32> = load_clamped(dilated_motion, trunc(uv * fsr.render_size), fsr.render_size).xy;
  let velocity: f32 = to_4k_velocity(motion);
  let reprojected: vec2<f32> = uv + motion;
  let is_existing: bool = is_uv_inside(reprojected);
  let is_new: bool = !is_existing || fsr.frame_index < 0.5;
  let instability: f32 = textureSampleLevel(luma_instability, linear_sampler, clamped_uv(uv, fsr.render_size), 0.0).x;
  let farthest_in_meters: f32 = textureSampleLevel(farthest_depth, linear_sampler,
    clamped_uv(drawn_uv, fsr.shading_size), 0.0).x;
  var reactive_masks: vec4<f32> = saturate(textureSampleLevel(masks, linear_sampler,
    clamped_uv(drawn_uv, fsr.render_size), 0.0));

  reactive_masks[MASK_ACCUMULATION] *= f32(round(reactive_masks[MASK_ACCUMULATION] * 100.0) > 1.0);

  let reactive: f32 = reactive_masks[MASK_REACTIVE];
  let disoccluded: f32 = reactive_masks[MASK_DISOCCLUSION];
  let changed: f32 = reactive_masks[MASK_SHADING_CHANGE];
  let accumulation: f32 = reactive_masks[MASK_ACCUMULATION];

  // `ReprojectHistoryColor`.
  var previous_color: vec3<f32> = vec3<f32>(0.0);
  var lock: f32 = 0.0;

  if (!is_new) {
    let previous: vec4<f32> = lanczos2_history(reprojected, fsr.display_size);

    previous_color = to_ycocg(previous.rgb);
    lock = previous.w;
  }

  // `UpdateLockStatus`: a lock wears away where the surface changes, takes in a new thin feature, and dies where the
  // surface leaves the screen next frame.
  let lifetime_cut: f32 = max(changed, max(reactive, disoccluded));

  lock = max(0.0, lock - lifetime_cut * LOCK_MAX);

  let lock_contribution: f32 = saturate(saturate(lock - LOCK_THRESHOLD) * (LOCK_MAX - LOCK_THRESHOLD));

  lock = clamp(lock + new_lock(position) * (1.0 - reactive), 0.0, LOCK_MAX);
  lock = max(0.0, lock - 0.1 / fsr.jitter_phase_count * (1.0 - lifetime_cut));
  lock *= f32(is_uv_inside(uv - motion));

  // `ComputeBaseAccumulationWeight`: the history trusted less the faster the view moves.
  let base_weight: f32 = min(accumulation, mix(accumulation, 0.15,
    saturate(max(0.0, velocity * fsr.velocity_factor) / 0.5)));
  let gathered: Upsampled = upsampled(position, reactive_masks, base_weight);
  let box: Box = gathered.box;

  // `RectifyHistory`: the history clamped into the box's ellipsoid, wider the more it is to be trusted, but kept where
  // a lock or a flicker holds it.
  let box_scale_t: f32 = max(saturate(velocity / 20.0), max(saturate(0.75 - farthest_in_meters / 20.0),
    max(1.0 - accumulation, max(sqrt(reactive), changed))));
  let scaled_spread: vec3<f32> = box.spread * vec3<f32>(1.7, 1.0, 1.0) * mix(3.0, 1.0, box_scale_t);
  let transformed: vec3<f32> = (previous_color - box.centre) / max(scaled_spread, vec3<f32>(1.193e-7));

  if (length(transformed) > 1.0) {
    let clamped: vec3<f32> = normalize(transformed) * scaled_spread + box.centre;
    let contribution: f32 = max(instability, lock_contribution) * accumulation * (1.0 - disoccluded);

    previous_color = mix(clamped, previous_color, saturate(contribution));
  }

  // `Accumulate`: over linear light, both blended through the reversible tonemap and the result taken back out of it.
  let history_weight: f32 = max(FSR_EPSILON,
    gathered.history_weight * f32(gathered.history_weight > FSR_FP16_MIN) + gathered.weight);
  let blend: f32 = saturate(gathered.weight / history_weight);
  var resolved: vec3<f32>;

  if (is_hdr()) {
    let blended: vec3<f32> = mix(to_ycocg(tonemapped(to_rgb(previous_color))),
      to_ycocg(tonemapped(to_rgb(gathered.color))), blend);

    resolved = inverse_tonemapped(to_rgb(blended));
  } else {
    resolved = to_rgb(mix(previous_color, gathered.color, blend));
  }

  return Accumulated(vec4<f32>(max(resolved, vec3<f32>(0.0)), lock), blend);
}

@fragment
fn fs_accumulate(in: FullscreenVarying) -> @location(0) vec4<f32> {
  return accumulate(floor(in.clip.xy)).history;
}

// The FSR frame share view: the same accumulation, showing how much of each pixel this frame gave, as its square root
// so a frame's usual twentieth reads as a fifth.
@fragment
fn fs_frame_share(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let share: f32 = sqrt(accumulate(floor(in.clip.xy)).frame_share);

  return vec4<f32>(share, share, share, 1.0);
}
