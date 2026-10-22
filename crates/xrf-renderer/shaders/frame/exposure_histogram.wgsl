#import "common/luminance"
#import "common/lighting"

// The physical light's exposure: the scene's light counted into a histogram of its log2 luminance, a texel in every
// four each way, the centre weighed up to four times the edges and the sky not at all, then the average between the
// darkest and the brightest shares it leaves out (`LOW_SHARE`, `HIGH_SHARE`) brought to where a daylit exterior's is
// shown, less the share of its distance from one it keeps, never under the dark's limit, the scale moving towards it at
// most so many stops a second, faster darkening than brightening. Beside it the sky's own level, its pictures' log2
// average as the game paints them, which the pictures are shown by (`PictureShares`).

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

var<workgroup> local_bins: array<atomic<u32>, 128>;
var<workgroup> counts: array<u32, 128>;
var<workgroup> local_sky: array<atomic<u32>, 3>;

@compute @workgroup_size(16, 16)
fn cs_bin(@builtin(global_invocation_id) id: vec3<u32>, @builtin(local_invocation_index) index: u32) {
  if (index < BINS) {
    atomicStore(&local_bins[index], 0u);
  }

  if (index < 3u) {
    atomicStore(&local_sky[index], 0u);
  }

  workgroupBarrier();

  let size: vec2<u32> = textureDimensions(scene);
  let texel: vec2<u32> = id.xy * SAMPLE_STRIDE;

  if (all(texel < size)) {
    // The scene holds light at the scale this frame was drawn at.
    let light: f32 = luminance(textureLoad(scene, texel, 0).rgb) / max(state.adapted, 1e-6);
    let at: f32 = saturate((log2(max(light, 1e-9)) - params.least_log) / params.span_log);
    let bin: u32 = min(u32(at * f32(BINS)), BINS - 1u);
    let off_centre: f32 = length((vec2<f32>(texel) + 0.5) / vec2<f32>(size) - 0.5) * 2.0;
    // The exposure is the level's alone: the game's sky and sun are pictures with no level of their own against the
    // ground's, so seeing them never darkens the ground under them, and a frame of sky alone holds the exposure it had.
    let is_sky: bool = textureLoad(depth_target, min(texel, textureDimensions(depth_target) - 1u), 0) <= 0.0;
    let weight: u32 = select(1u + u32(3.0 * saturate(1.0 - off_centre)), 0u, is_sky);

    atomicAdd(&local_bins[bin], weight);

    // The sky as the game paints it: the light by the scale its pictures were drawn at this frame.
    if (is_sky) {
      let painted: f32 = light / max(light_unit(lighting) * picture_share(lighting), 1e-12);
      let level: f32 = clamp(log2(max(painted, 1e-12)) - SKY_FLOOR_LOG, 0.0, -2.0 * SKY_FLOOR_LOG);

      atomicAdd(&local_sky[0], u32(round(level * SKY_STEPS)));
      atomicAdd(&local_sky[1], 1u);
      atomicAdd(&local_sky[2], u32(round(level * level)));
    }
  }

  workgroupBarrier();

  if (index < BINS) {
    atomicAdd(&bins[index], atomicLoad(&local_bins[index]));
  }

  if (index < 3u) {
    atomicAdd(&bins[SKY_SUM + index], atomicLoad(&local_sky[index]));
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

  atomicStore(&bins[SKY_SUM], 0u);
  atomicStore(&bins[SKY_COUNT], 0u);
  atomicStore(&bins[SKY_SQUARES], 0u);

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
