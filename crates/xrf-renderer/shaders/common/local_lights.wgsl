#import "common/camera"
#import "generated/structs"

// A local light as it lights a point in view space, shared by the lights pass and a traced hit: its falloff, a spot's
// cone and projector, and a shadowed light's map in the shadow atlas, as `pass/light_record.rs` writes its record.

// `gbd.P += gbd.N * 0.015`: the virtual offset `accum_base` moves a point by with the optimised G-buffer.
const VIRTUAL_OFFSET: f32 = 0.015;

// A texel of the shadow atlas, in texture coordinates.
const ATLAS_TEXEL: f32 = 1.0 / 4096.0;

// Texels of its face a point is moved along its normal before it is compared.
const NORMAL_OFFSET: f32 = 1.0;

// `KERNEL`: how far `shadow_hw`'s four taps stand from the point, in texels of the atlas.
const SHADOW_KERNEL: f32 = 0.6;

// `r2_ls_depth_scale`, and each filter's `r2_ls_depth_bias`: what the point's depth is moved by before it is compared.
const DEPTH_SCALE: f32 = 1.00001;
const ENGINE_BIAS: f32 = -0.0003;
const SOFT_BIAS: f32 = -0.001;

// `PCSS_PIXEL`, `PCSS_PIXEL_MIN` and `PCSS_SUN_WIDTH`: how far the blockers are searched, and the penumbra's scale.
const PCSS_PIXEL: f32 = 5.0;
const PCSS_PIXEL_MIN: f32 = 1.0;
const PCSS_WIDTH: f32 = 150.0;

// Anomaly's `poissonDisk`: the first twelve, which its `shadow_pcss` takes at its default quality.
const POISSON_DISK: array<vec2<f32>, 12> = array<vec2<f32>, 12>(
  vec2<f32>(0.0617981, 0.07294159),
  vec2<f32>(0.6470215, 0.7474022),
  vec2<f32>(-0.5987766, -0.7512833),
  vec2<f32>(-0.693034, 0.6913887),
  vec2<f32>(0.6987045, -0.6843052),
  vec2<f32>(-0.9402866, 0.04474335),
  vec2<f32>(0.8934509, 0.07369385),
  vec2<f32>(0.1592735, -0.9686295),
  vec2<f32>(-0.05664673, 0.995282),
  vec2<f32>(-0.1203411, -0.1301079),
  vec2<f32>(0.1741608, -0.1682285),
  vec2<f32>(-0.09369049, 0.3196758)
);

// A point light's six faces along the world's axes, each as where it looks, its up, and its right, `direction x up`.
const POINT_DIRECTIONS: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
  vec3<f32>(1.0, 0.0, 0.0),
  vec3<f32>(-1.0, 0.0, 0.0),
  vec3<f32>(0.0, 1.0, 0.0),
  vec3<f32>(0.0, -1.0, 0.0),
  vec3<f32>(0.0, 0.0, 1.0),
  vec3<f32>(0.0, 0.0, -1.0)
);
const POINT_UPS: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
  vec3<f32>(0.0, 1.0, 0.0),
  vec3<f32>(0.0, 1.0, 0.0),
  vec3<f32>(0.0, 0.0, -1.0),
  vec3<f32>(0.0, 0.0, 1.0),
  vec3<f32>(0.0, 1.0, 0.0),
  vec3<f32>(0.0, 1.0, 0.0)
);
const POINT_RIGHTS: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
  vec3<f32>(0.0, 0.0, 1.0),
  vec3<f32>(0.0, 0.0, -1.0),
  vec3<f32>(-1.0, 0.0, 0.0),
  vec3<f32>(-1.0, 0.0, 0.0),
  vec3<f32>(-1.0, 0.0, 0.0),
  vec3<f32>(1.0, 0.0, 0.0)
);

// `cot` of half an omni part's widened cone: a quarter turn and `tan_shift`.
const POINT_SCALE: f32 = 0.94070607;

// `plight_local`'s falloff: by the squared distance from the light, to none at 95% of its range.
fn light_falloff(record: LightRecord, to_point: vec3<f32>) -> f32 {
  return saturate(1.0 - dot(to_point, to_point) * record.position.w);
}

