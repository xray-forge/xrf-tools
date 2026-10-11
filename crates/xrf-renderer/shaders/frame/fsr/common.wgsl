// AMD FSR 3.1.5's upscaler (FidelityFX, `ffx_fsr3upscaler_common.h` and `ffx_fsr3upscaler_sample.h`; AMD, MIT): what
// its passes share, for reversed depth, colour with an exposure of one, and motion drawn at the render size. Every
// target is exactly its size, so `ClampUv` against a larger resource is clamping within the size itself.

#import "generated/structs"

// `cbFSR3Upscaler`'s fields the passes read, declared in Rust.
@group(0) @binding(0) var<uniform> fsr: Fsr;

const FSR_FP16_MIN: f32 = 6.10e-5;
const FSR_FP16_MAX: f32 = 65504.0;
const FSR_FP32_MIN: f32 = 1.175494351e-38;
const FSR_FP32_MAX: f32 = 3.402823466e38;
const FSR_EPSILON: f32 = FSR_FP16_MIN;
// `fReconstructedDepthBilinearWeightThreshold`.
const RECONSTRUCTED_DEPTH_WEIGHT_THRESHOLD: f32 = FSR_EPSILON * 10.0;
// `fAverageLanczosWeightPerFrame`: the share of a frame a jittered sample's Lanczos weight averages.
const AVERAGE_LANCZOS_WEIGHT_PER_FRAME: f32 = 0.74 / 16.0;
// `fLockThreshold` and `fLockMax`: a lock counts past the threshold, and lives no longer than the maximum.
const LOCK_THRESHOLD: f32 = 1.0;
const LOCK_MAX: f32 = 2.0;
// `fAccumulationAddedPerFrame` and `fMinDisocclusionAccumulation`, AMD's defaults: a third of a frame's trust regained
// each frame, and a disoccluded texel starting a third below none.
const ACCUMULATION_ADDED_PER_FRAME: f32 = 1.0 / 3.0;
const MIN_DISOCCLUSION_ACCUMULATION: f32 = -1.0 / 3.0;
const PI: f32 = 3.14159265;

// The channels of the dilated reactive masks: `REACTIVE`, `DISOCCLUSION`, `SHADING_CHANGE` and `ACCUMULAION`.
const MASK_REACTIVE: i32 = 0;
const MASK_DISOCCLUSION: i32 = 1;
const MASK_SHADING_CHANGE: i32 = 2;
const MASK_ACCUMULATION: i32 = 3;

// The nine texels around the centre, the centre first: `FindDepthExtents`'s order.
const NEAREST_ORDER: array<vec2<f32>, 9> = array<vec2<f32>, 9>(
  vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0), vec2<f32>(0.0, -1.0), vec2<f32>(-1.0, 0.0),
  vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0),
);

fn to_ycocg(rgb: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(rgb.x * 0.25 + rgb.y * 0.5 + rgb.z * 0.25, rgb.x * 0.5 - rgb.z * 0.5,
    rgb.x * -0.25 + rgb.y * 0.5 - rgb.z * 0.25);
}

fn to_rgb(ycocg: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(ycocg.x + ycocg.y - ycocg.z, ycocg.x + ycocg.z, ycocg.x - ycocg.y - ycocg.z);
}

// `Tonemap` and `InverseTonemap`: the reversible tonemap the HDR option accumulates through, by the largest channel.
fn tonemapped(rgb: vec3<f32>) -> vec3<f32> {
  return rgb / (max(max(rgb.x, max(rgb.y, rgb.z)), 0.0) + 1.0);
}

fn inverse_tonemapped(rgb: vec3<f32>) -> vec3<f32> {
  return rgb / max(FSR_EPSILON, 1.0 - max(rgb.x, max(rgb.y, rgb.z)));
}

// Whether the colour is linear light past the display's range: `FFX_FSR3UPSCALER_OPTION_HDR_COLOR_INPUT`.
fn is_hdr() -> bool {
  return fsr.hdr > 0.5;
}

// `RGBToLuma`.
fn to_luma(rgb: vec3<f32>) -> f32 {
  return dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// `Get4KVelocity`: a motion's length in pixels of a 4K view, whatever the view's own size.
fn to_4k_velocity(motion: vec2<f32>) -> f32 {
  return length(motion * vec2<f32>(3840.0, 2160.0));
}

// `ReconstructedDepthMvPxThreshold`: the 4K pixels of motion under which a surface counts as still, more the farther
// it stands.
fn still_threshold(depth_in_meters: f32) -> f32 {
  return mix(0.25, 0.75, saturate(depth_in_meters / 100.0));
}

// `GetViewSpaceDepth`: a device depth as a distance along the view, in metres as X-Ray's units are.
fn to_view_depth(depth: f32) -> f32 {
  return fsr.device_to_view.y / (depth - fsr.device_to_view.x);
}

// `IsOnScreen`, for a texel held as floats.
fn is_on_screen(position: vec2<f32>, size: vec2<f32>) -> bool {
  return all(position >= vec2<f32>(0.0)) && all(position < size);
}

// `IsUvInside`.
fn is_uv_inside(uv: vec2<f32>) -> bool {
  return all(uv >= vec2<f32>(0.0)) && all(uv <= vec2<f32>(1.0));
}

// `ClampUv`: a coordinate kept half a texel inside a texture of a size.
fn clamped_uv(uv: vec2<f32>, size: vec2<f32>) -> vec2<f32> {
  return clamp(uv * size, vec2<f32>(0.5), size - 0.5) / size;
}

// `ClampLoad`: a texel held as floats, the nearest on a texture of a size where it lies off it.
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

  if (at < FSR_EPSILON) {
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

// `MinDividedByMax`, with what it gives where both are zero.
fn min_over_max_or(a: f32, b: f32, on_zero: f32) -> f32 {
  let most: f32 = max(a, b);

  return select(on_zero, min(a, b) / max(most, FSR_FP32_MIN), most != 0.0);
}

fn min_over_max(a: f32, b: f32) -> f32 {
  return min_over_max_or(a, b, 0.0);
}

// The four texels a bilinear read of a point takes: the first, held as floats, and each one's weight.
struct BilinearFootprint {
  base: vec2<f32>,
  weights: vec4<f32>,
};

// `GetBilinearSamplingData`.
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
