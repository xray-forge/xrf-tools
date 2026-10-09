#import "common/camera"
#import "common/octahedral"
#import "common/light_clusters"
#import "common/local_lights"
#import "generated/frame/lights"

// Which of a pixel's lights are shadowed, found alike by the visibility pass that shadows them and by the lights pass
// that reads what it found: each light of the pixel's cluster, in the cluster's order, that reaches the pixel and
// either casts a shadow or is among the strongest the contact shadows march towards takes the next of the pixel's
// visibility slots, four to a layer of the visibility target.

// Lights a pixel marches contact shadows towards at most, as `RENDER_MAX_CONTACT_SHADOW_LIGHTS` caps them.
const MAX_CONTACT_LIGHTS: u32 = 8u;

// Slots of visibility a pixel holds, and how many a layer of the target holds; a light past the last is unshadowed.
const VISIBILITY_SLOTS: u32 = 8u;
const VISIBILITY_LAYER_SLOTS: u32 = 4u;

// A slot of the lights' contact shadows no light was marched into.
const NO_CONTACT: u32 = 0xffffffffu;

// The frame's texel a texel of the lights' contact shadows, at half the frame's size, was marched from: the first of
// its two by two.
fn contact_texel(marched: vec2<i32>) -> vec2<i32> {
  return min(marched * 2, vec2<i32>(camera.viewport.xy) - 1);
}

// A slot of the lights' contact shadows: the light's record in the high half, the share of it kept in the low.
fn contact_slot(item: u32, lit: f32) -> u32 {
  return (item << 16u) | u32(round(saturate(lit) * 65535.0));
}

// A drawn pixel as its lights are found: where it stands in view space and its normal, its point moved off the surface
// as a spot or a shadowed light reads it, its cluster, the lights its contact shadows march towards at most, and the
// weight a light must reach to be among them.
struct LitPixel {
  position: vec3<f32>,
  normal: vec3<f32>,
  offset: vec3<f32>,
  cluster: u32,
  contact_count: u32,
  threshold: f32,
}

// A light of a pixel's cluster as it reaches the pixel: towards the pixel from it, how much reaches, whether it casts a
// shadow, and its weight for the contact shadows.
struct ReachingLight {
  to_point: vec3<f32>,
  falloff: f32,
  is_shadowed: bool,
  weight: f32,
}

// The pixel at a point of the frame with a depth drawn there, its contact shadows marched towards `contact_lights`
// lights at most, the strongest there.
fn lit_pixel(clip: vec2<f32>, texel: vec2<i32>, depth: f32, contact_lights: u32) -> LitPixel {
  var pixel: LitPixel = lit_pixel_by(clip, texel, depth, contact_lights, 0.0);

  if (pixel.contact_count > 0u) {
    pixel.threshold = contact_threshold(pixel.cluster, pixel.contact_count, pixel.position, pixel.normal);
  }

  return pixel;
}

// The pixel at a point of the frame with a depth drawn there, its contact shadows marched towards `contact_lights`
// lights at most, each reaching the weight `threshold` the light contact pass found beside it.
fn lit_pixel_by(clip: vec2<f32>, texel: vec2<i32>, depth: f32, contact_lights: u32, threshold: f32) -> LitPixel {
  let position: vec3<f32> = camera_view_position(clip, depth);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let cluster: u32 = light_cluster(lights, clip / camera.viewport.xy, -position.z);

  return LitPixel(position, normal, position + normal * VIRTUAL_OFFSET, cluster,
    min(contact_lights, MAX_CONTACT_LIGHTS), threshold);
}

// The texel of the lights' contact shadows, at half the frame's size, whose weight a frame's texel takes.
fn threshold_texel(texel: vec2<i32>, size: vec2<u32>) -> vec2<i32> {
  return min(texel / 2, vec2<i32>(size) - 1);
}

// A light of a pixel's cluster as it reaches the pixel.
fn reaching_light(pixel: LitPixel, record: LightRecord) -> ReachingLight {
  let is_spot: bool = record.axis.w > -1.0;
  let is_shadowed: bool = record.shadow.z > 0.0;
  // `accum_base`, a spot's and a shadowed omni part's, offsets the point; the unshadowed omni shader does not.
  let to_point: vec3<f32> = select(pixel.position, pixel.offset, is_spot || is_shadowed) - record.position.xyz;
  let weight: f32 = select(0.0, contact_weight(record, pixel.position, pixel.normal), pixel.contact_count > 0u);

  return ReachingLight(to_point, light_falloff(record, to_point), is_shadowed, weight);
}

// Whether a light reaching a pixel lights it at all: within its reach, and inside a spot's cone.
fn is_reaching(record: LightRecord, light: ReachingLight) -> bool {
  return light.falloff > 0.0 && !is_outside_cone(record, light.to_point);
}

// Whether a light reaching a pixel takes a visibility slot: it casts a shadow, or the contact shadows may march to it.
fn takes_slot(pixel: LitPixel, light: ReachingLight) -> bool {
  return light.is_shadowed || is_contact_marched(pixel, light);
}

// Whether a light is among the strongest at a pixel, which the contact shadows march towards while they have marches
// left.
fn is_contact_marched(pixel: LitPixel, light: ReachingLight) -> bool {
  return light.weight > 0.0 && light.weight >= pixel.threshold;
}

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
