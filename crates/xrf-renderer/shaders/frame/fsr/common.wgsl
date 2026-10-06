// FidelityFX FSR 2.2 (`ffx_fsr2_common.h`, `ffx_fsr2_sample.h`, AMD, MIT): what its passes share, for reversed depth,
// colour in the display's range with an exposure of one, and motion drawn at the render size.

#import "generated/structs"

// `cbFSR2`'s fields the passes read, declared in Rust.
@group(0) @binding(0) var<uniform> fsr: Fsr;

const FSR2_EPSILON: f32 = 1e-3;
const FSR2_FP16_MAX: f32 = 65504.0;
// `fReconstructedDepthBilinearWeightThreshold`.
const RECONSTRUCTED_DEPTH_WEIGHT_THRESHOLD: f32 = 0.01;
// `fUpsampleLanczosWeightScale`.
const UPSAMPLE_LANCZOS_WEIGHT_SCALE: f32 = 1.0 / 12.0;
// `fMaxAccumulationLanczosWeight`.
const MAX_ACCUMULATION_LANCZOS_WEIGHT: f32 = 1.0;
// `fAverageLanczosWeightPerFrame`.
const AVERAGE_LANCZOS_WEIGHT_PER_FRAME: f32 = 0.74 / 12.0;
const PI: f32 = 3.14159265;

// The nine texels around the centre, the centre first: `FindNearestDepth`'s order.
const NEAREST_ORDER: array<vec2<f32>, 9> = array<vec2<f32>, 9>(
  vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0), vec2<f32>(0.0, -1.0), vec2<f32>(-1.0, 0.0),
  vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0),
);

// The nine texels around the centre, row by row.
fn neighbour(index: i32) -> vec2<f32> {
  return vec2<f32>(f32(index % 3 - 1), f32(index / 3 - 1));
}

fn to_ycocg(rgb: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(rgb.x * 0.25 + rgb.y * 0.5 + rgb.z * 0.25, rgb.x * 0.5 - rgb.z * 0.5,
    rgb.x * -0.25 + rgb.y * 0.5 - rgb.z * 0.25);
}

fn to_rgb(ycocg: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(ycocg.x + ycocg.y - ycocg.z, ycocg.x + ycocg.z, ycocg.x - ycocg.y - ycocg.z);
}

fn to_luma(rgb: vec3<f32>) -> f32 {
  return dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// `RGBToPerceivedLuma`: CIE lightness over a hundred.
fn to_perceived_luma(rgb: vec3<f32>) -> f32 {
  let luminance: f32 = to_luma(rgb);

  return select(pow(max(luminance, 0.0), 1.0 / 3.0) * 116.0 - 16.0, luminance * (24389.0 / 27.0),
    luminance <= 216.0 / 24389.0) * 0.01;
}

// The natural log of a luma kept off zero, as the luminance pyramid takes it.
fn to_log_luma(rgb: vec3<f32>) -> f32 {
  return log(max(to_luma(rgb), FSR2_EPSILON));
}

// `GetViewSpaceDepth`: a device depth as a distance along the view.
fn to_view_depth(depth: f32) -> f32 {
  return fsr.device_to_view.y / (depth - fsr.device_to_view.x);
}

// `IsOnScreen`, for a texel held as floats.
fn is_on_screen(position: vec2<f32>, size: vec2<f32>) -> bool {
  return all(position >= vec2<f32>(0.0)) && all(position < size);
}

// `ClampUv`: a coordinate kept half a texel inside a texture of a size.
fn clamped_uv(uv: vec2<f32>, size: vec2<f32>) -> vec2<f32> {
  return clamp(uv * size, vec2<f32>(0.5), size - 0.5) / size;
}

// A texel held as floats, the nearest on a texture of a size where it lies off it.
fn clamped_texel(position: vec2<f32>, size: vec2<f32>) -> vec2<i32> {
  return vec2<i32>(clamp(position, vec2<f32>(0.0), size - 1.0));
}

// `ComputeHrPosFromLrPos`: the display pixel a drawn texel's sample falls in.
fn to_display_position(render_position: vec2<f32>) -> vec2<f32> {
  return floor((render_position + 0.5 - fsr.jitter) / fsr.render_size * fsr.display_size);
}

// `Lanczos2`: the reference two-lobe window.
fn lanczos2(x: f32) -> f32 {
  let at: f32 = min(abs(x), 2.0);

  if (at < FSR2_EPSILON) {
    return 1.0;
  }

  return sin(at * PI) / (at * PI) * (sin(at * 0.5 * PI) / (at * 0.5 * PI));
}

// `Lanczos2ApproxSq`: FSR 1's approximation, of the squared distance, no further than two.
fn lanczos2_approx_sq(x2: f32) -> f32 {
  let at: f32 = min(x2, 4.0);
  let a: f32 = at * (2.0 / 5.0) - 1.0;
  let b: f32 = at * 0.25 - 1.0;

  return (a * a * (25.0 / 16.0) - (25.0 / 16.0 - 1.0)) * (b * b);
}

// `MinDividedByMax`.
fn min_over_max(a: f32, b: f32) -> f32 {
  let most: f32 = max(a, b);

  return select(0.0, min(a, b) / max(most, 1e-20), most != 0.0);
}

// The four texels a bilinear read of a point takes: the first, held as floats, and each one's weight.
struct BilinearFootprint {
  base: vec2<f32>,
  weights: vec4<f32>,
};

fn bilinear_footprint(uv: vec2<f32>, size: vec2<f32>) -> BilinearFootprint {
  let position: vec2<f32> = uv * size - 0.5;
  let base: vec2<f32> = floor(position);
  let f: vec2<f32> = position - base;

  return BilinearFootprint(base, vec4<f32>((1.0 - f.x) * (1.0 - f.y), f.x * (1.0 - f.y), (1.0 - f.x) * f.y, f.x * f.y));
}

// The offset of a footprint's corner, in its weights' order.
fn footprint_corner(index: i32) -> vec2<f32> {
  return vec2<f32>(f32(index % 2), f32(index / 2));
}
