// The screen's occlusion, searched at half resolution as visibility then distance along the view.

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
