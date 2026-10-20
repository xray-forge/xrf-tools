#import "common/luminance"

// The physical light's exposure: the scene's light counted into a histogram of its log2 luminance, a texel in every
// four each way and the centre weighed up to four times the edges, then the average between the darkest and the
// brightest shares the settings leave out brought to the target, less the share of its distance from a daylit
// exterior it keeps, the scale moving towards it at most so many stops a second, faster darkening than brightening.

#import "generated/frame/exposure_histogram"

// Bins the luminance is counted in (`ExposureHistogramUniform::BINS`).
const BINS: u32 = 128u;
// Texels each way a sample stands for.
const SAMPLE_STRIDE: u32 = 4u;
// Invocations a counting workgroup runs each way (`ExposurePass::HISTOGRAM_WORKGROUP`).
const HISTOGRAM_WORKGROUP: u32 = 16u;

var<workgroup> local_bins: array<atomic<u32>, 128>;
var<workgroup> counts: array<u32, 128>;

@compute @workgroup_size(16, 16)
fn cs_bin(@builtin(global_invocation_id) id: vec3<u32>, @builtin(local_invocation_index) index: u32) {
  if (index < BINS) {
    atomicStore(&local_bins[index], 0u);
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
    let weight: u32 = 1u + u32(3.0 * saturate(1.0 - off_centre));

    atomicAdd(&local_bins[bin], weight);
  }

  workgroupBarrier();

  if (index < BINS) {
    atomicAdd(&bins[index], atomicLoad(&local_bins[index]));
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

  var total: f32 = 0.0;

  for (var bin: u32 = 0u; bin < BINS; bin++) {
    total += f32(counts[bin]);
  }

  if (total <= 0.0) {
    return;
  }

  let low: f32 = total * params.low_share;
  let high: f32 = total * params.high_share;
  var below: f32 = 0.0;
  var weight: f32 = 0.0;
  var sum: f32 = 0.0;

  // Each bin's count within the window, at the log2 luminance of its middle.
  for (var bin: u32 = 0u; bin < BINS; bin++) {
    let count: f32 = f32(counts[bin]);
    let windowed: f32 = max(min(below + count, high) - max(below, low), 0.0);

    below += count;
    weight += windowed;
    sum += windowed * (params.least_log + (f32(bin) + 0.5) / f32(BINS) * params.span_log);
  }

  let average: f32 = select(log2(params.aim), sum / max(weight, 1e-6), weight > 0.0);
  let shown: f32 = log2(params.aim) + params.kept_share * (average - params.reference_log);
  let wanted: f32 = clamp(shown - average, params.least_scale_log, params.most_scale_log);
  let current: f32 = log2(max(state.adapted, 1e-6));
  let reach: f32 = select(params.brightening, params.darkening, wanted < current) * params.delta;
  let next: f32 = select(current + clamp(wanted - current, -reach, reach), wanted, params.snap > 0.5);

  state.used = state.adapted;
  state.adapted = exp2(next);
}
