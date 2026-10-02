#import "common/fullscreen"

// FidelityFX Super Resolution 1 (`ffx_fsr1.h`, MIT): EASU upscales a frame drawn smaller than its viewport, RCAS
// sharpens an upscaled one.

struct Upscale {
  // The size EASU upscales to, in pixels.
  output_size: vec2<f32>,
  // RCAS's `con`: `exp2(-stops)`, one sharpening most.
  sharpness: f32,
  pad: f32,
};

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var<uniform> upscale: Upscale;

// EASU's taps by `ffx_fsr1.h`'s names, as indices into its twelve.
const B: u32 = 0u;
const C: u32 = 1u;
const E: u32 = 2u;
const F: u32 = 3u;
const G: u32 = 4u;
const H: u32 = 5u;
const I: u32 = 6u;
const J: u32 = 7u;
const K: u32 = 8u;
const L: u32 = 9u;
const N: u32 = 10u;
const O: u32 = 11u;

// `FSR_RCAS_LIMIT`: the most negative lobe, so the sharpened pixel never rings past its neighbours.
const RCAS_LIMIT: f32 = 0.25 - 1.0 / 16.0;

fn load_clamped(at: vec2<f32>, size: vec2<f32>) -> vec3<f32> {
  return textureLoad(source, vec2<i32>(clamp(at, vec2<f32>(0.0), size - 1.0)), 0).rgb;
}

// Luma times two, in two multiply-adds.
fn luma2(color: vec3<f32>) -> f32 {
  return color.z * 0.5 + (color.x * 0.5 + color.y);
}

// `fsrEasuSetFloat`'s one axis: a reversal of the gradient comes to nothing, a steady one to one, shaped by its square.
// The span is kept off zero, where HLSL's approximate reciprocal is large but finite and WGSL's is infinite.
fn easu_axis(before: f32, centre: f32, after: f32) -> vec2<f32> {
  let span: f32 = max(max(abs(after - centre), abs(centre - before)), 1e-8);
  let delta: f32 = after - before;
  let strength: f32 = saturate(abs(delta) / span);

  return vec2<f32>(delta, strength * strength);
}

// `fsrEasuSetFloat`: one bilinear corner's share of the edge's direction (xy) and strength (z), from the `+` of lumas
// around it.
fn easu_set(weight: f32, above: f32, left: f32, centre: f32, right: f32, below: f32) -> vec3<f32> {
  let x: vec2<f32> = easu_axis(left, centre, right);
  let y: vec2<f32> = easu_axis(above, centre, below);

  return vec3<f32>(x.x, y.x, x.y + y.y) * weight;
}

// `fsrEasuTapFloat`: one tap's weight, its offset turned along the edge and stretched, under the approximate Lanczos-2
// `(25/16 (2/5 x² - 1)² - 9/16) (w x² - 1)²` clipped at the window's end.
fn easu_tap(offset: vec2<f32>, direction: vec2<f32>, anisotropy: vec2<f32>, lobe: f32, clip: f32) -> f32 {
  let turned: vec2<f32> = vec2<f32>(
    offset.x * direction.x + offset.y * direction.y,
    -offset.x * direction.y + offset.y * direction.x,
  ) * anisotropy;
  let distance: f32 = min(dot(turned, turned), clip);
  let base: f32 = distance * (2.0 / 5.0) - 1.0;
  let window: f32 = lobe * distance - 1.0;

  return (base * base * (25.0 / 16.0) - (25.0 / 16.0 - 1.0)) * (window * window);
}

