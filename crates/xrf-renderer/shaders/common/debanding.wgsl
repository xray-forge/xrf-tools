#import "common/display_transform"

// Debanding: a pixel averaged with four neighbours scattered around it wherever none of them, nor their mean, differs
// from it by more than a band's step, so a gradient's eight-bit bands blend into each other and edges stay. Each pass
// reads further out, by a radius and a direction drawn from noise. Taking the image and its sampler as arguments, so
// the present's frame and the fog scattering's blur share it; an image of linear light has its reads tonemapped, so
// the steps compared are the shown ones.

// What a neighbour, or the mean, may differ from the pixel by and still blend: ten eight-bit steps.
const DEBAND_THRESHOLD: f32 = 0.039215;

fn deband_wrap(value: f32) -> f32 {
  return value - floor(value / 289.0) * 289.0;
}

// A permutation polynomial over the integers below 289, scrambling a value into another of that range.
fn deband_permute(value: f32) -> f32 {
  return deband_wrap((value * 34.0 + 1.0) * value);
}

// A hash of the coordinates moving with the clock, from nothing to a quarter.
fn deband_noise(uv: vec2<f32>, time: f32) -> f32 {
  return fract(sin(dot(uv, vec2<f32>(12.0, 78.0) + time)) * 43758.0) * 0.25;
}

// A read of the image as the display shows it under `display` (`shown_for_display`).
fn deband_read(image: texture_2d<f32>, image_sampler: sampler, uv: vec2<f32>, display: vec4<f32>) -> vec3<f32> {
  return shown_for_display(textureSampleLevel(image, image_sampler, uv, 0.0).rgb, display);
}

// One pass: the four diagonal neighbours `offset` away, blended in by how close they and their mean lie to the shown
// colour.
fn deband_pass(image: texture_2d<f32>, image_sampler: sampler, color: vec3<f32>, uv: vec2<f32>, offset: vec2<f32>,
  display: vec4<f32>) -> vec3<f32> {
  let first: vec3<f32> = deband_read(image, image_sampler, uv + offset, display);
  let second: vec3<f32> = deband_read(image, image_sampler, uv - offset, display);
  let third: vec3<f32> = deband_read(image, image_sampler, uv + vec2<f32>(-offset.x, offset.y), display);
  let fourth: vec3<f32> = deband_read(image, image_sampler, uv + vec2<f32>(offset.x, -offset.y), display);
  let mean: vec3<f32> = (first + second + third + fourth) / 4.0;
  let widest: vec3<f32> = max(max(abs(color - first), abs(color - second)), max(abs(color - third), abs(color - fourth)));
  let blend: vec3<f32> = saturate((1.0 - abs(color - mean) / DEBAND_THRESHOLD) * 3.0) *
    saturate((1.0 - widest / DEBAND_THRESHOLD) * 3.0);

  return mix(color, mean, blend);
}

// A shown colour read at `uv` debanded over `passes` passes reaching `radius` pixels of `pixel` at most, the noise
// turning with `time`; `display` how its reads are shown, as `deband_read` takes it.
fn debanded(image: texture_2d<f32>, image_sampler: sampler, color: vec3<f32>, uv: vec2<f32>, pixel: vec2<f32>,
  passes: u32, radius: f32, time: f32, display: vec4<f32>) -> vec3<f32> {
  var noise: f32 = deband_permute(deband_permute(deband_permute(uv.x) + uv.y) + deband_noise(uv, time));
  let direction: vec2<f32> = vec2<f32>(cos(noise), sin(noise));
  var result: vec3<f32> = color;

  for (var index: u32 = 1u; index <= passes; index++) {
    noise = deband_permute(noise);

    let reach: f32 = f32(index) / f32(passes);

    result = deband_pass(image, image_sampler, result, uv, noise * 0.0033 * radius * reach * reach * pixel * direction,
      display);
  }

  return result;
}
