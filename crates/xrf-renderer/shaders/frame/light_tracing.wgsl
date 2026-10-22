#import "frame/light_selection"
#import "common/scene_trace"
#import "common/noise"
#import "common/axis_frame"
#import "generated/frame/light_tracing"
#import "common/math"

// How much of each of its shadowed lights a pixel sees, traced, in the visibility pass's place where DLSS ray
// reconstruction denoises the frame: per light one ray towards a point of its bulb, drawn anew each frame, traced
// against the level's static geometry and its spawned models, cut-out leaves by their alpha, stopping its light's near
// plane short of it as the atlas's faces do, so its own fitting does not shadow it. Each shadowed light takes the slot
// `frame/light_selection` gives it, the contact shadows not marched. The grass near the eye stands in the trace as the
// view's cards where it is shadowed (`GrassCards`).

// Metres a ray starts off its surface along its normal, and how much more each metre away, so it leaves the surface it
// starts on.
const LIFT: f32 = 0.03;
const LIFT_PER_METRE: f32 = 0.002;
// Metres a plant's ray starts towards its light past it, clear of its own leaf's card and its neighbours'.
const PLANT_LIFT: f32 = 0.3;

// Whether a ray from a pixel reaches a point of its light's bulb, `drawn` placing the point: none where something
// solid stands between them.
fn traced_seen(pixel: LitPixel, record: LightRecord, is_plant: bool, drawn: vec2<f32>) -> f32 {
  let to_world: mat4x4<f32> = transpose(camera.view);
  let distance: f32 = -pixel.position.z;
  let point: vec3<f32> = camera.position.xyz + (to_world * vec4<f32>(pixel.offset, 0.0)).xyz;
  let light: vec3<f32> = camera.position.xyz + (to_world * vec4<f32>(record.position.xyz, 0.0)).xyz;
  let to_light: vec3<f32> = normalize(light - point);
  let lift: vec3<f32> = select(
    normalize((to_world * vec4<f32>(pixel.normal, 0.0)).xyz) * (LIFT + distance * LIFT_PER_METRE),
    to_light * PLANT_LIFT,
    is_plant,
  );
  let origin: vec3<f32> = point + lift;
  // A point of the bulb, on the disc it shows the pixel.
  let frame: AxisFrame = axis_frame(to_light);
  let across: vec3<f32> = frame.across;
  let along: vec3<f32> = frame.along;
  let bulb: f32 = sqrt(record.shadow.w);
  let radius: f32 = sqrt(drawn.x) * bulb;
  let angle: f32 = drawn.y * TAU;
  let target_point: vec3<f32> = light + (across * cos(angle) + along * sin(angle)) * radius;
  let toward: vec3<f32> = target_point - origin;
  let length_to: f32 = length(toward);
  // Its light's near plane short of it, as the atlas's faces leave out what stands nearer the light.
  let reach: f32 = length_to - max(record.shadow.x, 0.0);

  if (reach <= 0.0) {
    return 1.0;
  }

  let cone: SceneCone = SceneCone(camera_pixel_footprint(distance), bulb / max(length_to, 1e-3));

  return select(1.0, 0.0, trace_scene_blocked(origin, toward / length_to, reach, cone, SCENE_NO_PLACE));
}

@compute @workgroup_size(8, 8)
fn cs_visibility_traced(@builtin(global_invocation_id) id: vec3<u32>) {
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

  let pixel: LitPixel = lit_pixel_by(clip, texel, depth, 0u, 0.0);
  let marks: f32 = textureLoad(material_target, texel, 0).a;
  let is_plant: bool = has_mark(marks, MARK_PLANT) || has_mark(marks, MARK_FLORA);
  let start: vec2<f32> = fract(gradient_noise_2(clip) +
    light_trace.noise.xy);
  var seen: array<f32, VISIBILITY_SLOTS> = array<f32, VISIBILITY_SLOTS>(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
  var slot: u32 = 0u;

  for (var index: u32 = 0u; index < counts[pixel.cluster] && slot < VISIBILITY_SLOTS; index++) {
    let record: LightRecord = records[items[pixel.cluster * LIGHT_CLUSTER_CAPACITY + index]];
    let light: ReachingLight = reaching_light(pixel, record);

    if (!is_reaching(record, light) || !takes_slot(pixel, light)) {
      continue;
    }

    seen[slot] = traced_seen(pixel, record, is_plant, fract(start + f32(slot) * R2_STEPS));
    slot++;
  }

  if (slot > 0u) {
    textureStore(light_visibility_out, texel, 0, vec4<f32>(seen[0], seen[1], seen[2], seen[3]));
  }

  if (slot > VISIBILITY_LAYER_SLOTS) {
    textureStore(light_visibility_out, texel, 1, vec4<f32>(seen[4], seen[5], seen[6], seen[7]));
  }
}