// Whether a point stands outside a spot's cone or behind its apex; never for a light without a cone.
fn is_outside_cone(record: LightRecord, to_point: vec3<f32>) -> bool {
  let along: f32 = dot(to_point, record.axis.xyz);

  return record.axis.w > -1.0 && (along <= 0.0 || along < record.axis.w * length(to_point));
}

// Where a point within a spot's cone reads its projector, half a texel of a `size` texture inside its edges, as a
// clamped sampler keeps it: the bindless array's sampler repeats.
fn projector_uv(record: LightRecord, to_point: vec3<f32>, size: vec2<f32>) -> vec2<f32> {
  let across: vec2<f32> = vec2<f32>(dot(to_point, record.right.xyz), dot(to_point, record.up.xyz));
  let edge: vec2<f32> = 0.5 / size;

  return clamp(face_uv(across, dot(to_point, record.axis.xyz), record.right.w), edge, 1.0 - edge);
}

// A comparison filtered as hardware filters it, between the four texels around a point: lit where the stored depth,
// reversed, is no nearer than the reference.
fn compared_texels(atlas: texture_depth_2d, compare: sampler_comparison, at: vec2<f32>, reference: f32) -> f32 {
  return textureSampleCompareLevel(atlas, compare, at * ATLAS_TEXEL, reference);
}

// Anomaly's `shadow_pcss`: nine texels five apart searched for what stands nearer the light, lit where none does and
// dark where all do; between, a penumbra of comparisons as wide as the blockers stand from the point.
fn penumbra_lit(
  atlas: texture_depth_2d,
  compare: sampler_comparison,
  centre: vec2<f32>,
  least: vec2<f32>,
  most: vec2<f32>,
  depth: f32,
) -> f32 {
  let texel_centre: vec2<f32> = floor(centre) + 0.5;
  let reference: f32 = 1.0 - depth;
  var found: f32 = 0.0;
  var blockers: f32 = 0.0;

  for (var row: i32 = -1; row <= 1; row++) {
    for (var column: i32 = -1; column <= 1; column++) {
      let at: vec2<f32> = clamp(texel_centre + vec2<f32>(f32(column), f32(row)) * PCSS_PIXEL, least, most);
      // Held reversed: the engine's own depth is its complement.
      let stored: f32 = 1.0 - textureLoad(atlas, vec2<i32>(floor(at)), 0);
      let is_blocker: f32 = 1.0 - step(depth - 0.0001, stored);

      blockers += is_blocker;
      found += stored * is_blocker;
    }
  }

  if (blockers >= 9.0) {
    return 0.0;
  }

  if (blockers < 1.0) {
    return 1.0;
  }

  let blocker: f32 = found / blockers;
  let ratio: f32 = saturate((depth - blocker) * PCSS_WIDTH / blocker);
  let radius: f32 = max(PCSS_PIXEL_MIN, ratio * ratio * PCSS_PIXEL);
  var total: f32 = 0.0;

  for (var index: u32 = 0u; index < 12u; index++) {
    total += compared_texels(atlas, compare, clamp(centre + POISSON_DISK[index] * radius, least, most), reference);
  }

  return total / 12.0;
}

// Where a point reads a shadowed light's map: the square of the atlas its face holds, in texels, a texel in at each
// edge (`least` and `most`), where the point falls in it (`centre`), its depth in the engine's own moved by its scale and
// bias (`moved`, `0` near), and how far the face has faded.
struct ShadowPlace {
  centre: vec2<f32>,
  least: vec2<f32>,
  most: vec2<f32>,
  moved: f32,
  fade: f32,
}

// A face's square of the shadow atlas, picked by constant indices: indexing the faces by a runtime value keeps the
// whole record in scratch memory.
fn face_rect(record: LightRecord, face: u32) -> vec4<f32> {
  var rect: vec4<f32> = record.faces[0];

  if (face == 1u) {
    rect = record.faces[1];
  } else if (face == 2u) {
    rect = record.faces[2];
  } else if (face == 3u) {
    rect = record.faces[3];
  } else if (face == 4u) {
    rect = record.faces[4];
  } else if (face == 5u) {
    rect = record.faces[5];
  }

  return rect;
}

