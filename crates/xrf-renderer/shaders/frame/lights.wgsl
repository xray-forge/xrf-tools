enable wgpu_binding_array;

#import "common/camera"
#import "common/octahedral"
#import "common/light_clusters"
#import "common/fullscreen"
#import "common/contact_march"

// Every local light reaching a pixel, as the engine's `accum_omni` and `accum_spot` accumulate each:
// `Ldynamic_color * plight_local(m, P, N) * lightmap * shadow`, a spot's `lightmap` its projector where the pixel
// stands in its cone, and the shadow of a light that casts one. Diffuse in colour, specular in alpha, added to what the
// sun accumulated. While contact shadows are marched towards lights, the strongest few at each pixel are also
// shadowed by what `common/contact_march` meets on the way to them, shadow map or not.

#import "generated/frame/lights"

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var texture_sampler: sampler;

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

// Whether this pipeline marches contact shadows at all: one that does not leaves the march out, and its registers.
override CONTACT_MARCHED: bool = true;

// Lights a pixel marches contact shadows towards at most, as `RENDER_MAX_CONTACT_SHADOW_LIGHTS` caps them.
const MAX_CONTACT_LIGHTS: u32 = 8u;

// Metres short of a light a contact shadow's ray stops, so the lamp's own glass and housing around it hide nothing.
const CONTACT_CLEARANCE: f32 = 0.2;

// How much a contact hit weakens in the back half of the thickness: wholly, so a ray passing behind a thin board's
// silhouette (a table's back edge beside the wall) is not taken to be inside it.
const CONTACT_FALLOFF: f32 = 1.0;

// How many times the sun's spacing at its full reach a ray towards a light spaces its reads: near the camera half the
// sun's reads, and fewer as the span shortens.
const CONTACT_SPACING: f32 = 2.0;

// How much a light gives a point, as the contact shadows rank the lights by: its falloff, its colour's luminance and
// the surface's facing of it, none outside a spot's cone or on a surface facing away.
fn contact_weight(record: LightRecord, position: vec3<f32>, normal: vec3<f32>) -> f32 {
  let to_point: vec3<f32> = position - record.position.xyz;
  let reach: f32 = length(to_point);
  let falloff: f32 = saturate(1.0 - dot(to_point, to_point) * record.position.w);
  let facing: f32 = saturate(-dot(normal, to_point) / max(reach, 1e-4));
  let along: f32 = dot(to_point, record.axis.xyz);
  let is_lit: bool = record.axis.w <= -1.0 || (along > 0.0 && along >= record.axis.w * reach);
  let luminance: f32 = max(dot(record.color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722)), 0.0);

  return select(0.0, falloff * luminance * facing, is_lit);
}

// The weight a light must reach at a point to be among the `count` strongest of its cluster's lights; zero where the
// cluster holds no more than that, so any light giving the point something is.
fn contact_threshold(cluster: u32, count: u32, position: vec3<f32>, normal: vec3<f32>) -> f32 {
  let held: u32 = counts[cluster];

  if (held <= count) {
    return 0.0;
  }

  // Strongest first: a heavier weight takes each place it beats and carries the one it took on down.
  var strongest: array<f32, MAX_CONTACT_LIGHTS> = array<f32, MAX_CONTACT_LIGHTS>();

  for (var index: u32 = 0u; index < held; index++) {
    var weight: f32 = contact_weight(records[items[cluster * LIGHT_CLUSTER_CAPACITY + index]], position, normal);

    for (var rank: u32 = 0u; rank < MAX_CONTACT_LIGHTS; rank++) {
      let kept: f32 = strongest[rank];
      let is_heavier: bool = weight > kept;

      strongest[rank] = select(kept, weight, is_heavier);
      weight = select(weight, kept, is_heavier);
    }
  }

  return strongest[count - 1u];
}

// A comparison filtered as hardware filters it, between the four texels around a point: lit where the stored depth,
// reversed, is no nearer than the reference.
fn compared_texels(at: vec2<f32>, reference: f32) -> f32 {
  let corner: vec2<f32> = at - 0.5;
  let first: vec2<f32> = floor(corner);
  let blend: vec2<f32> = corner - first;
  let base: vec2<i32> = vec2<i32>(first);
  let lit00: f32 = step(textureLoad(shadow_atlas, base, 0), reference);
  let lit10: f32 = step(textureLoad(shadow_atlas, base + vec2<i32>(1, 0), 0), reference);
  let lit01: f32 = step(textureLoad(shadow_atlas, base + vec2<i32>(0, 1), 0), reference);
  let lit11: f32 = step(textureLoad(shadow_atlas, base + vec2<i32>(1, 1), 0), reference);

  return mix(mix(lit00, lit10, blend.x), mix(lit01, lit11, blend.x), blend.y);
}

