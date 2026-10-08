#import "common/camera"
#import "common/fullscreen"

// The enhanced bloom, from the finished scene rather than the engine's high part: the scene blurred to half its size by
// a Gaussian across and down; its bright parts raised to the threshold's power, the sky's scaled, self-lit surfaces
// added whole; halved six times and doubled back, each doubling adding the halving of its size; then split into its
// colours a few pixels apart, lifted, tonemapped and saturated. The present screens it over the frame. Alpha carries how
// much of a texel is self-lit through every stage.

// The image a stage reads, the coarser halving a doubling adds, their sampler, the frame's depth and marks, and the
// stage's sizes, spread and strengths.
#import "generated/frame/enhanced_bloom"

// The Gaussian blurring the scene to half its size: its deviation, in half-size texels, and the reads each side.
const BLUR_SIGMA: f32 = 2.5;
const BLUR_REACH: i32 = 3;
// What a self-lit surface adds to the bloom, whatever its colour.
const SELF_LIT: f32 = 0.25;
// How far, in the frame's pixels, the finish reads the bloom's red one way and its blue the other.
const RED_SHIFT: vec2<f32> = vec2<f32>(4.5, 5.3);
const BLUE_SHIFT: vec2<f32> = vec2<f32>(-3.2, -4.1);
// The luminance the vibrance pulls towards, `LUMINANCE_VECTOR`.
const LUMINANCE: vec3<f32> = vec3<f32>(0.3, 0.38, 0.22);

// The source's texel under a point, unfiltered.
fn read_nearest(at: vec2<f32>) -> vec3<f32> {
  let size: vec2<i32> = vec2<i32>(textureDimensions(source));
  let texel: vec2<i32> = clamp(vec2<i32>(floor(at * vec2<f32>(size))), vec2<i32>(0), size - 1);

  return textureLoad(source, texel, 0).rgb;
}

fn sample_at(image: texture_2d<f32>, at: vec2<f32>) -> vec4<f32> {
  return textureSampleLevel(image, source_sampler, at, 0.0);
}

// Whether a texel of the frame shows a self-lit surface, held to the frame's edges.
fn self_lit(texel: vec2<i32>) -> f32 {
  let last: vec2<i32> = vec2<i32>(bloom.size.zw) - 1;

  return select(0.0, 1.0, has_mark(textureLoad(material_target, clamp(texel, vec2<i32>(0), last), 0).a, MARK_EMISSIVE));
}

// A blur at half the frame's size, one way: seven unfiltered reads a half-size texel apart, weighed by the Gaussian.
@fragment
fn fs_bloom_blur(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let uv: vec2<f32> = in.clip.xy / bloom.size.xy;
  var sum: vec3<f32> = vec3<f32>(0.0);
  var total: f32 = 0.0;

  for (var index: i32 = -BLUR_REACH; index <= BLUR_REACH; index++) {
    let weight: f32 = exp(-0.5 * f32(index * index) / (BLUR_SIGMA * BLUR_SIGMA));

    sum += read_nearest(uv + f32(index) * bloom.spread.xy) * weight;
    total += weight;
  }

  return vec4<f32>(sum / total, 1.0);
}

// The build: the blurred scene, the sky's times its share, raised to the threshold's power; self-lit surfaces added by
// how much of the four frame texels under the texel they cover, and that share kept in alpha.
@fragment
fn fs_bloom_build(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let uv: vec2<f32> = in.clip.xy / bloom.size.xy;
  let frame: vec2<f32> = bloom.size.zw;
  let last: vec2<i32> = vec2<i32>(frame) - 1;
  let at: vec2<i32> = clamp(vec2<i32>(uv * frame), vec2<i32>(0), last);
  let is_sky: bool = textureLoad(depth_target, at, 0) <= 0.0;
  var result: vec3<f32> = sample_at(source, uv).rgb * select(1.0, bloom.strengths.z, is_sky);

  result = pow(abs(result), vec3<f32>(bloom.strengths.x));

  let corner: vec2<i32> = vec2<i32>(floor(uv * frame - 0.5));
  let covered: f32 = (self_lit(corner) + self_lit(corner + vec2<i32>(1, 0)) + self_lit(corner + vec2<i32>(0, 1)) +
    self_lit(corner + vec2<i32>(1, 1))) / 4.0;
  let added: f32 = covered * SELF_LIT;

  return vec4<f32>(saturate(result + added), added * 3.0);
}

