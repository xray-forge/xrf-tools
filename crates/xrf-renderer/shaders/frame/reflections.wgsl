#import "common/camera"
#import "common/octahedral"
#import "common/fullscreen"
#import "common/lighting"
#import "common/hmodel"
#import "common/occlusion"
#import "common/reflection_trace"
#import "common/water_enhanced"

// Screen-space reflections on the frame's surfaces, at the size the quality traces at, a pixel per `ratio` by `ratio`
// of the frame: each reflecting pixel's mirrored ray marched across the screen (`common/reflection_trace`), what it met
// as the frame shows it, the sky's where it met nothing on terrain or ran towards the sky, nothing elsewhere; blended
// with the last frame's, carried to where the reflection stood; then blurred twice, less where the surface is glossier.
// Combine blends each surface towards it by its share of reflection.
//
// Each stage writes the reflection, then one; alpha below none marks a pixel showing only the sky.

// The G-buffer, its light and occlusion, the material table, the lighting, the settings, the stage before's result and
// how far along the view each ray's hit lies, and the last frame's reflections and what they held.
#import "generated/frame/reflections"

// The sky's cubes, which a ray meeting nothing reflects.
#import "generated/common/sky"

// The share of reflection below which a pixel's ray is not marched.
const TRACE_FLOOR: f32 = 0.02;
// What the share is scaled by into how much of the reflection a pixel keeps, so a barely reflecting one leaks none.
const LEAK_SCALE: f32 = 10.0;
// How fast a reflection fades towards the top of the screen, as a share of its height.
const SCREEN_FADE: f32 = 4.0;
// The luminance what a ray meets is held to, so a lamp's glass or a sunlit highlight does not flare what reflects it.
const RADIANCE_LIMIT: f32 = 6.0;
// How much of the last frame's reflection is kept, what of that a reflection of the sky gives up, and how fast a
// changed depth where it stood gives it up.
const HISTORY_SHARE: f32 = 0.8;
const HISTORY_SKY: f32 = 0.2;
const HISTORY_DEPTH_SCALE: f32 = 5.0;
// The blur's taps, each turned from the last, and how much the gloss keeps of the unblurred reflection.
const BLUR_TAPS: u32 = 12u;
const BLUR_TURN: mat2x2<f32> = mat2x2<f32>(-0.666276, -0.745705, 0.745705, -0.666276);
const BLUR_GLOSS: f32 = 2.5;
const LUMINANCE: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);
const UNTRACED: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, -1.0);

// The frame's texel a traced pixel stands for: the first of its block.
fn traced_texel(pixel: vec2<f32>) -> vec2<i32> {
  return vec2<i32>(min(pixel * reflection.ratio, camera.viewport.xy - 1.0));
}

// The world's direction for one in view space.
fn world_direction(direction: vec3<f32>) -> vec3<f32> {
  return normalize((transpose(camera.view) * vec4<f32>(direction, 0.0)).xyz);
}

// What a ray met looks like as combine shows it before the tonemap: `hmodel` over its G-buffer, its occlusion where it
// is searched, fogged by its own distance, its luminance held to `RADIANCE_LIMIT`.
fn met_radiance(texel: vec2<i32>) -> vec3<f32> {
  let depth: f32 = textureLoad(depth_target, texel, 0);
  let albedo: vec4<f32> = textureLoad(albedo_target, texel, 0);
  let light: vec4<f32> = textureLoad(light_target, texel, 0);
  let material: vec4<f32> = textureLoad(material_target, texel, 0);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let position: vec3<f32> = camera_view_position(vec2<f32>(texel) + 0.5, depth);
  let searched: vec2<i32> = min(texel / 2, vec2<i32>(textureDimensions(occlusion_target)) - 1);
  let visible: f32 = select(1.0, textureLoad(occlusion_target, searched, 0).x, lighting.params.w > 0.5);
  let bounced: vec3<f32> = bounced_occlusion(visible, albedo.rgb, lighting.params.x);
  let occlusion: f32 = mix(1.0, material.x, camera.switches.z);
  let shaded: vec3<f32> = hmodel(lighting, material_lut, lut_sampler, sky_environment_0, sky_environment_1, sky_clamp,
    albedo, light, world_direction(normal), world_direction(normalize(position)), material.z, occlusion, bounced);
  let fogged: vec3<f32> = mix(shaded, lighting.fog_color.rgb, fog_amount(lighting, position));
  let luminance: f32 = dot(fogged, LUMINANCE);

  return fogged * min(1.0, RADIANCE_LIMIT / max(luminance, 1e-4));
}

