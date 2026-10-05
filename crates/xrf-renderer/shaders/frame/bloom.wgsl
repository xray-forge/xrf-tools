#import "common/fullscreen"

// The engine's bloom (`phase_bloom`): the frame's high part boxed into a 256-square target (`bloom_build.ps`), then
// blurred across and down by a fifteen-tap Gaussian (`bloom_filter.ps`). Every target is eight bits a channel, as the
// engine's are, so each stage clamps to one.

// The source and its sampler, and the draw's `Bloom`: for the build, x, y half a texel of the frame and z the threshold;
// for a filter, x, y one texel of the target along its way and z one where only the side behind is read, as Anomaly's
// `bloom_filter.ps` reads it; then `CalcGauss_wave`'s weights, the taps one to four out, then five to seven out and the
// centre.
#import "generated/frame/bloom"

// `BLOOM_size_X`, `BLOOM_size_Y`.
const BLOOM_SIZE: vec2<f32> = vec2<f32>(256.0, 256.0);

// Between the build's taps: half the frame across the target's 256 texels, `(1 / w) * (w / 2 / 256)`, whatever the
// frame's size.
const BUILD_STEP: f32 = 1.0 / 512.0;

// The source's colour at a point, filtered between its texels.
fn read_source(at: vec2<f32>) -> vec3<f32> {
  return textureSampleLevel(source, source_sampler, at, 0.0).rgb;
}

// `bloom_build.ps`: four reads of the high part around a texel of the target, twice their mean, and the brightness their
// three channels sum to less the threshold.
@fragment
fn fs_bloom_build(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let at: vec2<f32> = in.clip.xy / BLOOM_SIZE + bloom.params.xy;
  let sum: vec3<f32> = read_source(at) + read_source(at + vec2<f32>(BUILD_STEP, 0.0)) +
    read_source(at + vec2<f32>(0.0, BUILD_STEP)) + read_source(at + vec2<f32>(BUILD_STEP));
  let average: vec3<f32> = sum / 2.0;

  return vec4<f32>(average, dot(average, vec3<f32>(1.0)) - bloom.params.z);
}

// The weight of the tap `index` out from the centre, one to seven.
fn tap_weight(index: u32) -> f32 {
  if (index <= 4u) {
    return bloom.weights[0][index - 1u];
  }

  return bloom.weights[1][index - 5u];
}

// `bloom_filter.ps`: the centre and seven taps each way, each between two texels, 1.5, 3.5 ... 13.5 texels out.
@fragment
fn fs_bloom_filter(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let at: vec2<f32> = in.clip.xy / BLOOM_SIZE;
  let along: vec2<f32> = bloom.params.xy;
  let is_one_sided: bool = bloom.params.z > 0.5;
  var accumulated: vec4<f32> = bloom.weights[1].w * textureSampleLevel(source, source_sampler, at, 0.0);

  for (var index: u32 = 1u; index <= 7u; index++) {
    let offset: vec2<f32> = along * (2.0 * f32(index) - 0.5);
    let weight: f32 = tap_weight(index);

    accumulated += weight * textureSampleLevel(source, source_sampler, at - offset, 0.0);

    if (!is_one_sided) {
      accumulated += weight * textureSampleLevel(source, source_sampler, at + offset, 0.0);
    }
  }

  return accumulated;
}
