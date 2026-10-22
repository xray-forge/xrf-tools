enable wgpu_binding_array;

#import "common/fullscreen"
#import "frame/light_selection"
#import "common/scene_light"
#import "generated/frame/light_visibility"

// Every local light reaching a pixel, as the engine's `accum_omni` and `accum_spot` accumulate each:
// `Ldynamic_color * plight_local(m, P, N) * lightmap * shadow`, a spot's `lightmap` its projector where the pixel
// stands in its cone, and the shadow of a light that casts one. Diffuse in colour, specular in alpha, added to what the
// sun accumulated. A shadowed light's shadow, and while contact shadows are marched towards lights the strongest few's
// contact shadow, are what `frame/light_visibility` found in the pixel's slot of that light.

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var texture_sampler: sampler;

// Whether contact shadows are marched towards the lights, which the slots are found by.
override CONTACT_MARCHED: bool = true;

// The share of a light a pixel sees, in the slot the visibility pass wrote it to.
fn slot_visibility(texel: vec2<i32>, slot: u32) -> f32 {
  return textureLoad(light_visibility, texel, slot / VISIBILITY_LAYER_SLOTS, 0)[slot % VISIBILITY_LAYER_SLOTS];
}

@fragment
fn fs_lights(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  let view: u32 = light_view();

  // Nothing drawn there, nothing to light: its depth rebuilds no point. A pixel no light adds to is left out rather
  // than added nought to, sparing the blend reading and writing the light it already holds; and the cascades' and the
  // overdraw's views keep the lamps out of what they show.
  if (depth <= 0.0 || lights.count == 0u || view == LIGHT_VIEW_CASCADES || view == LIGHT_VIEW_OVERDRAW) {
    discard;
  }

  let threshold: f32 = textureLoad(light_threshold, threshold_texel(texel, textureDimensions(light_threshold)), 0).r;
  let pixel: LitPixel = lit_pixel_by(in.clip.xy, texel, depth, select(0u, contact.lights, CONTACT_MARCHED),
    threshold);

  // The lamps' view: how many lamps the pixel's cluster lists, which the sun left nought under.
  if (view == LIGHT_VIEW_LAMPS) {
    return vec4<f32>(f32(counts[pixel.cluster]), 0.0, 0.0, 0.0);
  }
  let material_texel: vec4<f32> = textureLoad(material_target, texel, 0);
  let slice: f32 = material_texel.z;
  // Foliage is lit from each light's side, as the light passes through its leaves.
  let is_flora: bool = has_mark(material_texel.a, MARK_FLORA);
  let to_eye: vec3<f32> = -camera_view_ray(pixel.position);
  var slot: u32 = 0u;
  var total: vec4<f32> = vec4<f32>(0.0);

  for (var index: u32 = 0u; index < counts[pixel.cluster]; index++) {
    let record: LightRecord = records[items[pixel.cluster * LIGHT_CLUSTER_CAPACITY + index]];
    let light: ReachingLight = reaching_light(pixel, record);

    // Past the light's reach, and a spot's behind its apex and outside its cone, nothing is added.
    if (!is_reaching(record, light)) {
      continue;
    }

    let is_spot: bool = record.axis.w > -1.0;
    let to_light: vec3<f32> = normalize(-light.to_point);
    let half_way: vec3<f32> = normalize(to_light + to_eye);
    // Foliage's normal turns towards the light, halfway for a spot or a shadowed omni, wholly for an unshadowed one,
    // and its gloss is cut as far.
    let is_unshadowed_omni: bool = !(is_spot || light.is_shadowed);
    let lit_normal: vec3<f32> = select(pixel.normal,
      select(normalize(pixel.normal + to_light), to_light, is_unshadowed_omni), is_flora);
    let material: vec4<f32> = textureSampleLevel(
      material_lut,
      lut_sampler,
      vec3<f32>(dot(to_light, lit_normal), dot(half_way, lit_normal), slice),
      0.0
    );
    let flora_gloss: f32 = select(1.0, select(0.5, 0.3, is_unshadowed_omni), is_flora);
    var lit: vec4<f32> = vec4<f32>(material.xxx, material.y * flora_gloss) *
      light_share(record, light.to_point, are_levels_physical(), 0.0);

    if (is_spot && record.up.w >= 0.0) {
      let projector: u32 = u32(record.up.w);
      let uv: vec2<f32> = projector_uv(record, light.to_point, vec2<f32>(textureDimensions(textures[projector])));

      let projected: vec4<f32> = textureSampleLevel(textures[projector], texture_sampler, uv, 0.0);

      lit *= vec4<f32>(authored_colour(projected.rgb), projected.a);
    }

    if (takes_slot(pixel, light)) {
      if (slot < VISIBILITY_SLOTS) {
        lit *= slot_visibility(texel, slot);
      }

      slot++;
    }

    total += record.color * lit;
  }

  if (all(total == vec4<f32>(0.0))) {
    discard;
  }

  return total;
}
