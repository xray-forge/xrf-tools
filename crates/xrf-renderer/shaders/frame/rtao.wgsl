#import "common/camera"
#import "common/fullscreen"
#import "common/scene_trace"

// RTAO at half the frame's size: from each point under a search pixel, a few short rays cosine-weighted about its
// surface, traced against the level's static geometry and its spawned models, cut-out leaves by their alpha; the share
// that meets nothing within the radius is its visibility, written as the bitmask search writes its own (visibility,
// distance along the view), for the same accumulation over frames and filter. Grass, planted on the GPU each frame,
// has no geometry in the trace: a ray clear of the trace passes the grass's layers instead, which darkens the ground
// under thick grass and its own lower half; a plant's own rays go about the world's up.

// The normals, the depth and the frame's points at half size, and the occlusion's radius, spread and noise.
#import "generated/frame/vbao"

// Rays a point casts each frame, by quality.
override RAYS: u32 = 4u;

const TAU: f32 = 6.2831853;
// The R2 sequence's steps, which spread a pixel's rays apart.
const R2: vec2<f32> = vec2<f32>(0.7548777, 0.5698403);
// Metres a ray starts off its surface, and how much more each metre away, so it leaves the surface it starts on.
const LIFT: f32 = 0.03;
const LIFT_PER_METRE: f32 = 0.002;
// Metres a plant's rays start over it, past the ground's bumps about its foot, which its sway would move it among.
const PLANT_LIFT: f32 = 0.3;

// Jimenez's interleaved gradient noise: neighbours far apart in value, so a few rays a pixel average out over a small
// footprint.
fn gradient_noise(pixel: vec2<f32>) -> f32 {
  return fract(52.9829189 * fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))));
}

// The view space point the frame shows at a texel, held to the frame; none where nothing is drawn.
fn shown_point(texel: vec2<i32>) -> vec4<f32> {
  let at: vec2<i32> = clamp(texel, vec2<i32>(0), vec2<i32>(textureDimensions(depth_target)) - 1);
  let depth: f32 = textureLoad(depth_target, at, 0);

  return vec4<f32>(camera_view_position(vec2<f32>(at) + 0.5, max(depth, 1e-7)), select(0.0, 1.0, depth > 0.0));
}

// Of the steps to a point's two neighbours along an axis, the one along its own surface: the nearer in depth, a drawn
// one before none.
fn surface_step(center: vec3<f32>, before: vec4<f32>, after: vec4<f32>) -> vec3<f32> {
  let behind: vec3<f32> = center - before.xyz;
  let ahead: vec3<f32> = after.xyz - center;
  let is_ahead: bool = after.w > 0.5 && (before.w < 0.5 || abs(ahead.z) < abs(behind.z));

  return select(behind, ahead, is_ahead);
}

// The occlusion traced at a search pixel: its visibility and its distance along the view.
fn trace_occlusion(clip: vec2<f32>) -> vec4<f32> {
  let pixel: vec2<f32> = floor(clip);
  // The frame's texel under the search pixel, the one its depth is read at.
  let texel: vec2<i32> = vec2<i32>(pixel) * 2;
  let center: vec4<f32> = shown_point(texel);

  if (center.w < 0.5) {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
  }

  let position: vec3<f32> = center.xyz;
  let distance: f32 = -position.z;
  // The surface's own normal from the depth, facing the camera: a bumped normal casts rays into the flat surface under
  // its bumps and darkens them, and a two-sided card's may face away.
  let across_screen: vec3<f32> =
    surface_step(position, shown_point(texel - vec2<i32>(1, 0)), shown_point(texel + vec2<i32>(1, 0)));
  let down_screen: vec3<f32> =
    surface_step(position, shown_point(texel - vec2<i32>(0, 1)), shown_point(texel + vec2<i32>(0, 1)));
  let crossed: vec3<f32> = normalize(cross(down_screen, across_screen));
  let surface: vec3<f32> = select(crossed, -crossed, dot(crossed, position) > 0.0);
  let to_world: mat4x4<f32> = transpose(camera.view);
  // A plant's rays go about the world's up: grass is not in the trace, so from a blade's own normal, which its depth
  // gives at random as it sways, they met the ground it grows from and whole tufts flickered between dark and lit;
  // about the up only what stands over it occludes it, as its light leans it up.
  let is_plant: bool = has_mark(textureLoad(material_target, texel, 0).a, MARK_PLANT);
  let normal: vec3<f32> = select(normalize((to_world * vec4<f32>(surface, 0.0)).xyz), vec3<f32>(0.0, 1.0, 0.0), is_plant);
  let lift: f32 = select(LIFT, PLANT_LIFT, is_plant) + distance * LIFT_PER_METRE;
  let point: vec3<f32> = camera.position.xyz + (to_world * vec4<f32>(position, 0.0)).xyz;
  let origin: vec3<f32> = point + normal * lift;
  // A frame about the normal.
  let axis: vec3<f32> = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(normal.y) > 0.9);
  let across: vec3<f32> = normalize(cross(normal, axis));
  let along: vec3<f32> = cross(normal, across);
  // What a ray's cone covers where it starts: a search pixel's footprint there.
  let cone: SceneCone = SceneCone(distance * occlusion.spread, 0.0);
  // Turned every frame by the occlusion's noise, which its accumulation gathers.
  let start: vec2<f32> = fract(vec2<f32>(gradient_noise(pixel), gradient_noise(pixel.yx + vec2<f32>(17.0, 59.0))) +
    vec2<f32>(occlusion.slice_noise, occlusion.step_noise));
  var open: f32 = 0.0;

  for (var ray: u32 = 0u; ray < RAYS; ray++) {
    let sample: vec2<f32> = fract(start + f32(ray) * R2);
    // Cosine-weighted about the normal: the share open is the hemisphere's light that reaches the point.
    let radius: f32 = sqrt(sample.x);
    let angle: f32 = sample.y * TAU;
    let direction: vec3<f32> = normalize(across * (radius * cos(angle)) + along * (radius * sin(angle)) +
      normal * sqrt(max(1.0 - sample.x, 0.0)));

    if (!trace_scene_blocked(origin, direction, occlusion.radius, cone, SCENE_NO_PLACE)) {
      open += scene_grass_through(origin, direction, occlusion.radius, fract(sample.x + sample.y), point.y);
    }
  }

  return vec4<f32>(open / f32(RAYS), distance, 0.0, 1.0);
}

@fragment
fn fs_rtao(in: FullscreenVarying) -> @location(0) vec4<f32> {
  return trace_occlusion(in.clip.xy);
}

// Raw: what the trace found, which the light's accumulation measures its history by, and what combine reads, through
// the strength's power as the filter would give it, its noise left to DLSS ray reconstruction.
struct RawTargets {
  @location(0) searched: vec4<f32>,
  @location(1) shown: vec4<f32>,
};

@fragment
fn fs_rtao_raw(in: FullscreenVarying) -> RawTargets {
  let traced: vec4<f32> = trace_occlusion(in.clip.xy);

  return RawTargets(traced, vec4<f32>(pow(max(traced.x, 1e-4), occlusion.power), traced.yzw));
}
