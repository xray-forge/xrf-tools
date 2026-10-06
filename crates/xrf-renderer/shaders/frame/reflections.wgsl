#import "common/camera"
#import "common/octahedral"
#import "common/fullscreen"
#import "common/lighting"
#import "common/hmodel"
#import "common/occlusion"
#import "common/reflection_trace"

// Screen-space reflections on the frame's surfaces, at the size the quality traces at, a pixel per `ratio` by `ratio`
// of the frame: each glossy pixel's reflected ray, spread about the mirror direction by how rough its gloss makes it,
// marched over the nearest depth at the size traced (`common/reflection_trace`), and what it met lit as combine lights
// it; then accumulated over frames, carried to where the reflection stood, and filtered. What reaches combine replaces
// the irradiance cube `hmodel` reflects wherever a ray met something, and leaves the cube wherever it met nothing.
//
// Each stage writes the radiance met times how far it is trusted, then that trust; alpha below none marks a pixel not
// traced, too matte, too far turned from the view or foliage, where the engine's cube stands alone.

// The G-buffer, its light, motion and occlusion, the material table, the lighting, the settings, the nearest depth at
// the size traced, the stage before's result and its rays' lengths, and the last frame's accumulation and what it held.
#import "generated/frame/reflections"

// Both keyframes' irradiance cubes, which light what a ray meets.
#import "generated/common/sky"

// The gloss at which a surface reflects most sharply, and the roughness at no gloss and at that gloss: the dry level's
// glosses (under 0.1) leave a broad blur of the surroundings, a wet patch's (0.2 to 0.6) a clear, slightly broken
// mirror.
const SMOOTH_GLOSS: f32 = 0.5;
const ROUGHEST: f32 = 0.35;
const SMOOTHEST: f32 = 0.03;
// The share of the environment a pixel reflects below which it is not traced: the cube alone shows the same there.
const TRACE_FLOOR: f32 = 0.01;
// Metres behind what the depth shows a ray may stand and still have met it, and what that grows by each metre from
// the camera: deeper, rays passing behind a post or a railing meet it.
const THICKNESS: f32 = 0.2;
const THICKNESS_GROWTH: f32 = 0.02;
// Metres off its surface a ray leaves along the normal, and what that grows by each metre from the camera.
const RAY_OFFSET: f32 = 0.02;
const RAY_OFFSET_GROWTH: f32 = 0.004;
// The luminance what a ray meets is held to, so a lamp's glass or a sunlit highlight does not flare what reflects it.
const RADIANCE_LIMIT: f32 = 6.0;
// Share of a point's distance its last frame's distance may differ by and still be carried.
const HISTORY_TOLERANCE: f32 = 0.1;
// Share of a point's distance a neighbour may stand off the plane through it and still be filtered with it.
const FILTER_TOLERANCE: f32 = 0.05;
// How far a neighbour's normal may turn from the centre's before it is filtered out, as a power of their cosine.
const FILTER_NORMAL_POWER: f32 = 16.0;
// Traced pixels the filter's taps are spaced by at the roughest, and its spread there.
const FILTER_SPACING: f32 = 2.0;
const FILTER_SPREAD: f32 = 1.5;
const LUMINANCE: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);
const UNTRACED: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, -1.0);

// The frame's texel a traced pixel stands for: the first of its block.
fn traced_texel(pixel: vec2<f32>) -> vec2<i32> {
  return vec2<i32>(min(pixel * reflection.ratio, camera.viewport.xy - 1.0));
}

// How rough a surface's gloss makes its reflection, as a Phong lobe's spread.
fn gloss_roughness(gloss: f32) -> f32 {
  return mix(ROUGHEST, SMOOTHEST, smoothstep(0.0, SMOOTH_GLOSS, gloss));
}

// How the settings march a ray, from a pixel's noise.
fn traced_march(noise: f32) -> ReflectionMarch {
  return ReflectionMarch(reflection.base, reflection.steps, THICKNESS, THICKNESS_GROWTH, noise);
}

// Interleaved gradient noise: one value a pixel, from zero to one.
fn trace_noise(pixel: vec2<f32>) -> f32 {
  return fract(52.9829189 * fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))));
}