// `ffxFsrEasuFloat`: each output pixel reconstructed from the twelve drawn texels around it with a Lanczos-2
// approximation stretched along the local edge, deringed to the four nearest.
@fragment
fn fs_easu(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let input_size: vec2<f32> = vec2<f32>(textureDimensions(source));
  let output_size: vec2<f32> = upscale.output_size;
  // The output pixel's centre in the drawn pixels, less half a texel, so `f` is the texel at or left of it.
  let position: vec2<f32> = (floor(in.clip.xy) + 0.5) * (input_size / output_size) - 0.5;
  let base: vec2<f32> = floor(position);
  let fraction: vec2<f32> = position - base;
  // `b c` above, `e f g h` and `i j k l` across, `n o` below.
  var offsets: array<vec2<f32>, 12> = array<vec2<f32>, 12>(
    vec2<f32>(0.0, -1.0), vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, 0.0), vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(2.0, 0.0),
    vec2<f32>(-1.0, 1.0), vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 1.0), vec2<f32>(2.0, 1.0),
    vec2<f32>(0.0, 2.0), vec2<f32>(1.0, 2.0),
  );
  var colors: array<vec3<f32>, 12>;
  var lumas: array<f32, 12>;

  for (var index: u32 = 0u; index < 12u; index++) {
    colors[index] = load_clamped(base + offsets[index], input_size);
    lumas[index] = luma2(colors[index]);
  }

  let px: f32 = fraction.x;
  let py: f32 = fraction.y;
  let sums: vec3<f32> = easu_set((1.0 - px) * (1.0 - py), lumas[B], lumas[E], lumas[F], lumas[G], lumas[J])
    + easu_set(px * (1.0 - py), lumas[C], lumas[F], lumas[G], lumas[H], lumas[K])
    + easu_set((1.0 - px) * py, lumas[F], lumas[I], lumas[J], lumas[K], lumas[N])
    + easu_set(px * py, lumas[G], lumas[J], lumas[K], lumas[L], lumas[O]);

  // Normalized, and a direction too short to trust taken as across.
  let squared: f32 = dot(sums.xy, sums.xy);
  let is_flat: bool = squared < 1.0 / 32768.0;
  let direction: vec2<f32> = select(sums.xy * inverseSqrt(max(squared, 1e-12)), vec2<f32>(1.0, 0.0), is_flat);
  // From `0..2` to `0..1`, shaped by its square.
  let edge: f32 = sums.z * 0.5;
  let shaped: f32 = edge * edge;
  // The kernel stretched from one across or down to `sqrt(2)` on a diagonal.
  let stretch: f32 = dot(direction, direction) / max(max(abs(direction.x), abs(direction.y)), 1e-8);
  let anisotropy: vec2<f32> = vec2<f32>(1.0 + (stretch - 1.0) * shaped, 1.0 - 0.5 * shaped);
  // The window shifts from `sqrt(2)` to slightly past two as the edge grows.
  let lobe: f32 = 0.5 + (1.0 / 4.0 - 0.04 - 0.5) * shaped;
  let clip: f32 = 1.0 / lobe;
  let least: vec3<f32> = min(min(colors[F], colors[G]), min(colors[J], colors[K]));
  let most: vec3<f32> = max(max(colors[F], colors[G]), max(colors[J], colors[K]));
  var color: vec3<f32> = vec3<f32>(0.0);
  var weight: f32 = 0.0;

  for (var index: u32 = 0u; index < 12u; index++) {
    let tap: f32 = easu_tap(offsets[index] - fraction, direction, anisotropy, lobe, clip);

    color += colors[index] * tap;
    weight += tap;
  }

  return vec4<f32>(min(most, max(least, color / weight)), 1.0);
}

// `FsrRcasF`: each pixel sharpened by a lobe over its four neighbours, as strong as its neighbourhood leaves room for
// without clipping, so an edge is crisped and a flat area left alone.
@fragment
fn fs_rcas(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let size: vec2<f32> = vec2<f32>(textureDimensions(source));
  let at: vec2<f32> = floor(in.clip.xy);
  let b: vec3<f32> = load_clamped(at + vec2<f32>(0.0, -1.0), size);
  let d: vec3<f32> = load_clamped(at + vec2<f32>(-1.0, 0.0), size);
  let e: vec3<f32> = load_clamped(at, size);
  let f: vec3<f32> = load_clamped(at + vec2<f32>(1.0, 0.0), size);
  let h: vec3<f32> = load_clamped(at + vec2<f32>(0.0, 1.0), size);
  let least: vec3<f32> = min(min(b, d), min(f, h));
  let most: vec3<f32> = max(max(b, d), max(f, h));
  // How far each channel may go down before the darkest neighbour clips, and up before the brightest does.
  let hit_min: vec3<f32> = least / max(most * 4.0, vec3<f32>(1e-5));
  let hit_max: vec3<f32> = (1.0 - most) / min(least * 4.0 - 4.0, vec3<f32>(-1e-5));
  let lobes: vec3<f32> = max(-hit_min, hit_max);
  let lobe: f32 = max(-RCAS_LIMIT, min(max(lobes.x, max(lobes.y, lobes.z)), 0.0)) * upscale.sharpness;

  return vec4<f32>(((b + d + f + h) * lobe + e) / (lobe * 4.0 + 1.0), 1.0);
}
