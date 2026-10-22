#import "common/luminance"
#import "common/lighting"

// The physical light's exposure: the scene's light counted into a histogram of its log2 luminance, a texel in every
// four each way, the centre weighed up to four times the edges and the sky not at all, then the average between the
// darkest and the brightest shares it leaves out (`LOW_SHARE`, `HIGH_SHARE`) brought to where a daylit exterior's is
// shown, less the share of its distance from one it keeps, never under the dark's limit, the scale moving towards it at
// most so many stops a second, faster darkening than brightening. Beside it the sky's own level, its pictures' log2
// average as the game paints them, which the pictures are shown by (`PictureShares`). At the authored levels the scale
// is the engine's own instead (`ExposureHistogramUniform::is_engine_metered`): the frame read as the engine's meter
// reads its high part, the sky with the land, and the engine's step taken on the scale the engine would show it at.

#import "generated/frame/exposure_histogram"

// Bins the luminance is counted in (`ExposureHistogramUniform::BINS`).
const BINS: u32 = 128u;
// Texels each way a sample stands for.
const SAMPLE_STRIDE: u32 = 4u;
// Invocations a counting workgroup runs each way (`ExposurePass::HISTOGRAM_WORKGROUP`).
const HISTOGRAM_WORKGROUP: u32 = 16u;
// Where the bins hold the sky's sum of log2 levels, its count and its sum of squared levels, after the histogram's
// (`ExposureHistogramUniform::SKY_SLOTS`); the sum in steps of a sixteenth of a stop over the least an eight-bit picture
// paints above black, `(1/255)^2.2`, so it adds as whole numbers and a black texel counts as that least, not as nothing,
// the squares in whole stops.
const SKY_SUM: u32 = 128u;
const SKY_COUNT: u32 = 129u;
const SKY_SQUARES: u32 = 130u;
// The share of the frame under its bright end (`ExposureHistogramUniform::BRIGHT_SHARE`): the brightest few of what is
// lit, which a night's moonlit metal is, over the window's top in the dark land; and how many deviations over its mean
// the sky's bright end stands, as a normal spread's percentile at that share does.
const BRIGHT_SHARE: f32 = 0.99;
const SKY_BRIGHT_DEVIATIONS: f32 = 2.33;
const SKY_STEPS: f32 = 16.0;
const SKY_FLOOR_LOG: f32 = -17.59;
// Where the bins hold the engine meter's sum and its count, after the sky's (`ExposureHistogramUniform::ENGINE_SLOTS`);
// the sum in steps of a sixty-fourth, each workgroup's added once, so a frame of eight thousand texels across stays
// within the slot.
const ENGINE_SUM: u32 = 131u;
const ENGINE_COUNT: u32 = 132u;
const ENGINE_STEPS: f32 = 64.0;
// The bounds the engine holds its scale within (`cs_adapt`).
const ENGINE_LEAST_SCALE: f32 = 1.0 / 128.0;
const ENGINE_MOST_SCALE: f32 = 20.0;
// Invocations in a counting workgroup.
const HISTOGRAM_INVOCATIONS: u32 = HISTOGRAM_WORKGROUP * HISTOGRAM_WORKGROUP;

var<workgroup> local_bins: array<atomic<u32>, 128>;
var<workgroup> counts: array<u32, 128>;
var<workgroup> local_sky: array<atomic<u32>, 3>;
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

  if (index < 3u) {
    atomicStore(&local_sky[index], 0u);
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
    // The exposure is the level's alone: the game's sky and sun are pictures with no level of their own against the
    // ground's, so seeing them never darkens the ground under them, and a frame of sky alone holds the exposure it had.
    let is_sky: bool = textureLoad(depth_target, min(texel, textureDimensions(depth_target) - 1u), 0) <= 0.0;
    let weight: u32 = select(1u + u32(3.0 * saturate(1.0 - off_centre)), 0u, is_sky);

    atomicAdd(&local_bins[bin], weight);

    // The sky as the game paints it: the scene by the scale its pictures were drawn at this frame.
    if (is_sky) {
      let painted: f32 = held / max(picture_scale(lighting, state.adapted), 1e-12);
      let level: f32 = clamp(log2(max(painted, 1e-12)) - SKY_FLOOR_LOG, 0.0, -2.0 * SKY_FLOOR_LOG);

      atomicAdd(&local_sky[0], u32(round(level * SKY_STEPS)));
      atomicAdd(&local_sky[1], 1u);
      atomicAdd(&local_sky[2], u32(round(level * level)));
    }

    // The engine's meter reads the whole frame alike, the sky with the land.
    if (is_engine_metered) {
      local_engine[index] = engine_metered_scene(colour, is_sky);
      atomicAdd(&local_engine_count, 1u);
    }
  }

  workgroupBarrier();

  if (index < BINS) {
    atomicAdd(&bins[index], atomicLoad(&local_bins[index]));
  }

  if (index < 3u) {
    atomicAdd(&bins[SKY_SUM + index], atomicLoad(&local_sky[index]));
  }

  if (is_engine_metered && index == 0u) {
    var sum: f32 = 0.0;

    for (var invocation: u32 = 0u; invocation < HISTOGRAM_INVOCATIONS; invocation++) {
      sum += local_engine[invocation];
    }

    atomicAdd(&bins[ENGINE_SUM], u32(round(sum * ENGINE_STEPS)));
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

  let sky_sum: u32 = atomicLoad(&bins[SKY_SUM]);
  let sky_count: u32 = atomicLoad(&bins[SKY_COUNT]);
  let sky_squares: u32 = atomicLoad(&bins[SKY_SQUARES]);

  let engine_sum: u32 = atomicLoad(&bins[ENGINE_SUM]);
  let engine_count: u32 = atomicLoad(&bins[ENGINE_COUNT]);

  atomicStore(&bins[SKY_SUM], 0u);
  atomicStore(&bins[SKY_COUNT], 0u);
  atomicStore(&bins[SKY_SQUARES], 0u);
  atomicStore(&bins[ENGINE_SUM], 0u);
  atomicStore(&bins[ENGINE_COUNT], 0u);

  // A frame with no sky in sight keeps the levels it last saw.
  if (sky_count > 0u) {
    let mean: f32 = f32(sky_sum) / (f32(sky_count) * SKY_STEPS);
    let spread: f32 = sqrt(max(f32(sky_squares) / f32(sky_count) - mean * mean, 0.0));

    state.sky = mean + SKY_FLOOR_LOG;
    state.sky_bright = mean + SKY_BRIGHT_DEVIATIONS * spread + SKY_FLOOR_LOG;
  }

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