// What a ray met looks like as combine lights it, before the fog and the tonemap: `hmodel` over its G-buffer, its
// occlusion where it is searched, fogged over the ray's length, its luminance held to `RADIANCE_LIMIT`.
fn met_radiance(texel: vec2<i32>, reach: f32) -> vec3<f32> {
  let depth: f32 = textureLoad(depth_target, texel, 0);
  let albedo: vec4<f32> = textureLoad(albedo_target, texel, 0);
  let light: vec4<f32> = textureLoad(light_target, texel, 0);
  let material: vec4<f32> = textureLoad(material_target, texel, 0);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let position: vec3<f32> = camera_view_position(vec2<f32>(texel) + 0.5, depth);
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
  let searched: vec2<i32> = min(texel / 2, vec2<i32>(textureDimensions(occlusion_target)) - 1);
  let visible: f32 = select(1.0, textureLoad(occlusion_target, searched, 0).x, lighting.params.w > 0.5);
  let bounced: vec3<f32> = bounced_occlusion(visible, albedo.rgb, lighting.params.x);
  let occlusion: f32 = mix(1.0, material.x, camera.switches.z);
  let shaded: vec3<f32> = hmodel(lighting, material_lut, lut_sampler, sky_environment_0, sky_environment_1, sky_clamp,
    albedo, light, normalize(rotation * normal), normalize(rotation * normalize(position)), material.z, occlusion,
    bounced);
  let fogged: vec3<f32> = mix(shaded, lighting.fog_color.rgb, fog_amount(lighting, vec3<f32>(0.0, 0.0, reach)));
  let luminance: f32 = dot(fogged, LUMINANCE);

  return fogged * min(1.0, RADIANCE_LIMIT / max(luminance, 1e-4));
}

// The traced pixels and how far each ray went.
struct TracedTargets {
  @location(0) traced: vec4<f32>,
  @location(1) reach: f32,
};