// Anomaly's `shadow_pcss`: nine texels five apart searched for what stands nearer the light, lit where none does and
// dark where all do; between, a penumbra of comparisons as wide as the blockers stand from the point.
fn penumbra_lit(centre: vec2<f32>, least: vec2<f32>, most: vec2<f32>, depth: f32) -> f32 {
  let texel_centre: vec2<f32> = floor(centre) + 0.5;
  let reference: f32 = 1.0 - depth;
  var found: f32 = 0.0;
  var blockers: f32 = 0.0;

  for (var row: i32 = -1; row <= 1; row++) {
    for (var column: i32 = -1; column <= 1; column++) {
      let at: vec2<f32> = clamp(texel_centre + vec2<f32>(f32(column), f32(row)) * PCSS_PIXEL, least, most);
      // Held reversed: the engine's own depth is its complement.
      let stored: f32 = 1.0 - textureLoad(shadow_atlas, vec2<i32>(floor(at)), 0);
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
    total += compared_texels(clamp(centre + POISSON_DISK[index] * radius, least, most), reference);
  }

  return total / 12.0;
}

// How much of a shadowed light reaches a point, as `shadow_hw` finds it: the face the point stands in, its depth there
// in the engine's own depth moved by its scale and bias, and its comparison filtered. A point light's face is its
// direction's longest axis, its faces standing in the world; a spot has one, in view space.
fn light_shadow(record: LightRecord, to_point: vec3<f32>, normal: vec3<f32>, is_spot: bool) -> f32 {
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

  let rect: vec4<f32> = record.faces[face];
  let side: f32 = rect.z / ATLAS_TEXEL;
  // A texel of the face across, in metres where the point stands: the face's `2 / scale` of its depth over its texels.
  let reach: f32 = dot(point, axis) * 2.0 / (scale * (side - 2.0));
  let shifted: vec3<f32> = point + bent * reach * NORMAL_OFFSET;
  let across: vec2<f32> = vec2<f32>(dot(shifted, right), dot(shifted, up));
  let along: f32 = dot(shifted, axis);
  let near: f32 = record.shadow.x;
  let far: f32 = record.shadow.y;
  let depth: f32 = max(along, near);
  let is_soft: bool = lights.shadow_filter == 1u;
  // The face's own depth as the engine stores it, `0` near and `1` far, moved as `m_TexelAdjust` moves it; the atlas
  // holds depth reversed, `1` near.
  let moved: f32 = far * (depth - near) / (depth * (far - near)) * DEPTH_SCALE + select(ENGINE_BIAS, SOFT_BIAS, is_soft);
  let uv: vec2<f32> = face_uv(across, along, scale);
  // In texels of the atlas: the face maps into its square a texel in; each tap is kept inside the square.
  let corner: vec2<f32> = rect.xy / ATLAS_TEXEL;
  let least: vec2<f32> = corner + 0.5;
  let most: vec2<f32> = corner + side - 0.5;
  let centre: vec2<f32> = corner + 1.0 + uv * (side - 2.0);

  // A point's faces fade each on its own, as the engine's omni parts do.
  if (is_soft) {
    return penumbra_lit(centre, least, most, moved) * rect.w;
  }

  let reference: f32 = 1.0 - moved;
  var lit: f32 = 0.0;

  for (var tap: u32 = 0u; tap < 4u; tap++) {
    let offset: vec2<f32> = vec2<f32>(f32(tap & 1u) * 2.0 - 1.0, f32(tap >> 1u) * 2.0 - 1.0) * SHADOW_KERNEL;

    lit += compared_texels(clamp(centre + offset, least, most), reference);
  }

  return lit / 4.0 * rect.w;
}

// `m_Lmap`: a point in a light's view, `x` right, `y` up, `along` its direction, over the square the widened cone
// covers, `v` down.
fn face_uv(across: vec2<f32>, along: f32, scale: f32) -> vec2<f32> {
  return vec2<f32>(0.5) + across * scale / along * vec2<f32>(0.5, -0.5);
}

@fragment
fn fs_lights(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  // Nothing drawn there, nothing to light: its depth rebuilds no point.
  if (depth <= 0.0 || lights.count == 0u) {
    return vec4<f32>(0.0);
  }

  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let material_texel: vec4<f32> = textureLoad(material_target, texel, 0);
  let slice: f32 = material_texel.z;
  // Foliage is lit from each light's side, as the light passes through its leaves.
  let is_flora: bool = has_mark(material_texel.a, MARK_FLORA);
  let cluster: u32 = light_cluster(lights, in.clip.xy / camera.viewport.xy, -position.z);
  let to_eye: vec3<f32> = normalize(-position);
  let offset: vec3<f32> = position + normal * VIRTUAL_OFFSET;
  let contact_count: u32 = select(0u, min(contact.lights, MAX_CONTACT_LIGHTS), CONTACT_MARCHED);
  var threshold: f32 = 0.0;
  var marched: u32 = 0u;
  var plane_normal: vec3<f32> = vec3<f32>(0.0);
  let step_pixels: f32 = contact.reach / f32(max(contact.steps, 1u)) * CONTACT_SPACING;
  var total: vec4<f32> = vec4<f32>(0.0);

  if (contact_count > 0u) {
    threshold = contact_threshold(cluster, contact_count, position, normal);
  }

  for (var index: u32 = 0u; index < counts[cluster]; index++) {
    let record: LightRecord = records[items[cluster * LIGHT_CLUSTER_CAPACITY + index]];
    let is_spot: bool = record.axis.w > -1.0;
    let is_shadowed: bool = record.shadow.z > 0.0;
    // `accum_base`, a spot's and a shadowed omni part's, offsets the point; the unshadowed omni shader does not.
    let to_point: vec3<f32> = select(position, offset, is_spot || is_shadowed) - record.position.xyz;
    // `plight_local`: falloff by the squared distance, to zero at 95% of the range.
    let falloff: f32 = saturate(1.0 - dot(to_point, to_point) * record.position.w);

    // Past the light's reach nothing below adds anything.
    if (falloff <= 0.0) {
      continue;
    }

    let to_light: vec3<f32> = normalize(-to_point);
    let half_way: vec3<f32> = normalize(to_light + to_eye);
    // Foliage's normal turns towards the light, halfway for a spot or a shadowed omni, wholly for an unshadowed one,
    // and its gloss is cut as far.
    let is_unshadowed_omni: bool = !(is_spot || is_shadowed);
    let lit_normal: vec3<f32> = select(normal,
      select(normalize(normal + to_light), to_light, is_unshadowed_omni), is_flora);
    let material: vec4<f32> = textureSampleLevel(
      material_lut,
      lut_sampler,
      vec3<f32>(dot(to_light, lit_normal), dot(half_way, lit_normal), slice),
      0.0
    );
    let flora_gloss: f32 = select(1.0, select(0.5, 0.3, is_unshadowed_omni), is_flora);
    var light: vec4<f32> = vec4<f32>(material.xxx, material.y * flora_gloss) * falloff;

    if (is_spot) {
      let along: f32 = dot(to_point, record.axis.xyz);

      // In front of the apex and within the cone; the rest of a spot's reach stays dark.
      if (along <= 0.0 || along < record.axis.w * length(to_point)) {
        continue;
      }

      if (record.up.w >= 0.0) {
        let across: vec2<f32> = vec2<f32>(dot(to_point, record.right.xyz), dot(to_point, record.up.xyz));
        let slot: u32 = u32(record.up.w);
        // Half a texel inside its edges, as a clamped sampler keeps it: the array's sampler repeats.
        let edge: vec2<f32> = 0.5 / vec2<f32>(textureDimensions(textures[slot]));
        let uv: vec2<f32> = clamp(face_uv(across, along, record.right.w), edge, 1.0 - edge);

        light *= textureSampleLevel(textures[slot], texture_sampler, uv, 0.0);
      }
    }

    if (is_shadowed) {
      light *= light_shadow(record, to_point, normal, is_spot);
    }

    // Among the strongest at the pixel and still reaching it: marched towards, stopping short of the light.
    if (marched < contact_count && any(light > vec4<f32>(0.0))) {
      let weight: f32 = contact_weight(record, position, normal);

      if (weight > 0.0 && weight >= threshold) {
        let to_light_point: vec3<f32> = record.position.xyz - position;
        let reach: f32 = length(to_light_point);

        if (marched == 0u) {
          plane_normal = contact_plane_normal(depth_target, texel, in.clip.xy, position);
        }

        marched++;
        light *= contact_lit(
          depth_target,
          contact,
          in.clip.xy,
          position,
          plane_normal,
          to_light_point / reach,
          min(contact.length, reach - CONTACT_CLEARANCE),
          CONTACT_FALLOFF,
          step_pixels
        );
      }
    }

    total += record.color * light;
  }

  return total;
}
