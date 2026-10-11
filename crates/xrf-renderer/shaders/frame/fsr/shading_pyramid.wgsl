#import "common/fullscreen"
#import "frame/fsr/common"

// `ffx_fsr3upscaler_shading_change_pyramid.h`'s first level: at half the drawn size, the mean over the two by two drawn
// texels under each texel of how far the luma about each changed since the frame before (`SpdLoadSourceImage`), and of
// that change's sign. `downsample` makes the levels after it.

@group(0) @binding(1) var dilated_motion: texture_2d<f32>;
@group(0) @binding(2) var luma: texture_2d<f32>;
@group(0) @binding(3) var previous_luma: texture_2d<f32>;

// `SHADING_CHANGE_SET_SIZE` and its `iSampleOffsets`: the centre and the four beside it.
const SET_SIZE: i32 = 5;
const SET_OFFSETS: array<vec2<f32>, 5> = array<vec2<f32>, 5>(
  vec2<f32>(0.0, 0.0), vec2<f32>(-1.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, -1.0), vec2<f32>(0.0, 1.0),
);

// `CompareSwap`.
fn compare_swap(samples: ptr<function, array<f32, 5>>, i: i32, j: i32) {
  let least: f32 = min((*samples)[i], (*samples)[j]);

  (*samples)[j] = max((*samples)[i], (*samples)[j]);
  (*samples)[i] = least;
}

// `SortSet`: a sorting network of five.
fn sort_set(samples: ptr<function, array<f32, 5>>) {
  compare_swap(samples, 0, 3);
  compare_swap(samples, 1, 4);
  compare_swap(samples, 0, 2);
  compare_swap(samples, 1, 3);
  compare_swap(samples, 0, 1);
  compare_swap(samples, 2, 4);
  compare_swap(samples, 1, 2);
  compare_swap(samples, 3, 4);
  compare_swap(samples, 2, 3);
}

// `ComputeMinimumDifference`: walking both sets sorted, the smallest signed relative difference between a luma of one
// and a luma of the other; none where any two match.
fn minimum_difference(current: array<f32, 5>, previous: array<f32, 5>) -> f32 {
  var set0: array<f32, 5> = current;
  var set1: array<f32, 5> = previous;
  var smallest: f32 = FSR_FP16_MAX - 1.0;
  var a: i32 = 0;
  var b: i32 = 0;

  sort_set(&set0);
  sort_set(&set1);

  if (min(set0[SET_SIZE - 1], set1[SET_SIZE - 1]) > FSR_FP32_MIN) {
    for (var index: i32 = 0; index < SET_SIZE && smallest < FSR_FP16_MAX; index++) {
      var diff: f32 = set0[a] - set1[b];

      if (abs(diff) > FSR_FP16_MIN) {
        diff = sign(diff) * (1.0 - min_over_max(set0[a], set1[b]));
        smallest = select(smallest, diff, abs(diff) < abs(smallest));
        // AMD's order: `b` steps against `a` as already stepped.
        a += i32(set0[a] < set1[b]);
        b += i32(set0[a] >= set1[b]);
      } else {
        smallest = FSR_FP16_MAX;
      }
    }
  }

  return smallest * f32(smallest < FSR_FP16_MAX - 1.0);
}

// The five lumas of a set about a texel of a texture of a size, kept off zero: `GetCurrentLumaBilinearSamples` and
// `GetPreviousLumaBilinearSamples` with an exposure of one.
fn load_set(source: texture_2d<f32>, base: vec2<f32>, size: vec2<f32>) -> array<f32, 5> {
  var samples: array<f32, 5>;

  for (var index: i32 = 0; index < SET_SIZE; index++) {
    samples[index] = max(textureLoad(source, clamped_texel(base + SET_OFFSETS[index], size), 0).x, FSR_EPSILON);
  }

  return samples;
}

// `ComputeDiff`: how far the luma about a drawn texel changed since the frame before, where its surface was on the
// screen then.
fn shading_difference(position: vec2<f32>) -> f32 {
  let texel: vec2<i32> = clamped_texel(position, fsr.render_size);
  let motion: vec2<f32> = textureLoad(dilated_motion, texel, 0).xy;
  let uv: vec2<f32> = (vec2<f32>(texel) + 0.5) / fsr.render_size;
  let reprojected: vec2<f32> = uv + fsr.previous_jitter / fsr.render_size + motion;

  if (!is_uv_inside(reprojected)) {
    return 0.0;
  }

  let current: array<f32, 5> = load_set(luma, floor((uv + fsr.jitter / fsr.render_size) * fsr.render_size),
    fsr.render_size);
  let previous: array<f32, 5> = load_set(previous_luma, floor(reprojected * fsr.render_size), fsr.render_size);

  return minimum_difference(current, previous);
}

@fragment
fn fs_shading_pyramid(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let base: vec2<f32> = floor(in.clip.xy) * 2.0;
  var sum: vec2<f32> = vec2<f32>(0.0);

  for (var corner: i32 = 0; corner < 4; corner++) {
    let difference: f32 = shading_difference(base + footprint_corner(corner));

    // `DIFFERENCE`, and `SIGN_SUM`: none where nothing changed.
    sum += vec2<f32>(difference, select(0.0, sign(difference), difference != 0.0));
  }

  return vec4<f32>(sum * 0.25, 0.0, 1.0);
}