@fragment
fn fs_trace(in: FullscreenVarying) -> TracedTargets {
  let pixel: vec2<f32> = floor(in.clip.xy);
  let texel: vec2<i32> = traced_texel(pixel);
  let depth: f32 = textureLoad(depth_target, texel, 0);
  let untraced: TracedTargets = TracedTargets(UNTRACED, 0.0);

  if (depth <= 0.0) {
    return untraced;
  }

  let material: vec4<f32> = textureLoad(material_target, texel, 0);

  // Foliage's gloss is cut where it is lit as foliage, and a lamp's glass reflects nothing it shows.
  if (has_mark(material.a, MARK_FLORA) || has_mark(material.a, MARK_EMISSIVE)) {
    return untraced;
  }

  let albedo: vec4<f32> = textureLoad(albedo_target, texel, 0);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let position: vec3<f32> = camera_view_position(vec2<f32>(texel) + 0.5, depth);
  let to_point: vec3<f32> = normalize(position);
  let occlusion: f32 = mix(1.0, material.x, camera.switches.z);
  let hemisphere: vec4<f32> = hmodel_hemisphere(material_lut, lut_sampler, normal, to_point, material.z, occlusion);
  let weight: f32 = hmodel_gloss_weight(lighting, hemisphere, albedo.a);
  // On Anomaly the reflection is tinted by the surface's lit albedo, about as bright as its albedo under the sky.
  let light: vec3<f32> = textureLoad(light_target, texel, 0).rgb;
  let tint: f32 = select(1.0, dot(albedo.rgb, LUMINANCE) * (dot(light, LUMINANCE) + 1.0), lighting.engine.x > 0.5);
  let share: f32 = hemisphere.y * weight * tint;

  if (share * reflection.intensity < TRACE_FLOOR) {
    return untraced;
  }

  let distance: f32 = -position.z;
  let noise: vec2<f32> = vec2<f32>(
    trace_noise(pixel + reflection.noise),
    trace_noise(pixel.yx + vec2<f32>(37.0, 17.0) + reflection.noise),
  );
  let direction: vec3<f32> = reflection_lobe(normal, to_point, gloss_roughness(albedo.a), noise);
  let ray: ReflectionRay = ReflectionRay(
    position + normal * (RAY_OFFSET + RAY_OFFSET_GROWTH * distance),
    direction,
    reflection.distance,
  );
  let march: ReflectionMarch = traced_march(trace_noise(pixel.yx + vec2<f32>(11.0, 53.0) + reflection.noise));
  let hit: ReflectionHit = reflection_march(nearest_depth, march, ray);
  let left: vec2<f32> = (vec2<f32>(texel) + 0.5) / camera.viewport.xy;
  let trust: f32 = reflection_confidence(nearest_depth, depth_target, normal_target, hit, ray, left, march);

  if (trust <= 0.0) {
    return TracedTargets(vec4<f32>(0.0), 0.0);
  }

  let reach: f32 = hit.along * reflection_length(ray);
  let met: vec2<i32> = clamp(vec2<i32>(hit.position.xy * camera.viewport.xy), vec2<i32>(0),
    vec2<i32>(camera.viewport.xy) - 1);

  return TracedTargets(vec4<f32>(met_radiance(met, reach) * trust, trust), reach);
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

// The accumulation and what it held: the distance along the view of the point, then the frames gathered.
struct AccumulatedTargets {
  @location(0) accumulated: vec4<f32>,
  @location(1) held: vec2<f32>,
};

// This frame's trace blended with the last frame's accumulation, read where the reflection stood: where the ray met
// something, where the point it met stood mirrored behind the surface, so a reflection moving over a still floor is
// carried with it; elsewhere where the surface stood. Kept within what was traced around the pixel this frame, so a
// turn or a newly seen place does not trail, and dropped where the surface there was another.
@fragment
fn fs_accumulate(in: FullscreenVarying) -> AccumulatedTargets {
  let pixel: vec2<f32> = floor(in.clip.xy);
  let size: vec2<f32> = vec2<f32>(textureDimensions(traced));
  let fresh: vec4<f32> = textureLoad(traced, vec2<i32>(pixel), 0);

  if (fresh.a < 0.0) {
    return AccumulatedTargets(UNTRACED, vec2<f32>(0.0));
  }

  let texel: vec2<i32> = traced_texel(pixel);
  let point: TracedPoint = traced_point(texel);
  let first: AccumulatedTargets = AccumulatedTargets(fresh, vec2<f32>(point.distance, 1.0));

  if (reflection.has_history < 0.5) {
    return first;
  }

  let reach: f32 = textureLoad(traced_length, vec2<i32>(pixel), 0).x;
  let mirrored: vec3<f32> = point.world + normalize(point.world - camera.position.xyz) * reach;
  let carried_point: vec3<f32> = select(point.world, mirrored, fresh.a > 0.5 && reach > 0.0);
  let clip: vec4<f32> = camera.motion_previous * vec4<f32>(carried_point, 1.0);
  let before: vec2<f32> = (clip.xy / clip.w) * vec2<f32>(0.5, -0.5) + 0.5;
  // How far along the last frame's view this point stood.
  let expected: f32 = (camera.motion_previous * vec4<f32>(point.world, 1.0)).w;

  if (clip.w <= 0.0 || any(before < vec2<f32>(0.0)) || any(before > vec2<f32>(1.0))) {
    return first;
  }

  let at: vec2<f32> = before * size - 0.5;
  let base: vec2<f32> = floor(at);
  let fraction: vec2<f32> = at - base;
  var carried: vec4<f32> = vec4<f32>(0.0);
  var frames: f32 = 0.0;
  var weights: f32 = 0.0;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let offset: vec2<f32> = vec2<f32>(f32(corner & 1u), f32(corner >> 1u));
    let held_at: vec2<i32> = vec2<i32>(clamp(base + offset, vec2<f32>(0.0), size - 1.0));
    let held: vec2<f32> = textureLoad(history_held, held_at, 0).xy;
    let colour: vec4<f32> = textureLoad(history, held_at, 0);
    let bilinear: f32 = mix(1.0 - fraction.x, fraction.x, offset.x) * mix(1.0 - fraction.y, fraction.y, offset.y);
    let is_same: bool = colour.a >= 0.0 && held.x > 0.0 && abs(held.x - expected) <= expected * HISTORY_TOLERANCE;
    let weight: f32 = select(0.0, bilinear, is_same);

    carried += colour * weight;
    frames += held.y * weight;
    weights += weight;
  }

  if (weights < 1e-3) {
    return first;
  }

  // What was traced around the pixel this frame, which the carried reflection is kept within.
  var low: vec4<f32> = fresh;
  var high: vec4<f32> = fresh;

  for (var index: i32 = 0; index < 9; index++) {
    let neighbour_at: vec2<i32> = vec2<i32>(clamp(pixel + vec2<f32>(f32(index % 3 - 1), f32(index / 3 - 1)),
      vec2<f32>(0.0), size - 1.0));
    let neighbour: vec4<f32> = textureLoad(traced, neighbour_at, 0);

    if (neighbour.a >= 0.0) {
      low = min(low, neighbour);
      high = max(high, neighbour);
    }
  }

  let previous: vec4<f32> = clamp(carried / weights, low, high);
  let gathered: f32 = min(round(frames / weights) + 1.0, reflection.frames);

  return AccumulatedTargets(mix(previous, fresh, 1.0 / gathered), vec2<f32>(point.distance, gathered));
}

