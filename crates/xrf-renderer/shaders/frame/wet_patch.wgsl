#import "common/camera"
#import "common/octahedral"
#import "common/rain_cover"
#import "common/fullscreen"
#import "common/wet"

// `rain_patch_normal` (`r3_rendertarget_draw_rain.cpp`): where the rain reaches a surface near the camera, splashes on
// what faces up and water running down what stands, as a normal bent in view space, and how wet it is in alpha, weighed
// by the albedo's brightness. Computed in the engine's axes, so the ripples run as the game's do.

@group(1) @binding(0) var depth_target: texture_depth_2d;
@group(1) @binding(1) var albedo_target: texture_2d<f32>;
@group(1) @binding(2) var normal_target: texture_2d<f32>;
@group(1) @binding(3) var cover: texture_depth_2d;
// `s_water`, a volume of rippling normals as its slices, and `s_waterFall`, the normals of water running down.
@group(1) @binding(4) var splash: texture_2d_array<f32>;
@group(1) @binding(5) var flow: texture_2d<f32>;
@group(1) @binding(6) var wet_sampler: sampler;
@group(1) @binding(7) var<uniform> wet: Wet;

// Metres a point may stand under its cover's texel and still take the rain, as a slope spans one.
const COVER_BIAS: f32 = 0.2;

// The engines' values: metres past which the rain wets nothing, how fast the splashes' volume runs, how far the water
// tilts the normal up, how strongly and how fast the flow bends and runs.
fn engine_value(vanilla: f32, extended: f32) -> f32 {
  return select(vanilla, extended, wet.is_extended > 0.5);
}

// Whether the rain reaches a point: four taps of the cover about it, as `shadow_rain` jitters four, each open where
// the point stands at or above the first thing over its column.
fn rain_open(world: vec3<f32>) -> f32 {
  let texel: f32 = wet.window.z * 2.0 / RAIN_COVER_RESOLUTION;
  var open: f32 = 0.0;

  for (var tap: u32 = 0u; tap < 4u; tap++) {
    let offset: vec2<f32> = vec2<f32>(f32(tap & 1u), f32(tap >> 1u)) - 0.5;
    let column: vec3<f32> = world + vec3<f32>(texel * offset.x, 0.0, texel * offset.y);

    open += select(0.0, 1.0, world.y + COVER_BIAS >= rain_cover_height(cover, wet.window, column));
  }

  return open * 0.25;
}

// `GetNVNMap`: the splashes' volume at a place and a moment, repeating every way and blended between its slices, its
// normal in `w`, `y` and `z`, laid flat.
fn splash_normal(place: vec2<f32>, moment: f32) -> vec3<f32> {
  let layers: f32 = f32(textureNumLayers(splash));
  let along: f32 = fract(moment) * layers - 0.5;
  let first: f32 = floor(along);
  let near: i32 = i32((first + layers) % layers);
  let far: i32 = i32((first + 1.0) % layers);
  let water: vec4<f32> = mix(
    textureSampleLevel(splash, wet_sampler, place, near, 0.0),
    textureSampleLevel(splash, wet_sampler, place, far, 0.0),
    along - first,
  ) - 0.5;

  return vec3<f32>(water.w * 6.0, engine_value(0.0, 0.1), water.z * 6.0);
}

// `GetWaterNMap`: the flow's normal at a point of it, laid flat.
fn flow_normal(at: vec2<f32>) -> vec3<f32> {
  let water: vec3<f32> = (textureSampleLevel(flow, wet_sampler, at, 0.0).xzy - 0.5) * 2.0 * engine_value(0.3, 0.4);

  return vec3<f32>(water.x, engine_value(0.0, 0.1), water.z);
}

@fragment
fn fs_wet_patch(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  // Reversed: nought where nothing was drawn.
  if (depth <= 0.0) {
    discard;
  }

  let albedo: vec3<f32> = textureLoad(albedo_target, texel, 0).rgb;
  let normal: vec3<f32> = normalize(octahedral_decode(textureLoad(normal_target, texel, 0).xy));
  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let size: vec2<f32> = camera.viewport.xy;
  let ndc: vec2<f32> = vec2<f32>(in.clip.x / size.x * 2.0 - 1.0, 1.0 - in.clip.y / size.y * 2.0);
  let world: vec3<f32> = camera_unproject(ndc, depth);
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
  let world_normal: vec3<f32> = normalize(rotation * normal);
  // The engine's axes: `z` negated.
  let place: vec3<f32> = vec3<f32>(world.x, world.y, -world.z);
  let facing: vec3<f32> = vec3<f32>(world_normal.x, world_normal.y, -world_normal.z);
  let fade: f32 = 1.0 - smoothstep(5.0, engine_value(20.0, 25.0), -position.z);
  // `-dot(Ldynamic_dir, N)`, the rain falling straight down.
  let up: f32 = facing.y;
  let wetness: f32 = rain_open(world) * fade * fade * wet.density * saturate(up * 10.0 + 10.0 * 0.5 + 0.5);
  let upward: f32 = max(up, 0.0);
  let splashed: vec3<f32> = splash_normal(place.xz, wet.time * engine_value(3.0, 1.0));
  let along: vec3<f32> = place / 2.0;
  let slide: f32 = ceil((1.0 - upward) * 10.0) * 0.1 * engine_value(0.5, 0.3);
  let fall_x: vec3<f32> = flow_normal(vec2<f32>(along.z, along.y + wet.time * slide));
  let fall_z: vec3<f32> = flow_normal(vec2<f32>(along.x, along.y + wet.time * slide));
  // Nothing on the weapon in hand, which the viewer has none of.
  let applied: f32 = wetness * smoothstep(0.8, 0.9, length(position));
  let water: vec3<f32> = splashed * (upward * applied) + fall_x.yxz * (abs(facing.x) * applied) +
    fall_z.zxy * (abs(facing.z) * applied);
  let bent: vec3<f32> = (camera.view * vec4<f32>(water.x, water.y, -water.z, 0.0)).xyz;

  return vec4<f32>(normalize(normal + bent), wetness * dot(albedo, vec3<f32>(0.33)));
}
