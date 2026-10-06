#import "common/camera"
#import "common/octahedral"

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

// The frame's texel a traced pixel stands for, its block `ratio` texels a side: the first of it that shows anything
// but a plant, which a reflection is never traced from, so a blade over a puddle does not leave its block untraced;
// the cell it starts from where all are plants or none shows anything. It starts from a different cell each frame
// (`turn`), three cells on a time, so every cell of a block of four is traced in turn and the full-size history
// rebuilds the frame's resolution from them.
fn reflection_texel(material: texture_2d<f32>, depth: texture_depth_2d, pixel: vec2<f32>, ratio: f32, turn: u32)
  -> vec2<i32> {
  let last: vec2<i32> = vec2<i32>(camera.viewport.xy) - 1;
  let first: vec2<i32> = min(vec2<i32>(pixel * ratio), last);
  let side: i32 = i32(ratio);
  let cells: i32 = side * side;
  let start: i32 = i32((turn * 3u) % u32(cells));

  for (var step: i32 = 0; step < cells; step++) {
    let index: i32 = (start + step) % cells;
    let texel: vec2<i32> = min(first + vec2<i32>(index % side, index / side), last);

    if (textureLoad(depth, texel, 0) > 0.0 && !has_mark(textureLoad(material, texel, 0).a, MARK_PLANT)) {
      return texel;
    }
  }

  return min(first + vec2<i32>(start % side, start / side), last);
}

// How far apart two surfaces' normals may face and their roughnesses stand before a traced pixel stops speaking for a
// frame's pixel: the normals' cosine raised to `UPSAMPLE_NORMAL_POWER`, the roughness by `UPSAMPLE_ROUGHNESS_SCALE` a
// unit of difference.
const UPSAMPLE_NORMAL_POWER: f32 = 8.0;
const UPSAMPLE_ROUGHNESS_SCALE: f32 = 20.0;

// The reflections traced brought up to the frame's pixel: the traced pixels around it, each standing for `ratio` of the
// frame's pixels a side, by how near each lies, how near the distance its pixel shows lies to the pixel's (`surface`'s
// `w`), how alike its normal faces to the pixel's (`surface`'s `xyz`, view space) and its roughness stands to
// `roughness`, its own read from `normals` and from the gloss in `albedo`'s alpha as the reflections read gloss, so a
// traced texel neither bleeds onto a rougher neighbour nor across an edge. What is reflected, then one; alpha below none where no pixel around it was
// traced.
fn upsampled_reflection(
  traced: texture_2d<f32>,
  depth: texture_depth_2d,
  material: texture_2d<f32>,
  normals: texture_2d<f32>,
  albedo: texture_2d<f32>,
  pixel: vec2<f32>,
  surface: vec4<f32>,
  roughness: f32,
  ratio: f32,
  turn: u32,
) -> vec4<f32> {
  let distance: f32 = surface.w;
  let last: vec2<f32> = vec2<f32>(textureDimensions(traced)) - 1.0;
  let base: vec2<f32> = floor(pixel / ratio);
  let fraction: vec2<f32> = (pixel - base * ratio) / ratio;
  var sum: vec4<f32> = vec4<f32>(0.0);
  var weights: f32 = 0.0;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let offset: vec2<f32> = vec2<f32>(f32(corner & 1u), f32(corner >> 1u));
    let at: vec2<f32> = clamp(base + offset, vec2<f32>(0.0), last);
    let texel: vec4<f32> = textureLoad(traced, vec2<i32>(at), 0);
    let shown: vec2<i32> = reflection_texel(material, depth, at, ratio, turn);
    let stored: f32 = textureLoad(depth, shown, 0);
    let there: f32 = -camera_view_position(vec2<f32>(shown) + 0.5, max(stored, 1e-7)).z;
    let bilinear: f32 = mix(1.0 - fraction.x, fraction.x, offset.x) * mix(1.0 - fraction.y, fraction.y, offset.y);
    let difference: f32 = abs(there - distance) / max(distance, 1e-3);
    let facing: f32 = pow(saturate(dot(octahedral_decode(textureLoad(normals, shown, 0).xy), surface.xyz)),
      UPSAMPLE_NORMAL_POWER);
    let glossy: f32 = 1.0 - sqrt(saturate(textureLoad(albedo, shown, 0).a));
    let alike: f32 = 1.0 / (1.0 + UPSAMPLE_ROUGHNESS_SCALE * abs(glossy * glossy - roughness));
    let weight: f32 = select(0.0, bilinear * facing * alike / (difference * difference + 1e-3),
      texel.a >= 0.0 && stored > 0.0);

    sum += texel * weight;
    weights += weight;
  }

  if (weights <= 1e-6) {
    return vec4<f32>(0.0, 0.0, 0.0, -1.0);
  }

  return sum / weights;
}