// The accumulation filtered over its neighbours of the same surface, weighed by how near each lies to the plane
// through the centre and how near its normal turns to the centre's, its taps spread by how rough the surface is and
// its bright texels weighed down, so a lone hot hit does not sparkle; then times the intensity.
@fragment
fn fs_filter(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let pixel: vec2<f32> = floor(in.clip.xy);
  let last: vec2<f32> = vec2<f32>(textureDimensions(traced)) - 1.0;
  let center: vec4<f32> = textureLoad(traced, vec2<i32>(pixel), 0);

  if (center.a < 0.0) {
    return UNTRACED;
  }

  let texel: vec2<i32> = traced_texel(pixel);
  let point: TracedPoint = traced_point(texel);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let rough: f32 = saturate((gloss_roughness(textureLoad(albedo_target, texel, 0).a) - SMOOTHEST) /
    (ROUGHEST - SMOOTHEST));
  let spacing: f32 = mix(1.0, FILTER_SPACING, rough);
  let spread: f32 = mix(0.5, FILTER_SPREAD, rough);
  let position: vec3<f32> = camera_view_position(vec2<f32>(texel) + 0.5, textureLoad(depth_target, texel, 0));
  let tolerance: f32 = max(point.distance * FILTER_TOLERANCE, 1e-3);
  var sum: vec4<f32> = vec4<f32>(0.0);
  var weights: f32 = 0.0;

  for (var y: i32 = -1; y <= 1; y++) {
    for (var x: i32 = -1; x <= 1; x++) {
      let offset: vec2<f32> = vec2<f32>(f32(x), f32(y));
      let at: vec2<f32> = clamp(pixel + round(offset * spacing), vec2<f32>(0.0), last);
      let sample: vec4<f32> = textureLoad(traced, vec2<i32>(at), 0);
      let sample_texel: vec2<i32> = traced_texel(at);
      let depth: f32 = textureLoad(depth_target, sample_texel, 0);

      if (sample.a < 0.0 || depth <= 0.0) {
        continue;
      }

      let sample_position: vec3<f32> = camera_view_position(vec2<f32>(sample_texel) + 0.5, depth);
      let off_plane: f32 = abs(dot(sample_position - position, normal)) / tolerance;
      let sample_normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, sample_texel, 0).xy);
      let facing: f32 = pow(saturate(dot(sample_normal, normal)), FILTER_NORMAL_POWER);
      let spatial: f32 = exp(-dot(offset, offset) / (2.0 * spread * spread));
      let radiance: f32 = dot(sample.rgb, LUMINANCE);
      let weight: f32 = spatial * facing * saturate(1.0 - off_plane) / (1.0 + radiance);

      sum += sample * weight;
      weights += weight;
    }
  }

  let filtered: vec4<f32> = select(center, sum / weights, weights > 1e-4);

  return filtered * reflection.intensity;
}
