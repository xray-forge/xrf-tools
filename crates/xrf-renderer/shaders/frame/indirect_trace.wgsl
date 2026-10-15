#import "common/camera"
#import "common/fullscreen"
#import "common/octahedral"
#import "common/scene_trace"
#import "generated/common/sky"
#import "generated/frame/indirect_trace"

// The indirect light traced at half the frame's size, in the screen-space bounce's place where DLSS ray reconstruction
// denoises the frame: from each point under a pixel one ray, cosine-weighted about its bumped normal and turned each
// frame, traced against the level's static geometry, its swayed trees and its cut-outs (taken whole, as its wide cone
// is), the grass passed through; what it meets lit as the reflections light a hit (the sun, the lamps, its baked
// hemisphere, which stands for the bounces past it), the light it carries back. A ray meeting nothing carries none: the
// sky is the hemisphere's. Written as the bounce's filter writes it, the light by the intensity and the distance along
// the view, for combine and the debug view to bring up to the frame alike; and how far the ray went, none where it met
// nothing, which ray reconstruction's guides take as its diffuse hit distance.

const TAU: f32 = 6.2831853;
// Metres a ray starts off its surface, and how much more each metre away, so it leaves the surface it starts on.
const LIFT: f32 = 0.05;
const LIFT_PER_METRE: f32 = 0.002;
// Metres a plant's rays start over it, past the ground's bumps about its foot, which its sway would move it among.
const PLANT_LIFT: f32 = 0.3;
// The most a hit's light carries back, by its luminance, so a lamp's hot spot is no firefly.
const RADIANCE_MOST: f32 = 8.0;

// Jimenez's interleaved gradient noise.
fn gradient_noise(pixel: vec2<f32>) -> f32 {
  return fract(52.9829189 * fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))));
}

fn brightness(colour: vec3<f32>) -> f32 {
  return dot(colour, vec3<f32>(0.2126, 0.7152, 0.0722));
}

struct IndirectTargets {
  @location(0) light: vec4<f32>,
  @location(1) hit: f32,
};

@fragment
fn fs_indirect_trace(in: FullscreenVarying) -> IndirectTargets {
  let pixel: vec2<f32> = floor(in.clip.xy);
  // The frame's texel under the pixel.
  let texel: vec2<i32> = min(vec2<i32>(pixel) * 2, vec2<i32>(textureDimensions(depth_target)) - 1);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    return IndirectTargets(vec4<f32>(0.0), 0.0);
  }

  let position: vec3<f32> = camera_view_position(vec2<f32>(texel) + 0.5, depth);
  let distance: f32 = -position.z;
  let to_world: mat4x4<f32> = transpose(camera.view);
  let is_plant: bool = has_mark(textureLoad(material_target, texel, 0).a, MARK_PLANT);
  let bumped: vec3<f32> = normalize((to_world * vec4<f32>(octahedral_decode(textureLoad(normal_target, texel, 0).xy),
    0.0)).xyz);
  // A plant's rays go about the world's up, as its light leans it up, from over the ground about its foot.
  let normal: vec3<f32> = select(bumped, vec3<f32>(0.0, 1.0, 0.0), is_plant);
  let lift: f32 = select(LIFT, PLANT_LIFT, is_plant) + distance * LIFT_PER_METRE;
  let origin: vec3<f32> = camera.position.xyz + (to_world * vec4<f32>(position, 0.0)).xyz + normal * lift;
  let axis: vec3<f32> = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(normal.y) > 0.9);
  let across: vec3<f32> = normalize(cross(normal, axis));
  let along: vec3<f32> = cross(normal, across);
  let sample: vec2<f32> = fract(vec2<f32>(gradient_noise(pixel), gradient_noise(pixel.yx + vec2<f32>(17.0, 59.0))) +
    indirect.noise.xy);
  let radius: f32 = sqrt(sample.x);
  let angle: f32 = sample.y * TAU;
  let direction: vec3<f32> = normalize(across * (radius * cos(angle)) + along * (radius * sin(angle)) +
    normal * sqrt(max(1.0 - sample.x, 0.0)));
  // A diffuse bounce's ray spreads as wide as a hemisphere: its hit is read coarsely.
  let cone: SceneCone = SceneCone(distance * 0.004, 0.5);
  let hit: SceneHit = trace_scene_ray(origin, direction, 0.0, indirect.noise.z, cone);

  if (hit.distance < 0.0) {
    return IndirectTargets(vec4<f32>(0.0, 0.0, 0.0, distance), 0.0);
  }

  let shown: vec3<f32> = shade_scene_hit(hit, direction, cone, lighting, material_lut, lut_sampler, sky_environment_0,
    sky_environment_1, sky_clamp);
  let carried: vec3<f32> = shown * min(1.0, RADIANCE_MOST / max(brightness(shown), 1e-4));

  return IndirectTargets(vec4<f32>(carried * indirect.noise.w, distance), hit.distance);
}
