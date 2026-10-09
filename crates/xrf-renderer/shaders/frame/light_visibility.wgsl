#import "frame/light_selection"
#import "generated/frame/light_shadowing"

// How much of each of its shadowed lights a pixel sees, before the lights pass lights it: a light's shadow from the
// atlas, as `shadow_hw` or `shadow_pcss` compares it, and for the strongest few its contact shadow, marched at half the
// frame's size by `frame/light_contact` and brought up here by depth. Each light `frame/light_selection` gives a slot
// writes its share there, so the lights pass, which runs over every light reaching every pixel, carries none of the
// shadows' work or registers.

// Whether this pipeline reads contact shadows at all.
override CONTACT_MARCHED: bool = true;

// The four marched texels around a pixel, as the light contact pass marched them, and how much each speaks for it: by
// how near it lies, and by how near its distance along the view lies to the pixel's, so an edge never takes the other
// side's; none where nothing was drawn there.
struct ContactNeighbours {
  base: vec2<i32>,
  weights: vec4<f32>,
}

fn contact_neighbours(pixel: vec2<f32>, distance: f32) -> ContactNeighbours {
  let last: vec2<i32> = vec2<i32>(textureDimensions(light_contact)) - 1;
  let base: vec2<f32> = floor(pixel / 2.0);
  let fraction: vec2<f32> = (pixel - base * 2.0) * 0.5;
  var weights: vec4<f32> = vec4<f32>(0.0);

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let offset: vec2<f32> = vec2<f32>(f32(corner & 1u), f32(corner >> 1u));
    let marched: vec2<i32> = contact_texel(min(vec2<i32>(base + offset), last));
    let depth: f32 = textureLoad(depth_target, marched, 0);
    let there: f32 = -camera_view_position(vec2<f32>(marched) + 0.5, max(depth, 1e-7)).z;
    let bilinear: f32 = mix(1.0 - fraction.x, fraction.x, offset.x) * mix(1.0 - fraction.y, fraction.y, offset.y);
    let difference: f32 = abs(there - distance) / max(distance, 1e-3);

    weights[corner] = select(0.0, bilinear / (difference * difference + 1e-3), depth > 0.0);
  }

  return ContactNeighbours(vec2<i32>(base), weights);
}

// The share of the light at record `item` a pixel keeps past what its contact shadow meets: the marches towards it of
// the texels around the pixel, by how much each speaks for it; all where none of them marched towards it.
fn contact_seen(neighbours: ContactNeighbours, item: u32, layers: u32) -> f32 {
  let last: vec2<i32> = vec2<i32>(textureDimensions(light_contact)) - 1;
  var sum: f32 = 0.0;
  var weights: f32 = 0.0;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let weight: f32 = neighbours.weights[corner];

    if (weight <= 0.0) {
      continue;
    }

    let at: vec2<i32> = min(neighbours.base + vec2<i32>(i32(corner & 1u), i32(corner >> 1u)), last);

    for (var layer: u32 = 0u; layer < layers; layer++) {
      let slots: vec4<u32> = textureLoad(light_contact, at, layer, 0);
      let found: vec4<bool> = (slots >> vec4<u32>(16u)) == vec4<u32>(item);

      if (any(found)) {
        let slot: u32 = dot(select(vec4<u32>(0u), slots & vec4<u32>(0xffffu), found), vec4<u32>(1u));

        sum += f32(slot) / 65535.0 * weight;
        weights += weight;
        break;
      }
    }
  }

  return select(1.0, sum / weights, weights > 0.0);
}

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

  let threshold: f32 = textureLoad(light_threshold, threshold_texel(texel, textureDimensions(light_threshold)), 0).r;
  let pixel: LitPixel = lit_pixel_by(clip, texel, depth, select(0u, contact.lights, CONTACT_MARCHED), threshold);
  let layers: u32 = textureNumLayers(light_contact);
  var seen: array<f32, VISIBILITY_SLOTS> = array<f32, VISIBILITY_SLOTS>(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
  var slot: u32 = 0u;
  var marched: u32 = 0u;
  var neighbours: ContactNeighbours;

  for (var index: u32 = 0u; index < counts[pixel.cluster] && slot < VISIBILITY_SLOTS; index++) {
    let item: u32 = items[pixel.cluster * LIGHT_CLUSTER_CAPACITY + index];
    let record: LightRecord = records[item];
    let light: ReachingLight = reaching_light(pixel, record);

    if (!is_reaching(record, light) || !takes_slot(pixel, light)) {
      continue;
    }

    var visible: f32 = 1.0;

    if (light.is_shadowed) {
      visible = light_shadow(shadow_atlas, shadow_sampler, record, light.to_point, pixel.normal, record.axis.w > -1.0,
        lights.shadow_filter == 1u);
    }

    // Among the strongest at the pixel and still reaching it past its shadow: what its march met.
    if (marched < pixel.contact_count && visible > 0.0 && is_contact_marched(pixel, light)) {
      if (marched == 0u) {
        neighbours = contact_neighbours(floor(clip), -pixel.position.z);
      }

      marched++;
      visible *= contact_seen(neighbours, item, layers);
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
