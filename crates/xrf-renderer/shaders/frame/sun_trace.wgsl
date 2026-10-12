#import "common/camera"
#import "common/fullscreen"
#import "common/octahedral"
#import "common/scene_trace"
#import "common/noise"
#import "common/axis_frame"

// The sun's shadow traced at the frame's size: from each drawn pixel one ray towards a point of the sun's disc, drawn
// anew each frame, traced against the level's static geometry and its spawned models, cut-out leaves by their alpha;
// one where it meets nothing, which the sun's pass takes in its cascades' place, the noise of its soft edges left to
// DLSS ray reconstruction. Trees stand swayed and the grass near the eye stands as cards where the view's own top level
// holds them (`ViewTraceFeature`, `GrassCards`).

// The G-buffer's normals, material and depth, and the sun traced towards.
#import "generated/frame/sun_trace"
#import "common/math"

// Metres a ray starts off its surface along its normal, and how much more each metre away, so it leaves the surface it
// starts on.
const LIFT: f32 = 0.03;
const LIFT_PER_METRE: f32 = 0.002;
// Metres a plant's ray starts towards the sun past it: a leaf's or a blade's card faces any way and stands among
// others, which would shadow it at random.
const PLANT_LIFT: f32 = 0.3;

@fragment
fn fs_sun_trace(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    return vec4<f32>(1.0);
  }

  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let marks: f32 = textureLoad(material_target, texel, 0).a;
  let is_plant: bool = has_mark(marks, MARK_PLANT) || has_mark(marks, MARK_FLORA);

  // A surface facing away from the sun takes none of it, as the sun's pass lights it; a plant passes it through.
  if (!is_plant && dot(normal, sun_trace.to_sun.xyz) <= 0.0) {
    return vec4<f32>(0.0);
  }

  let to_world: mat4x4<f32> = transpose(camera.view);
  let to_sun: vec3<f32> = normalize((to_world * vec4<f32>(sun_trace.to_sun.xyz, 0.0)).xyz);
  let distance: f32 = -position.z;
  let world: vec3<f32> = camera.position.xyz + (to_world * vec4<f32>(position, 0.0)).xyz;
  let lift: vec3<f32> = select(normalize((to_world * vec4<f32>(normal, 0.0)).xyz) * (LIFT + distance * LIFT_PER_METRE),
    to_sun * PLANT_LIFT, is_plant);
  // A point of the sun's disc, turned every frame.
  let pixel: vec2<f32> = floor(in.clip.xy);
  let drawn: vec2<f32> = fract(gradient_noise_2(pixel) +
    vec2<f32>(sun_trace.noise_x, sun_trace.noise_y));
  let frame: AxisFrame = axis_frame(to_sun);
  let across: vec3<f32> = frame.across;
  let along: vec3<f32> = frame.along;
  let radius: f32 = sqrt(drawn.x) * sun_trace.spread;
  let angle: f32 = drawn.y * TAU;
  let direction: vec3<f32> = normalize(to_sun + (across * cos(angle) + along * sin(angle)) * radius);
  // What the ray's cone covers where it starts, a pixel's footprint, widening as the disc does.
  let cone: SceneCone = SceneCone(distance * 2.0 / (camera.projection[1][1] * camera.viewport.y), sun_trace.spread);
  let is_blocked: bool = trace_scene_blocked(world + lift, direction, SCENE_SUN_REACH, cone, SCENE_NO_PLACE);

  return vec4<f32>(select(1.0, 0.0, is_blocked));
}
