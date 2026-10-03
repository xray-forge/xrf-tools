#import "common/fullscreen"
#import "frame/fsr/common"

// `ffx_fsr2_lock.h`: the thin features to lock, found at the render size. FSR writes each to the display pixel its
// sample falls in; the accumulation gathers it from there instead.

@group(0) @binding(1) var lock_luma: texture_2d<f32>;

// A neighbour's luma within this share of the centre's is similar to it.
const SIMILAR_THRESHOLD: f32 = 1.05;

// `ComputeThinFeatureConfidence`: a texel whose luma stands above or below all it differs from around it, and that no
// quadrant of similar texels surrounds, is a thin feature to lock.
@fragment
fn fs_lock(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let position: vec2<f32> = floor(in.clip.xy);
  let nucleus: f32 = textureLoad(lock_luma, clamped_texel(position, fsr.render_size), 0).x;
  var similar: array<bool, 9>;
  var dissimilar_min: f32 = 3.402823466e38;
  var dissimilar_max: f32 = 0.0;

  for (var index: i32 = 0; index < 9; index++) {
    if (index == 4) {
      similar[index] = true;
      continue;
    }

    let luma: f32 = textureLoad(lock_luma, clamped_texel(position + neighbour(index), fsr.render_size), 0).x;
    // `max / min < 1.05` without the division, which the reference leaves to IEEE's NaN for a black texel.
    let least: f32 = min(luma, nucleus);
    let is_similar: bool = least > 0.0 && max(luma, nucleus) < least * SIMILAR_THRESHOLD;

    similar[index] = is_similar;

    if (!is_similar) {
      dissimilar_min = min(dissimilar_min, luma);
      dissimilar_max = max(dissimilar_max, luma);
    }
  }

  let is_ridge: bool = nucleus > dissimilar_max || nucleus < dissimilar_min;
  // The quadrants of the neighbourhood, any of which all similar surrounds the centre.
  let is_surrounded: bool = (similar[0] && similar[1] && similar[3] && similar[4])
    || (similar[1] && similar[2] && similar[4] && similar[5])
    || (similar[3] && similar[4] && similar[6] && similar[7])
    || (similar[4] && similar[5] && similar[7] && similar[8]);

  return vec4<f32>(f32(is_ridge && !is_surrounded), 0.0, 0.0, 1.0);
}