// A halving: the source's texel four times over and its four diagonal neighbours, half a texel of the target times the
// spread out.
@fragment
fn fs_bloom_halve(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let uv: vec2<f32> = in.clip.xy / bloom.size.xy;
  let spread: vec2<f32> = 0.5 / bloom.size.xy * bloom.spread.z;
  let around: vec4<f32> = sample_at(source, uv + vec2<f32>(-spread.x, spread.y)) + sample_at(source, uv + spread) +
    sample_at(source, uv + vec2<f32>(spread.x, -spread.y)) + sample_at(source, uv - spread);

  return (sample_at(source, uv) * 4.0 + around) / 8.0;
}

// A doubling: the coarser stage's nine texels about the point, a target's texel times the spread apart, and the halving
// of that same size's four beside it.
@fragment
fn fs_bloom_double(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let uv: vec2<f32> = in.clip.xy / bloom.size.xy;
  let spacing: vec2<f32> = 1.0 / bloom.size.xy * bloom.spread.z;
  var sum: vec4<f32> = vec4<f32>(0.0);

  for (var y: i32 = -1; y <= 1; y++) {
    for (var x: i32 = -1; x <= 1; x++) {
      sum += sample_at(source, uv + vec2<f32>(f32(x), f32(y)) * spacing);
    }
  }

  sum += sample_at(coarser, uv + vec2<f32>(0.0, spacing.y)) + sample_at(coarser, uv - vec2<f32>(spacing.x, 0.0)) +
    sample_at(coarser, uv + vec2<f32>(spacing.x, 0.0)) + sample_at(coarser, uv - vec2<f32>(0.0, spacing.y));

  return saturate(sum / 13.0);
}

// Hable's filmic curve (`Uncharted2`) at an exposure, scaled so a linear white of 11.2 comes out one.
fn filmic(color: vec3<f32>, exposure: f32) -> vec3<f32> {
  let a: f32 = 0.15;
  let b: f32 = 0.5;
  let c: f32 = 0.1;
  let d: f32 = 0.2;
  let e: f32 = 0.02;
  let f: f32 = 0.3;
  let w: f32 = 11.2;
  let white: f32 = 1.0 / (((w * (a * w + c * b) + d * e) / (w * (a * w + b) + d * f)) - e / f);
  let x: vec3<f32> = color * exposure;

  return (((x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f)) - e / f) * white;
}

// The finish: red and blue read a few frame pixels apart from green, the square root of each, lifted where self-lit,
// tonemapped at the exposure and saturated by the vibrance.
@fragment
fn fs_bloom_finish(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let uv: vec2<f32> = in.clip.xy / bloom.size.xy;
  let frame: vec2<f32> = bloom.size.zw;
  let centre: vec4<f32> = sample_at(source, uv);
  var glow: vec4<f32> = vec4<f32>(sample_at(source, uv + RED_SHIFT / frame).r, centre.g,
    sample_at(source, uv + BLUE_SHIFT / frame).b, centre.a);

  glow = sqrt(abs(glow));

  let lifted: vec3<f32> = saturate(glow.rgb * (1.0 + glow.a));
  let toned: vec3<f32> = filmic(lifted, bloom.strengths.y);

  return vec4<f32>(mix(vec3<f32>(dot(toned, LUMINANCE)), toned, bloom.strengths.w), 1.0);
}