// The traced pixels and how far along the view each ray's hit lies, nothing where it met none.
struct TracedTargets {
  @location(0) traced: vec4<f32>,
  @location(1) depth: f32,
};

@fragment
fn fs_trace(in: FullscreenVarying) -> TracedTargets {
  let pixel: vec2<f32> = floor(in.clip.xy);
  let texel: vec2<i32> = traced_texel(pixel);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    return TracedTargets(UNTRACED, 0.0);
  }

  let material: vec4<f32> = textureLoad(material_target, texel, 0);
  let albedo: vec4<f32> = textureLoad(albedo_target, texel, 0);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let position: vec3<f32> = camera_view_position(vec2<f32>(texel) + 0.5, depth);
  let to_point: vec3<f32> = normalize(position);
  let mirrored: vec3<f32> = reflect(to_point, normal);
  let share: f32 = reflection_share(albedo.a, normal, to_point, reflection.intensity,
    has_mark(material.a, MARK_FLORA), fog_amount(lighting, position));
  let is_terrain: f32 = select(0.0, 1.0, has_mark(material.a, MARK_TERRAIN));
  var hit: ReflectionHit = ReflectionHit(vec2<f32>(0.0), 0.0, 0.0);

  if (share > TRACE_FLOOR) {
    hit = reflection_march(depth_target, position, mirrored, reflection.distance, reflection.steps, reflection.limit,
      reflection.is_refined > 0.5);
  }

  var colour: vec3<f32> = enhanced_sky(world_direction(mirrored), lighting, sky_cube_0, sky_cube_1, sky_clamp);

  if (all(hit.position != vec2<f32>(0.0))) {
    let met: vec2<i32> = clamp(vec2<i32>(hit.position * camera.viewport.xy), vec2<i32>(0),
      vec2<i32>(camera.viewport.xy) - 1);

    colour = mix(colour * is_terrain, met_radiance(met), saturate(hit.position.y * SCREEN_FADE));
  } else {
    colour *= saturate(saturate(hit.sky * SCREEN_FADE) + is_terrain);
  }

  return TracedTargets(vec4<f32>(colour * saturate(share * LEAK_SCALE), 1.0), hit.depth);
}

// Where a texel of the frame stands in the world, and how far along the view.
struct TracedPoint {
  world: vec3<f32>,
  distance: f32,
};

fn traced_point(texel: vec2<i32>) -> TracedPoint {
  let depth: f32 = textureLoad(depth_target, texel, 0);
  let uv: vec2<f32> = (vec2<f32>(texel) + 0.5) / camera.viewport.xy;
  let world: vec3<f32> = camera_unproject(vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0), max(depth, 1e-7));

  return TracedPoint(world, -camera_view_position(vec2<f32>(texel) + 0.5, max(depth, 1e-7)).z);
}

// The reflections and what they held: the distance along the view of the point, then one.
struct AccumulatedTargets {
  @location(0) accumulated: vec4<f32>,
  @location(1) held: vec2<f32>,
};

