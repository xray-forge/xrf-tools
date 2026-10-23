#import "common/luminance"
#import "common/lighting"
#import "common/display_transform"

// The physical light's exposure: the scene's light counted into a histogram of its log2 luminance, a texel in every
// four each way, the centre weighed up to four times the edges and the sky not at all, then the average between the
// darkest and the brightest shares it leaves out (`LOW_SHARE`, `HIGH_SHARE`) brought to where a daylit exterior's is
// shown, less the share of its distance from one it keeps, never under the dark's limit, the scale moving towards it at
// most so many stops a second, faster darkening than brightening. At the authored levels the scale is the engine's own
// instead (`ExposureHistogramUniform::is_engine_metered`): the frame read as the engine's meter reads its high
// part, the sky with the land, and the engine's step taken on the scale the engine would show it at.

#import "generated/frame/exposure_histogram"

// Bins the luminance is counted in (`ExposureHistogramUniform::BINS`).
const BINS: u32 = 128u;
// Texels each way a sample stands for.
const SAMPLE_STRIDE: u32 = 4u;
// Invocations a counting workgroup runs each way (`ExposurePass::HISTOGRAM_WORKGROUP`).
const HISTOGRAM_WORKGROUP: u32 = 16u;
// The share of the frame under its bright end (`ExposureHistogramUniform::BRIGHT_SHARE`): the brightest few of what is
// lit, which a night's moonlit metal is, over the window's top in the dark land.
const BRIGHT_SHARE: f32 = 0.99;
// Where the bins hold the engine meter's sum and its count, after the histogram's
// (`ExposureHistogramUniform::ENGINE_SLOTS`); the sum in steps of a sixty-fourth, each workgroup's added once, so a
// frame of eight thousand texels across stays within the slot.
const ENGINE_SUM: u32 = 128u;
const ENGINE_COUNT: u32 = 129u;
const ENGINE_STEPS: f32 = 64.0;
// The bounds the engine holds its scale within (`cs_adapt`).
const ENGINE_LEAST_SCALE: f32 = 1.0 / 128.0;
const ENGINE_MOST_SCALE: f32 = 20.0;
// Invocations in a counting workgroup.
const HISTOGRAM_INVOCATIONS: u32 = HISTOGRAM_WORKGROUP * HISTOGRAM_WORKGROUP;

var<workgroup> local_bins: array<atomic<u32>, 128>;
var<workgroup> counts: array<u32, 128>;
var<workgroup> local_engine: array<f32, 256>;
var<workgroup> local_engine_count: atomic<u32>;

// The engine's step (`cs_adapt`) at the authored levels, from the mean the engine's meter reads, `luminance`: taken on
// the scale the engine would show this frame at, the authored scale times the light's unit, the compensation taken off,
// encoded as the display encodes the light, then brought back.
fn engine_scale(luminance: f32) -> f32 {
  let unit: f32 = light_unit(lighting) / params.compensation_scale;
  let current: f32 = pow(max(unit * state.adapted, 1e-12), 1.0 / SRGB_DISPLAY_POWER);
  let wanted: f32 = params.engine_target / (luminance * params.engine_weight + params.engine_floor);
  let blend: f32 = select(params.engine_blend, 1.0, params.snap > 0.5);
  let next: f32 = clamp(mix(current, wanted, blend), ENGINE_LEAST_SCALE, ENGINE_MOST_SCALE);

  return pow(next, SRGB_DISPLAY_POWER) / unit;
}