// A point's place in a shadowed light's map: the face it stands in, a point light's its direction's longest axis, its
// faces standing in the world, a spot's its one, in view space; moved along its normal by a texel of the face.
fn shadow_place(record: LightRecord, to_point: vec3<f32>, normal: vec3<f32>, is_spot: bool, is_soft: bool) -> ShadowPlace {
  var face: u32 = 0u;
  var right: vec3<f32> = record.right.xyz;
  var up: vec3<f32> = record.up.xyz;
  var axis: vec3<f32> = record.axis.xyz;
  var point: vec3<f32> = to_point;
  var bent: vec3<f32> = normal;
  var scale: f32 = record.right.w;

  if (!is_spot) {
    let to_world: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
    let world: vec3<f32> = to_world * to_point;
    let size: vec3<f32> = abs(world);

    if (size.x >= size.y && size.x >= size.z) {
      face = select(1u, 0u, world.x > 0.0);
    } else if (size.y >= size.z) {
      face = select(3u, 2u, world.y > 0.0);
    } else {
      face = select(5u, 4u, world.z > 0.0);
    }

    right = POINT_RIGHTS[face];
    up = POINT_UPS[face];
    axis = POINT_DIRECTIONS[face];
    point = world;
    bent = to_world * normal;
    scale = POINT_SCALE;
  }

  let rect: vec4<f32> = face_rect(record, face);
  let side: f32 = rect.z / ATLAS_TEXEL;
  // A texel of the face across, in metres where the point stands: the face's `2 / scale` of its depth over its texels.
  let reach: f32 = dot(point, axis) * 2.0 / (scale * (side - 2.0));
  let shifted: vec3<f32> = point + bent * reach * NORMAL_OFFSET;
  let across: vec2<f32> = vec2<f32>(dot(shifted, right), dot(shifted, up));
  let along: f32 = dot(shifted, axis);
  let near: f32 = record.shadow.x;
  let far: f32 = record.shadow.y;
  let depth: f32 = max(along, near);
  // The face's own depth as the engine stores it, `0` near and `1` far, moved as `m_TexelAdjust` moves it; the atlas
  // holds depth reversed, `1` near.
  let moved: f32 = far * (depth - near) / (depth * (far - near)) * DEPTH_SCALE + select(ENGINE_BIAS, SOFT_BIAS, is_soft);
  let uv: vec2<f32> = face_uv(across, along, scale);
  // In texels of the atlas: the face maps into its square a texel in; each tap is kept inside the square.
  let corner: vec2<f32> = rect.xy / ATLAS_TEXEL;

  // A point's faces fade each on its own, as the engine's omni parts do.
  return ShadowPlace(corner + 1.0 + uv * (side - 2.0), corner + 0.5, corner + side - 0.5, moved, rect.w);
}

// `shadow_hw`'s four taps about a point of the atlas, each kept inside its face's square.
fn shadow_tap(place: ShadowPlace, tap: u32) -> vec2<f32> {
  let offset: vec2<f32> = vec2<f32>(f32(tap & 1u) * 2.0 - 1.0, f32(tap >> 1u) * 2.0 - 1.0) * SHADOW_KERNEL;

  return clamp(place.centre + offset, place.least, place.most);
}

// How much of a shadowed light reaches a point, as `shadow_hw` finds it: its place in the light's map (`shadow_place`)
// and its comparison filtered. `is_soft` compares as `shadow_pcss` does.
fn light_shadow(
  atlas: texture_depth_2d,
  compare: sampler_comparison,
  record: LightRecord,
  to_point: vec3<f32>,
  normal: vec3<f32>,
  is_spot: bool,
  is_soft: bool,
) -> f32 {
  let place: ShadowPlace = shadow_place(record, to_point, normal, is_spot, is_soft);

  if (is_soft) {
    return penumbra_lit(atlas, compare, place.centre, place.least, place.most, place.moved) * place.fade;
  }

  let reference: f32 = 1.0 - place.moved;
  var lit: f32 = 0.0;

  for (var tap: u32 = 0u; tap < 4u; tap++) {
    lit += compared_texels(atlas, compare, shadow_tap(place, tap), reference);
  }

  return lit / 4.0 * place.fade;
}

// `m_Lmap`: a point in a light's view, `x` right, `y` up, `along` its direction, over the square the widened cone
// covers, `v` down.
fn face_uv(across: vec2<f32>, along: f32, scale: f32) -> vec2<f32> {
  return vec2<f32>(0.5) + across * scale / along * vec2<f32>(0.5, -0.5);
}
