#import "common/contact_march"
#import "frame/light_selection"
#import "generated/frame/light_shadowing"

// How much of each of its shadowed lights a pixel sees, before the lights pass lights it: a light's shadow from the
// atlas, as `shadow_hw` or `shadow_pcss` compares it, and for the strongest few its contact shadow, what
// `common/contact_march` meets on the way to it, stopping short of it. Each light `frame/light_selection` gives a slot
// writes its share there, so the lights pass, which runs over every light reaching every pixel, carries none of the
// shadows' work or registers.

// Whether this pipeline marches contact shadows at all.
override CONTACT_MARCHED: bool = true;

// Metres short of a light a contact shadow's ray stops, so the lamp's own glass and housing around it hide nothing.
const CONTACT_CLEARANCE: f32 = 0.2;

// How much a contact hit weakens in the back half of the thickness: wholly, so a ray passing behind a thin board's
// silhouette (a table's back edge beside the wall) is not taken to be inside it.
const CONTACT_FALLOFF: f32 = 1.0;

// How many times the sun's spacing at its full reach a ray towards a light spaces its reads: near the camera half the
// sun's reads, and fewer as the span shortens.
const CONTACT_SPACING: f32 = 2.0;

@compute @workgroup_size(8, 8)
fn cs_visibility(@builtin(global_invocation_id) id: vec3<u32>) {
  if (any(id.xy >= vec2<u32>(camera.viewport.xy))) {
    return;
  }

  let texel: vec2<i32> = vec2<i32>(id.xy);
  let clip: vec2<f32> = vec2<f32>(id.xy) + 0.5;
  let depth: f32 = textureLoad(depth_target, texel, 0);

  // Nothing drawn there, nothing the lights pass lights.
  if (depth <= 0.0 || lights.count == 0u) {
    return;
  }

  let pixel: LitPixel = lit_pixel(clip, texel, depth, select(0u, contact.lights, CONTACT_MARCHED));
  let step_pixels: f32 = contact.reach / f32(max(contact.steps, 1u)) * CONTACT_SPACING;
  var seen: array<f32, VISIBILITY_SLOTS> = array<f32, VISIBILITY_SLOTS>(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
  var slot: u32 = 0u;
  var marched: u32 = 0u;
  var plane_normal: vec3<f32> = vec3<f32>(0.0);

  for (var index: u32 = 0u; index < counts[pixel.cluster] && slot < VISIBILITY_SLOTS; index++) {
    let record: LightRecord = records[items[pixel.cluster * LIGHT_CLUSTER_CAPACITY + index]];
    let light: ReachingLight = reaching_light(pixel, record);

    if (!is_reaching(record, light) || !takes_slot(pixel, light)) {
      continue;
    }

    var visible: f32 = 1.0;

    if (light.is_shadowed) {
      visible = light_shadow(shadow_atlas, shadow_sampler, record, light.to_point, pixel.normal, record.axis.w > -1.0,
        lights.shadow_filter == 1u);
    }

    // Among the strongest at the pixel and still reaching it past its shadow: marched towards, stopping short of it.
    if (marched < pixel.contact_count && visible > 0.0 && is_contact_marched(pixel, light)) {
      let to_light_point: vec3<f32> = record.position.xyz - pixel.position;
      let reach: f32 = length(to_light_point);

      if (marched == 0u) {
        plane_normal = contact_plane_normal(depth_target, texel, clip, pixel.position);
      }

      marched++;
      visible *= contact_lit(depth_target, contact, clip, pixel.position, plane_normal, to_light_point / reach,
        min(contact.length, reach - CONTACT_CLEARANCE), CONTACT_FALLOFF, step_pixels);
    }

    seen[slot] = visible;
    slot++;
  }

  if (slot > 0u) {
    textureStore(light_visibility_out, texel, 0, vec4<f32>(seen[0], seen[1], seen[2], seen[3]));
  }

  if (slot > VISIBILITY_LAYER_SLOTS) {
    textureStore(light_visibility_out, texel, 1, vec4<f32>(seen[4], seen[5], seen[6], seen[7]));
  }
}