@compute @workgroup_size(16, 16)
fn cs_bin(@builtin(global_invocation_id) id: vec3<u32>, @builtin(local_invocation_index) index: u32) {
  if (index < BINS) {
    atomicStore(&local_bins[index], 0u);
  }

  if (index == 0u) {
    atomicStore(&local_engine_count, 0u);
  }

  local_engine[index] = 0.0;
  workgroupBarrier();

  let size: vec2<u32> = textureDimensions(scene);
  let texel: vec2<u32> = id.xy * SAMPLE_STRIDE;
  let is_engine_metered: bool = params.is_engine_metered > 0.5;

  if (all(texel < size)) {
    // The scene holds light at the scale this frame was drawn at.
    let colour: vec3<f32> = textureLoad(scene, texel, 0).rgb;
    let held: f32 = luminance(colour);
    let light: f32 = held / max(state.adapted, 1e-6);
    let at: f32 = saturate((log2(max(light, 1e-9)) - params.least_log) / params.span_log);
    let bin: u32 = min(u32(at * f32(BINS)), BINS - 1u);
    let off_centre: f32 = length((vec2<f32>(texel) + 0.5) / vec2<f32>(size) - 0.5) * 2.0;
    // The land's histogram (the physical levels' law, the key and the bright end) is the level's alone: the game's sky
    // and sun are pictures with no level of their own against the ground's, so under the law seeing them never darkens
    // the ground under them, and a frame of sky alone holds the exposure it had.
    let is_sky: bool = textureLoad(depth_target, min(texel, textureDimensions(depth_target) - 1u), 0) <= 0.0;
    let weight: u32 = select(1u + u32(3.0 * saturate(1.0 - off_centre)), 0u, is_sky);

    atomicAdd(&local_bins[bin], weight);

    // The engine's meter reads the whole frame alike, the sky with the land: the land's exposed colour, the sky as the
    // display shows it, which the engine's high part holds.
    if (is_engine_metered) {
      var shown: vec3<f32> = srgb_encoded(colour);

      if (is_sky) {
        shown = shown_for_display(colour, lighting.display);
      }

      local_engine[index] = engine_metered_shown(shown);
      atomicAdd(&local_engine_count, 1u);
    }
  }

  workgroupBarrier();

  if (index < BINS) {
    atomicAdd(&bins[index], atomicLoad(&local_bins[index]));
  }

  if (!is_engine_metered) {
    return;
  }

  // The workgroup's readings summed by halves before the sum is rounded to its steps, so a dark texel's reading of a
  // fraction of a step still counts.
  for (var stride: u32 = HISTOGRAM_INVOCATIONS / 2u; stride > 0u; stride /= 2u) {
    if (index < stride) {
      local_engine[index] += local_engine[index + stride];
    }

    workgroupBarrier();
  }

  if (index == 0u) {
    atomicAdd(&bins[ENGINE_SUM], u32(round(local_engine[0] * ENGINE_STEPS)));
    atomicAdd(&bins[ENGINE_COUNT], atomicLoad(&local_engine_count));
  }
}

@compute @workgroup_size(128)
fn cs_settle(@builtin(local_invocation_index) index: u32) {
  counts[index] = atomicLoad(&bins[index]);
  // Emptied for the next frame's count, as each frame meters afresh.
  atomicStore(&bins[index], 0u);
  workgroupBarrier();

  if (index != 0u) {
    return;
  }

  let engine_sum: u32 = atomicLoad(&bins[ENGINE_SUM]);
  let engine_count: u32 = atomicLoad(&bins[ENGINE_COUNT]);

  atomicStore(&bins[ENGINE_SUM], 0u);
  atomicStore(&bins[ENGINE_COUNT], 0u);

  var total: f32 = 0.0;

  for (var bin: u32 = 0u; bin < BINS; bin++) {
    total += f32(counts[bin]);
  }

  // The scale this frame's tonemap used, which a frame of sky alone, holding its exposure, keeps too.
  state.used = state.adapted;

  // At the authored levels the scale is the engine's, which a frame of sky alone moves too, as the engine's does.
  let is_engine_metered: bool = params.is_engine_metered > 0.5;

  if (is_engine_metered && engine_count > 0u) {
    state.adapted = engine_scale(f32(engine_sum) / (f32(engine_count) * ENGINE_STEPS));
  }

  if (total <= 0.0) {
    return;
  }

  let low: f32 = total * params.low_share;
  let high: f32 = total * params.high_share;
  var below: f32 = 0.0;
  var weight: f32 = 0.0;
  var sum: f32 = 0.0;
  let brightest: f32 = total * BRIGHT_SHARE;
  var bright: f32 = params.least_log;

  // Each bin's count within the window, at the log2 luminance of its middle; and the bin the bright end falls in.
  for (var bin: u32 = 0u; bin < BINS; bin++) {
    let count: f32 = f32(counts[bin]);
    let windowed: f32 = max(min(below + count, high) - max(below, low), 0.0);
    let middle: f32 = params.least_log + (f32(bin) + 0.5) / f32(BINS) * params.span_log;

    // Placed within its bin by how far into it the bright end's share falls, so it moves smoothly as the frame does.
    if (below < brightest && below + count >= brightest) {
      bright = params.least_log + (f32(bin) + (brightest - below) / max(count, 1.0)) / f32(BINS) * params.span_log;
    }

    below += count;
    weight += windowed;
    sum += windowed * middle;
  }

  let average: f32 = select(log2(params.aim), sum / max(weight, 1e-6), weight > 0.0);

  // The key and the bright end as the engine's scale now exposes them.
  if (is_engine_metered) {
    let scale: f32 = log2(max(state.adapted, 1e-12));

    state.key = average + scale;
    state.bright = bright + scale;

    return;
  }

  let shown: f32 = max(log2(params.aim) + params.kept_share * (average - params.reference_log), params.darkest_log);
  let wanted: f32 = clamp(shown - average, params.least_scale_log, params.most_scale_log);
  let current: f32 = log2(max(state.adapted, 1e-6));
  let reach: f32 = select(params.brightening, params.darkening, wanted < current) * params.delta;
  let next: f32 = select(current + clamp(wanted - current, -reach, reach), wanted, params.snap > 0.5);

  state.adapted = exp2(next);
  // The key and the bright end as the scale now exposes them.
  state.key = average + next;
  state.bright = bright + next;
}
