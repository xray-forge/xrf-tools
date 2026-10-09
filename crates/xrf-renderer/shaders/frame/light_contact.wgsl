#import "common/contact_march"
#import "frame/light_selection"
#import "generated/frame/light_contact"

// Contact shadows towards the lights at half the frame's size: from the first pixel of each two by two, a ray towards
// each of the strongest lights reaching it, marched over the frame's depth as `common/contact_march` marches it and
// stopping short of the light. Each march takes the next slot, the light's record in its high half and the share of
// the light it keeps in its low; the visibility pass brings them up to its own pixels by depth, light by light. The
// weight a light must reach to be among the strongest there is kept beside them, which the visibility and the lights
// find the marched lights by at the frame's size, so the strongest are sorted once.

// Metres short of a light a contact shadow's ray stops, so the lamp's own glass and housing around it hide nothing.
const CONTACT_CLEARANCE: f32 = 0.2;

// How much a contact hit weakens in the back half of the thickness: wholly, so a ray passing behind a thin board's
// silhouette (a table's back edge beside the wall) is not taken to be inside it.
const CONTACT_FALLOFF: f32 = 1.0;

// How many times the sun's spacing at its full reach a ray towards a light spaces its reads: near the camera half the
// sun's reads, and fewer as the span shortens.
const CONTACT_SPACING: f32 = 2.0;

@compute @workgroup_size(8, 8)
fn cs_light_contact(@builtin(global_invocation_id) id: vec3<u32>) {
  if (any(id.xy >= textureDimensions(light_contact_out))) {
    return;
  }

  let texel: vec2<i32> = contact_texel(vec2<i32>(id.xy));
  let clip: vec2<f32> = vec2<f32>(texel) + 0.5;
  let depth: f32 = textureLoad(depth_target, texel, 0);
  var slots: array<u32, MAX_CONTACT_LIGHTS> = array<u32, MAX_CONTACT_LIGHTS>(NO_CONTACT, NO_CONTACT, NO_CONTACT,
    NO_CONTACT, NO_CONTACT, NO_CONTACT, NO_CONTACT, NO_CONTACT);
  var threshold: f32 = 0.0;

  if (depth > 0.0 && lights.count > 0u) {
    let pixel: LitPixel = lit_pixel(clip, texel, depth, contact.lights);
    threshold = pixel.threshold;
    let step_pixels: f32 = contact.reach / f32(max(contact.steps, 1u)) * CONTACT_SPACING;
    var marched: u32 = 0u;
    var plane_normal: vec3<f32> = vec3<f32>(0.0);

    for (var index: u32 = 0u; index < counts[pixel.cluster] && marched < pixel.contact_count; index++) {
      let item: u32 = items[pixel.cluster * LIGHT_CLUSTER_CAPACITY + index];
      let record: LightRecord = records[item];
      let light: ReachingLight = reaching_light(pixel, record);

      if (!is_reaching(record, light) || !is_contact_marched(pixel, light)) {
        continue;
      }

      let to_light_point: vec3<f32> = record.position.xyz - pixel.position;
      let reach: f32 = length(to_light_point);

      if (marched == 0u) {
        plane_normal = contact_plane_normal(depth_target, texel, clip, pixel.position);
      }

      let lit: f32 = contact_lit(depth_target, contact, clip, pixel.position, plane_normal, to_light_point / reach,
        min(contact.length, reach - CONTACT_CLEARANCE), CONTACT_FALLOFF, step_pixels);

      slots[marched] = contact_slot(item, lit);
      marched++;
    }
  }

  textureStore(light_threshold_out, id.xy, vec4<f32>(threshold));
  textureStore(light_contact_out, id.xy, 0, vec4<u32>(slots[0], slots[1], slots[2], slots[3]));

  if (textureNumLayers(light_contact_out) > 1u) {
    textureStore(light_contact_out, id.xy, 1, vec4<u32>(slots[4], slots[5], slots[6], slots[7]));
  }
}