// This frame's trace blended with the last frame's reflection, read where the point the pixel reflects stood: along the
// pixel's view at its hit's depth, or far off where it reflects the sky. Kept less where it reflects the sky, and none
// where it stood off the screen or where the depth there has changed.
@fragment
fn fs_accumulate(in: FullscreenVarying) -> AccumulatedTargets {
  let pixel: vec2<f32> = floor(in.clip.xy);
  let size: vec2<f32> = vec2<f32>(textureDimensions(traced));
  let fresh: vec4<f32> = textureLoad(traced, vec2<i32>(pixel), 0);

  if (fresh.a < 0.0) {
    return AccumulatedTargets(UNTRACED, vec2<f32>(0.0));
  }

  let point: TracedPoint = traced_point(traced_texel(pixel));
  let first: AccumulatedTargets = AccumulatedTargets(fresh, vec2<f32>(point.distance, 1.0));

  if (reflection.has_history < 0.5) {
    return first;
  }

  let hit_depth: f32 = textureLoad(traced_depth, vec2<i32>(pixel), 0).x;
  let toward: vec3<f32> = point.world - camera.position.xyz;
  let reflected: vec3<f32> = camera.position.xyz +
    select(toward * (hit_depth / max(point.distance, 1e-4)), normalize(toward) * 1e5, hit_depth <= 0.0);
  let clip: vec4<f32> = camera.motion_previous * vec4<f32>(reflected, 1.0);
  let before: vec2<f32> = (clip.xy / clip.w) * vec2<f32>(0.5, -0.5) + 0.5;

  if (clip.w <= 0.0 || any(before < vec2<f32>(0.0)) || any(before > vec2<f32>(1.0))) {
    return first;
  }

  let at: vec2<f32> = before * size - 0.5;
  let base: vec2<f32> = floor(at);
  let fraction: vec2<f32> = at - base;
  var carried: vec4<f32> = vec4<f32>(0.0);
  var weights: f32 = 0.0;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let offset: vec2<f32> = vec2<f32>(f32(corner & 1u), f32(corner >> 1u));
    let colour: vec4<f32> = textureLoad(history, vec2<i32>(clamp(base + offset, vec2<f32>(0.0), size - 1.0)), 0);
    let bilinear: f32 = mix(1.0 - fraction.x, fraction.x, offset.x) * mix(1.0 - fraction.y, fraction.y, offset.y);
    let weight: f32 = select(0.0, bilinear, colour.a >= 0.0);

    carried += colour * weight;
    weights += weight;
  }

  if (weights < 1e-3) {
    return first;
  }

  let held: f32 = textureLoad(history_held, vec2<i32>(clamp(floor(before * size), vec2<f32>(0.0), size - 1.0)), 0).x;
  let changed: f32 = select(1.0, abs(1.0 - point.distance / held) * HISTORY_DEPTH_SCALE, held > 0.0);
  let kept: f32 = saturate(HISTORY_SHARE - changed - select(0.0, HISTORY_SKY, hit_depth <= 0.0));

  return AccumulatedTargets(mix(fresh, carried / weights, kept), vec2<f32>(point.distance, 1.0));
}

// The reflections blurred over a spiral of taps `spacing` traced pixels apart, the glossier a surface the more of its
// unblurred reflection kept; at half size, never less blurred than a third.
fn blurred(pixel: vec2<f32>, spacing: f32) -> vec4<f32> {
  let center: vec4<f32> = textureLoad(traced, vec2<i32>(pixel), 0);

  if (center.a < 0.0) {
    return UNTRACED;
  }

  let last: vec2<f32> = vec2<f32>(textureDimensions(traced)) - 1.0;
  let gloss: f32 = textureLoad(albedo_target, traced_texel(pixel), 0).a;
  var offset: vec2<f32> = vec2<f32>(1.0);
  var radius: f32 = 0.9;
  var sum: vec3<f32> = vec3<f32>(0.0);

  for (var tap: u32 = 0u; tap < BLUR_TAPS; tap++) {
    radius += 1.0 / radius;
    offset = offset * BLUR_TURN;

    let at: vec2<f32> = clamp(floor(pixel + 0.5 + offset * (radius - 1.0) * spacing), vec2<f32>(0.0), last);

    sum += max(textureLoad(traced, vec2<i32>(at), 0).rgb, vec3<f32>(0.0));
  }

  let blur: f32 = clamp(0.0, (reflection.ratio / 10.0 - 0.1) * 3.0, 1.0);

  return vec4<f32>(mix(sum / f32(BLUR_TAPS), center.rgb, saturate(gloss * (1.0 - blur) * BLUR_GLOSS)), center.a);
}

// The first blur, its taps a traced pixel apart.
@fragment
fn fs_blur(in: FullscreenVarying) -> @location(0) vec4<f32> {
  return blurred(floor(in.clip.xy), 1.0);
}

// The second, its taps a frame's pixel apart.
@fragment
fn fs_blur_fine(in: FullscreenVarying) -> @location(0) vec4<f32> {
  return blurred(floor(in.clip.xy), 1.0 / reflection.ratio);
}
