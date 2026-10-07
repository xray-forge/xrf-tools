enable wgpu_binding_array;

#import "common/camera"
#import "common/octahedral"
#import "common/light_clusters"
#import "common/fullscreen"
#import "common/contact_march"
#import "common/local_lights"

// Every local light reaching a pixel, as the engine's `accum_omni` and `accum_spot` accumulate each:
// `Ldynamic_color * plight_local(m, P, N) * lightmap * shadow`, a spot's `lightmap` its projector where the pixel
// stands in its cone, and the shadow of a light that casts one. Diffuse in colour, specular in alpha, added to what the
// sun accumulated. While contact shadows are marched towards lights, the strongest few at each pixel are also
// shadowed by what `common/contact_march` meets on the way to them, shadow map or not.

#import "generated/frame/lights"

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var texture_sampler: sampler;

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

@fragment
fn fs_lights(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  // Nothing drawn there, nothing to light: its depth rebuilds no point. A pixel no light adds to is left out rather
  // than added nought to, sparing the blend reading and writing the light it already holds.
  if (depth <= 0.0 || lights.count == 0u) {
    discard;
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
    let falloff: f32 = light_falloff(record, to_point);

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

    // The rest of a spot's reach, behind its apex and outside its cone, stays dark.
    if (is_outside_cone(record, to_point)) {
      continue;
    }

    if (is_spot && record.up.w >= 0.0) {
      let slot: u32 = u32(record.up.w);
      let uv: vec2<f32> = projector_uv(record, to_point, vec2<f32>(textureDimensions(textures[slot])));

      light *= textureSampleLevel(textures[slot], texture_sampler, uv, 0.0);
    }

    if (is_shadowed) {
      light *= light_shadow(shadow_atlas, record, to_point, normal, is_spot, lights.shadow_filter == 1u);
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

  if (all(total == vec4<f32>(0.0))) {
    discard;
  }

  return total;
}
