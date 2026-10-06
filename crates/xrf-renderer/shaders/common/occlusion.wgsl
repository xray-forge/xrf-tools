#import "common/camera"

// The screen's occlusion, searched at half resolution as visibility then distance along the view, and the indirect
// light gathered beside it as colour then distance; and the reflections traced at a size of their own.

// The occlusion brought up to the frame's pixel: the four searched pixels around it by how near each lies, and each by
// how near its distance lies to the pixel's, so an edge never takes the other side's.
fn upsampled_occlusion(searched: texture_2d<f32>, pixel: vec2<f32>, distance: f32) -> f32 {
  let last: vec2<f32> = vec2<f32>(textureDimensions(searched)) - 1.0;
  let base: vec2<f32> = floor(pixel / 2.0);
  let fraction: vec2<f32> = (pixel - base * 2.0) * 0.5;
  var sum: f32 = 0.0;
  var weights: f32 = 0.0;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let offset: vec2<f32> = vec2<f32>(f32(corner & 1u), f32(corner >> 1u));
    let texel: vec4<f32> = textureLoad(searched, vec2<i32>(clamp(base + offset, vec2<f32>(0.0), last)), 0);
    let bilinear: f32 = mix(1.0 - fraction.x, fraction.x, offset.x) * mix(1.0 - fraction.y, fraction.y, offset.y);
    let difference: f32 = abs(texel.y - distance) / max(distance, 1e-3);

    sum += texel.x * bilinear / (difference * difference + 1e-3);
    weights += bilinear / (difference * difference + 1e-3);
  }

  return sum / max(weights, 1e-6);
}

// The indirect light brought up to the frame's pixel as the occlusion is: the searched pixels' colour by how near each
// lies and how near its distance, in alpha, lies to the pixel's.
fn upsampled_light(gathered: texture_2d<f32>, pixel: vec2<f32>, distance: f32) -> vec3<f32> {
  let last: vec2<f32> = vec2<f32>(textureDimensions(gathered)) - 1.0;
  let base: vec2<f32> = floor(pixel / 2.0);
  let fraction: vec2<f32> = (pixel - base * 2.0) * 0.5;
  var sum: vec3<f32> = vec3<f32>(0.0);
  var weights: f32 = 0.0;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let offset: vec2<f32> = vec2<f32>(f32(corner & 1u), f32(corner >> 1u));
    let texel: vec4<f32> = textureLoad(gathered, vec2<i32>(clamp(base + offset, vec2<f32>(0.0), last)), 0);
    let bilinear: f32 = mix(1.0 - fraction.x, fraction.x, offset.x) * mix(1.0 - fraction.y, fraction.y, offset.y);
    let difference: f32 = abs(texel.a - distance) / max(distance, 1e-3);
    let weight: f32 = bilinear / (difference * difference + 1e-3);

    sum += texel.rgb * weight;
    weights += weight;
  }

  return sum / max(weights, 1e-6);
}

// The occlusion lightened by the light a surface's colour bounces between the sides of its creases, by `bounce` from
// none to all: Jimenez et al.'s cubic fit of the multiple-bounce visibility against the single one and the albedo
// ("Practical Real-Time Strategies for Accurate Indirect Occlusion", 2016), never darker than the single one.
fn bounced_occlusion(visible: f32, albedo: vec3<f32>, bounce: f32) -> vec3<f32> {
  let a: vec3<f32> = 2.0404 * albedo - 0.3324;
  let b: vec3<f32> = -4.7951 * albedo + 0.6417;
  let c: vec3<f32> = 2.7552 * albedo + 0.6903;
  let bounced: vec3<f32> = max(vec3<f32>(visible), ((visible * a + b) * visible + c) * visible);

  return mix(vec3<f32>(visible), bounced, saturate(bounce));
}

// The reflections traced brought up to the frame's pixel: the traced pixels around it, each standing for `ratio` of the
// frame's pixels a side, by how near each lies and how near the distance its pixel shows lies to the pixel's. Radiance
// times trust, then trust; alpha below none where no pixel around it was traced.
fn upsampled_reflection(traced: texture_2d<f32>, depth: texture_depth_2d, pixel: vec2<f32>, distance: f32,
  ratio: f32) -> vec4<f32> {
  let last: vec2<f32> = vec2<f32>(textureDimensions(traced)) - 1.0;
  let frame_last: vec2<f32> = camera.viewport.xy - 1.0;
  let base: vec2<f32> = floor(pixel / ratio);
  let fraction: vec2<f32> = (pixel - base * ratio) / ratio;
  var sum: vec4<f32> = vec4<f32>(0.0);
  var weights: f32 = 0.0;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let offset: vec2<f32> = vec2<f32>(f32(corner & 1u), f32(corner >> 1u));
    let at: vec2<f32> = clamp(base + offset, vec2<f32>(0.0), last);
    let texel: vec4<f32> = textureLoad(traced, vec2<i32>(at), 0);
    let shown: vec2<f32> = min(at * ratio, frame_last);
    let stored: f32 = textureLoad(depth, vec2<i32>(shown), 0);
    let there: f32 = -camera_view_position(shown + 0.5, max(stored, 1e-7)).z;
    let bilinear: f32 = mix(1.0 - fraction.x, fraction.x, offset.x) * mix(1.0 - fraction.y, fraction.y, offset.y);
    let difference: f32 = abs(there - distance) / max(distance, 1e-3);
    let weight: f32 = select(0.0, bilinear / (difference * difference + 1e-3), texel.a >= 0.0 && stored > 0.0);

    sum += texel * weight;
    weights += weight;
  }

  if (weights <= 0.0) {
    return vec4<f32>(0.0, 0.0, 0.0, -1.0);
  }

  return sum / weights;
}
