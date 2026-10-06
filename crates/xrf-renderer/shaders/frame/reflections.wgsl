#import "common/camera"
#import "common/octahedral"
#import "common/fullscreen"
#import "common/lighting"
#import "common/hmodel"
#import "common/occlusion"
#import "common/reflection_trace"
#import "common/water_enhanced"

// Stochastic screen-space reflections on the frame's surfaces, at the size the quality traces at, a pixel per `ratio`
// by `ratio` of the frame. Each reflecting pixel's ray leaves in a direction drawn from its surface's GGX lobe (the wet
// film's where it is wet) by blue noise that turns every frame, and walks the frame's nearest depth pyramid
// (`common/reflection_trace`); what it met is lit as the frame shows it, weighed by how sure the hit is, the
// environment `hmodel` reflects otherwise, so a ray meeting nothing changes nothing. Then the
// denoiser: the last frame's reflection reprojected, from where the reflected point or the surface stood; an
// eighth-size average; the trace filtered across the surface, stopping at edges; and the two resolved over time, the
// history held to the neighbourhood, into this frame's history, which combine blends each surface towards by its share.
//
// Each stage marks a pixel nothing is traced at with an alpha below none.

// The G-buffer, its light and occlusion, the material table, the lighting and the settings, the nearest depth pyramid
// and blue noise, the stages before, and the last frame's reflection, surface and held distance.
#import "generated/frame/reflections"

// The sky's cubes, which a ray meeting nothing reflects.
#import "generated/common/sky"

// What a pixel reflects at most, of the environment or as the water over it, below which it is not traced; and the
// roughness past which it reflects the environment alone.
const TRACE_FLOOR: f32 = 0.02;
const ROUGHNESS_LIMIT: f32 = 0.6;
// The baked hemisphere over which a mirror sees the sky its missed rays reflect: none indoors, all of it outdoors; and
// the roughness over which they reflect the environment's blur instead.
const SKY_SEEN: vec2<f32> = vec2<f32>(0.05, 0.35);
const MIRROR_ROUGHNESS: vec2<f32> = vec2<f32>(0.02, 0.15);
// The luminance what a ray meets is held to, so a lamp's glass or a sunlit highlight does not flare what reflects it.
const RADIANCE_LIMIT: f32 = 6.0;
// Metres behind a surface a ray may lie, and what of its distance more, and still have met it; further, it passes
// behind it.
const HIT_THICKNESS: vec2<f32> = vec2<f32>(0.2, 0.008);
// Metres over the plane a ray was reflected off a surface must stand to stop it: a little, and for what stands in a
// puddle, whose level water lies under the bumps of the ground the depth shows, as far as the placing let it rise.
const PLANE_SLACK: vec2<f32> = vec2<f32>(0.02, 0.3);
// The share of the screen a hit fades out over towards its edges.
const EDGE_FADE: f32 = 0.05;
// The length a ray that met nothing is given, so its reflection is carried as the sky's, far off.
const MISSED_LENGTH: f32 = 1e4;
// The R2 sequence, which turns a pixel's noise from one frame to the next.
const NOISE_TURN: vec2<f32> = vec2<f32>(0.7548776662, 0.5698402910);
const LUMINANCE: vec3<f32> = vec3<f32>(0.299, 0.587, 0.114);
const UNTRACED: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, -1.0);

// The denoiser's weights: the neighbourhood's radius and falloff; how much of the average's luminance weighs a pixel;
// how the radiance, variance, normal and depth stop the filter; the disocclusion's normal and depth weights and the
// share below which history is dropped; how close a reprojected normal must be to take the reflected point's; how far
// the surface's history may stray from the neighbourhood; the most frames held; and how wide the history is clipped.
const NEIGHBOURHOOD_RADIUS: i32 = 2;
const GAUSSIAN_K: f32 = 3.0;
const AVERAGE_LUMINANCE_WEIGHT: f32 = 0.3;
const RADIANCE_WEIGHT_BIAS: f32 = 0.6;
const RADIANCE_WEIGHT_VARIANCE: f32 = 0.1;
const PREFILTER_VARIANCE_WEIGHT: f32 = 4.4;
const PREFILTER_VARIANCE_BIAS: f32 = 0.1;
const PREFILTER_NORMAL_SIGMA: f32 = 512.0;
// The share of the distance a tap may stand off the pixel's plane: flat ground passes whole at any angle.
const PREFILTER_PLANE: f32 = 0.02;
const DISOCCLUSION_NORMAL_WEIGHT: f32 = 1.4;
// Stricter than the reference's one where the surface's history is read, as a static world's reprojection has no
// motion vectors' noise to forgive; the reference's where the reflected point's is, held by its virtual distance.
const DISOCCLUSION_DEPTH_WEIGHT: f32 = 4.0;
const DISOCCLUSION_HIT_WEIGHT: f32 = 1.0;
const DISOCCLUSION_THRESHOLD: f32 = 0.9;
const REPROJECTION_NORMAL_SIMILARITY: f32 = 0.9999;
const SURFACE_DISCARD_VARIANCE: f32 = 1.5;
const MOST_SAMPLES: f32 = 32.0;
const HISTORY_CLIP: f32 = 1.1;
// The roughness over which a reflection blurs towards the average while it holds few frames: a mirror keeps its own.
const BLURRED_ROUGHNESS: vec2<f32> = vec2<f32>(0.02, 0.2);
// The up from which flat terrain's normal is taken as level for the denoiser, so the rain's ripples do not reject it.
const LEVEL_UP: f32 = 0.98;
const RADIANCE_THRESHOLD: f32 = 0.0001;

// The first fifteen of Halton(2, 3) stretched over three pixels each way, the centre left out: the prefilter's taps.
const PREFILTER_TAPS: array<vec2<i32>, 15> = array<vec2<i32>, 15>(
  vec2<i32>(0, 1), vec2<i32>(-2, 1), vec2<i32>(2, -3), vec2<i32>(-3, 0), vec2<i32>(1, 2),
  vec2<i32>(-1, -2), vec2<i32>(3, 0), vec2<i32>(-3, 3), vec2<i32>(0, -3), vec2<i32>(-1, -1),
  vec2<i32>(2, 1), vec2<i32>(-2, -2), vec2<i32>(1, 0), vec2<i32>(0, 2), vec2<i32>(3, -1),
);

// The frame's texel a traced pixel stands for: the first of its block that is not a plant.
fn traced_texel(pixel: vec2<f32>) -> vec2<i32> {
  return reflection_texel(material_target, depth_target, pixel, reflection.ratio);
}

// The world's direction for one in view space.
fn world_direction(direction: vec3<f32>) -> vec3<f32> {
  return normalize((transpose(camera.view) * vec4<f32>(direction, 0.0)).xyz);
}

fn luminance(colour: vec3<f32>) -> f32 {
  return max(dot(colour, LUMINANCE), 0.001);
}

// Two uniform numbers for a pixel this frame: the blue noise there and half its tile away, turned by the frame.
fn pixel_numbers(pixel: vec2<f32>) -> vec2<f32> {
  let size: vec2<u32> = textureDimensions(blue_noise);
  let at: vec2<u32> = vec2<u32>(pixel) % size;
  let noise: vec2<f32> = vec2<f32>(
    textureLoad(blue_noise, at, 0).r,
    textureLoad(blue_noise, (at + size / 2u) % size, 0).r,
  );

  return fract(noise + f32(reflection.frame) * NOISE_TURN);
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

  return fogged * min(1.0, RADIANCE_LIMIT / luminance(fogged));
}

// How sure a ray's end is a surface it met: none off the screen, on the sky, or further behind the surface the depth
// shows there than its thickness, where it passed behind something nearer, whose colour smeared down would show as
// slabs, unless the walk found that something solid (`is_behind`), which it then meets; faded towards the screen's edges and out to the distance traced. A ray ending on a surface's back or right
// where it left still counts: a grazing ray off a puddle meets the ground rising just past it, which the puddle's level
// normal shows as a back.
fn hit_confidence(hit: vec3<f32>, origin: vec3<f32>, is_behind: bool) -> f32 {
  let size: vec2<f32> = camera.viewport.xy;

  if (any(hit.xy < vec2<f32>(0.0)) || any(hit.xy > vec2<f32>(1.0))) {
    return 0.0;
  }

  let texel: vec2<i32> = vec2<i32>(hit.xy * size);
  let surface: f32 = textureLoad(depth_target, texel, 0);

  if (surface <= 0.0) {
    return 0.0;
  }

  let shown: vec3<f32> = ray_view_point(vec3<f32>(hit.xy, surface));
  let thickness: f32 = HIT_THICKNESS.x - shown.z * HIT_THICKNESS.y;
  let sure: f32 = select(1.0 - smoothstep(thickness * 0.5, thickness, length(shown - ray_view_point(hit))), 1.0,
    is_behind);

  let met: vec3<f32> = ray_view_point(hit);
  let edges: vec2<f32> = smoothstep(vec2<f32>(0.0), vec2<f32>(EDGE_FADE * size.y / size.x, EDGE_FADE), hit.xy) *
    (1.0 - smoothstep(1.0 - vec2<f32>(EDGE_FADE * size.y / size.x, EDGE_FADE), vec2<f32>(1.0), hit.xy));
  let reach: f32 = 1.0 - smoothstep(reflection.distance * 0.75, reflection.distance, length(met - origin));

  return sure * edges.x * edges.y * reach;
}

@fragment
fn fs_trace(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let pixel: vec2<f32> = floor(in.clip.xy);
  let texel: vec2<i32> = traced_texel(pixel);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    return UNTRACED;
  }

  let material: vec4<f32> = textureLoad(material_target, texel, 0);

  // Grass and leaves reflect the environment as `hmodel` does, and are never traced.
  if (has_mark(material.a, MARK_PLANT)) {
    return UNTRACED;
  }

  let albedo: vec4<f32> = textureLoad(albedo_target, texel, 0);
  let wet: vec4<f32> = textureLoad(wet_surface, texel, 0);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let position: vec3<f32> = camera_view_position(vec2<f32>(texel) + 0.5, depth);
  let toward: vec3<f32> = normalize(position);
  let coat: f32 = coat_roughness(wet.g);
  let water: f32 = wet.r * coat_fresnel(dot(normal, -toward), coat) * reflection.intensity;
  let dry: f32 = hmodel_environment_weight(lighting, material_lut, lut_sampler, albedo.a,
    mix(1.0, material.x, camera.switches.z), world_direction(reflect(toward, normal)), world_direction(toward),
    material.z);

  let sharp: f32 = min(dry, dielectric_fresnel(dot(normal, -toward), reflection_roughness(albedo.a), DIELECTRIC_F0)) *
    sharpened(reflection_roughness(albedo.a));

  if (max(sharp * reflection.intensity, water) <= TRACE_FLOOR) {
    return UNTRACED;
  }

  let roughness: f32 = traced_roughness(albedo.a, wet.g);
  let direction: vec3<f32> = reflected_direction(toward, normal, roughness, pixel_numbers(pixel));
  // What a ray meeting nothing reflects: the environment `hmodel` reflects, blurred; for a mirror, the sky as drawn,
  // clouds and all, where the surface sees it.
  let blurred: vec3<f32> = hmodel_reflected_environment(lighting, sky_environment_0, sky_environment_1, sky_clamp,
    world_direction(direction));
  let drawn: vec3<f32> = enhanced_sky(world_direction(direction), lighting, sky_cube_0, sky_cube_1, sky_clamp) *
    smoothstep(SKY_SEEN.x, SKY_SEEN.y, material.x);
  let sky: vec3<f32> = mix(drawn, blurred, smoothstep(MIRROR_ROUGHNESS.x, MIRROR_ROUGHNESS.y, roughness));

  if (roughness > ROUGHNESS_LIMIT) {
    return vec4<f32>(sky, MISSED_LENGTH);
  }

  // The ray's end, the distance traced out, or short of the near plane where it comes back towards the camera: its
  // line across the screen is as long as its reach, so a reach as short as the surface is near cut a puddle's
  // reflection off at a line that moved with the camera.
  let near: f32 = camera.projection[3][2] / (1.0 + camera.projection[2][2]);
  let reach: f32 = select(reflection.distance, min(reflection.distance, (-position.z - near * 1.05) / direction.z),
    direction.z > 0.0);
  let start: vec2<f32> = (vec2<f32>(texel) + 0.5) / camera.viewport.xy;
  let origin: vec3<f32> = vec3<f32>(start, depth);
  let toward_screen: vec3<f32> = ray_screen_point(position + direction * max(reach, 1e-3)) - origin;
  // The plane it was reflected off, which nothing beneath can stop it: a puddle's level water standing some way under
  // the ground its depth shows, anything else's own surface.
  let slack: vec2<f32> = vec2<f32>(PLANE_SLACK.x, select(PLANE_SLACK.x, PLANE_SLACK.y, wet.g > PUDDLE_LEAST));
  let walk: RayWalk = ray_walk(nearest_depth, material_target, wet_surface, origin, toward_screen,
    reflection.crossings, HIT_THICKNESS, vec4<f32>(normal, dot(normal, position)), slack);

  let confidence: f32 = select(0.0, hit_confidence(walk.position, position, walk.is_behind), walk.is_met);

  if (confidence <= 0.0) {
    return vec4<f32>(sky, MISSED_LENGTH);
  }

  let met: vec3<f32> = met_radiance(vec2<i32>(walk.position.xy * camera.viewport.xy));
  let length_met: f32 = length(ray_view_point(walk.position) - position);

  return vec4<f32>(mix(sky, met, confidence), select(MISSED_LENGTH, length_met, confidence > 0.5));
}

// A traced pixel's surface: its world normal, roughness and distance along the view, as the history keeps them.
struct TracedSurface {
  normal: vec3<f32>,
  roughness: f32,
  distance: f32,
  world: vec3<f32>,
  uv: vec2<f32>,
};

fn traced_surface(pixel: vec2<f32>) -> TracedSurface {
  let texel: vec2<i32> = traced_texel(pixel);
  let depth: f32 = max(textureLoad(depth_target, texel, 0), 1e-7);
  let normal: vec3<f32> = world_direction(octahedral_decode(textureLoad(normal_target, texel, 0).xy));
  let is_level: bool = has_mark(textureLoad(material_target, texel, 0).a, MARK_TERRAIN) && normal.y > LEVEL_UP;
  let uv: vec2<f32> = (vec2<f32>(texel) + 0.5) / camera.viewport.xy;
  let world: vec3<f32> = camera_unproject(vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0), depth);

  let wet: vec4<f32> = textureLoad(wet_surface, texel, 0);
  let roughness: f32 = traced_roughness(textureLoad(albedo_target, texel, 0).a, wet.g);

  return TracedSurface(select(normal, vec3<f32>(0.0, 1.0, 0.0), is_level), roughness,
    -camera_view_position(vec2<f32>(texel) + 0.5, depth).z, world, uv);
}

// Where a world point stood on the last frame's screen, in texture coordinates; off it where it stood behind it.
fn previous_uv(world: vec3<f32>) -> vec2<f32> {
  let clip: vec4<f32> = camera.motion_previous * vec4<f32>(world, 1.0);

  return select(vec2<f32>(-1.0), (clip.xy / clip.w) * vec2<f32>(0.5, -0.5) + 0.5, clip.w > 0.0);
}

// How much of the history at a place to trust, by how its normal and distance match this frame's.
fn disocclusion(normal: vec3<f32>, held_normal: vec3<f32>, distance: f32, held_distance: f32, weight: f32) -> f32 {
  return exp(-abs(1.0 - max(0.0, dot(normal, held_normal))) * DISOCCLUSION_NORMAL_WEIGHT) *
    exp(-abs(held_distance - distance) / max(distance, 1e-3) * weight);
}

// The last frame's held distance, frame count and virtual distance nearest a place.
fn held_at(uv: vec2<f32>) -> vec4<f32> {
  let size: vec2<f32> = vec2<f32>(textureDimensions(history_held));

  return textureLoad(history_held, vec2<i32>(clamp(floor(uv * size), vec2<f32>(0.0), size - 1.0)), 0);
}

fn history_surface_at(uv: vec2<f32>) -> vec4<f32> {
  return textureSampleLevel(history_surface, history_sampler, uv, 0.0);
}

fn history_radiance_at(uv: vec2<f32>) -> vec4<f32> {
  return textureSampleLevel(history_radiance, history_sampler, uv, 0.0);
}

// A Gaussian's weight `offset` pixels out over the neighbourhood.
fn neighbourhood_weight(offset: f32) -> f32 {
  let radius: f32 = f32(NEIGHBOURHOOD_RADIUS) + 1.0;

  return exp(-GAUSSIAN_K * offset * offset / (radius * radius));
}

// The mean and variance of a target's colour about a pixel, its untraced pixels left out.
struct Moments {
  mean: vec3<f32>,
  variance: vec3<f32>,
};

fn neighbourhood(source: texture_2d<f32>, pixel: vec2<f32>) -> Moments {
  let last: vec2<i32> = vec2<i32>(textureDimensions(source)) - 1;
  var mean: vec3<f32> = vec3<f32>(0.0);
  var squares: vec3<f32> = vec3<f32>(0.0);
  var weights: f32 = 0.0;

  for (var y: i32 = -NEIGHBOURHOOD_RADIUS; y <= NEIGHBOURHOOD_RADIUS; y++) {
    for (var x: i32 = -NEIGHBOURHOOD_RADIUS; x <= NEIGHBOURHOOD_RADIUS; x++) {
      let taken: vec4<f32> = textureLoad(source, clamp(vec2<i32>(pixel) + vec2<i32>(x, y), vec2<i32>(0), last), 0);
      let weight: f32 = select(0.0, neighbourhood_weight(f32(x)) * neighbourhood_weight(f32(y)), taken.a >= 0.0);

      mean += taken.rgb * weight;
      squares += taken.rgb * taken.rgb * weight;
      weights += weight;
    }
  }

  let safe: f32 = max(weights, 1e-4);

  return Moments(mean / safe, abs(squares / safe - (mean / safe) * (mean / safe)));
}

// How far the history's luminance strays from this frame's, squared and relative.
fn temporal_variance(held: vec3<f32>, fresh: vec3<f32>) -> f32 {
  let before: f32 = luminance(held);
  let now: f32 = luminance(fresh);
  let difference: f32 = abs(before - now) / max(max(before, now), 0.5);

  return difference * difference;
}

// The history reprojected and how many frames it holds, then its variance.
struct ReprojectedTargets {
  @location(0) reprojected: vec4<f32>,
  @location(1) variance: f32,
};

// The last frame's reflection at a traced pixel: read where the reflected point stood (the view's ray carried on through
// the surface by the hit's length) where the normals there match as a mirror's would, else where the surface stood if
// it keeps near the neighbourhood; searched about where the history disagrees, dropped where it still does. Holds more
// frames the rougher the surface.
@fragment
fn fs_reproject(in: FullscreenVarying) -> ReprojectedTargets {
  let pixel: vec2<f32> = floor(in.clip.xy);
  let fresh: vec4<f32> = textureLoad(traced, vec2<i32>(pixel), 0);
  if (fresh.a < 0.0) {
    return ReprojectedTargets(vec4<f32>(0.0), 0.0);
  }

  let surface: TracedSurface = traced_surface(pixel);
  // Without history a rough reflection is all noise, a mirror's little.
  let dropped: ReprojectedTargets = ReprojectedTargets(vec4<f32>(0.0, 0.0, 0.0, 1.0),
    mix(0.1, 1.0, smoothstep(BLURRED_ROUGHNESS.x, BLURRED_ROUGHNESS.y, surface.roughness)));

  if (reflection.has_history < 0.5) {
    return dropped;
  }

  let local: Moments = neighbourhood(traced, pixel);
  let toward: vec3<f32> = surface.world - camera.position.xyz;
  let through: vec3<f32> = camera.position.xyz + normalize(toward) * (length(toward) + fresh.a);
  let surface_uv: vec2<f32> = previous_uv(surface.world);
  let hit_uv: vec2<f32> = previous_uv(through);
  let surface_held: vec4<f32> = history_surface_at(surface_uv);
  let hit_held: vec4<f32> = history_surface_at(hit_uv);
  let hit_similarity: f32 = dot(normalize(hit_held.xyz + 1e-6), surface.normal);
  let surface_similarity: f32 = dot(normalize(surface_held.xyz + 1e-6), surface.normal);
  var uv: vec2<f32> = surface_uv;
  var held_normal: vec3<f32> = surface_held.xyz;
  var reprojection: vec3<f32> = history_radiance_at(surface_uv).rgb;
  // The surface's history is matched by its distance; the reflected point's by the distance through the surface to it.
  var distance: f32 = surface.distance;
  var channel: u32 = 0u;
  var weight: f32 = DISOCCLUSION_DEPTH_WEIGHT;

  if (hit_similarity > REPROJECTION_NORMAL_SIMILARITY && hit_similarity + 1e-3 > surface_similarity &&
    abs(hit_held.w - surface.roughness) < abs(surface_held.w - surface.roughness) + 1e-3) {
    uv = hit_uv;
    held_normal = hit_held.xyz;
    reprojection = history_radiance_at(hit_uv).rgb;
    distance = surface.distance + fresh.a;
    channel = 2u;
    weight = DISOCCLUSION_HIT_WEIGHT;
  } else {
    let away: vec3<f32> = reprojection - local.mean;

    if (dot(away, away) >= SURFACE_DISCARD_VARIANCE * length(local.variance)) {
      return dropped;
    }
  }

  if (any(uv <= vec2<f32>(0.0)) || any(uv >= vec2<f32>(1.0))) {
    return dropped;
  }

  var trust: f32 = disocclusion(surface.normal, held_normal, distance, held_at(uv)[channel], weight);

  // Not sure of a disocclusion: the nearest match a history texel about it.
  if (trust < DISOCCLUSION_THRESHOLD) {
    let cell: vec2<f32> = 1.0 / vec2<f32>(textureDimensions(history_held));
    let centre: vec2<f32> = uv;

    for (var y: i32 = -1; y <= 1; y++) {
      for (var x: i32 = -1; x <= 1; x++) {
        let near: vec2<f32> = centre + vec2<f32>(f32(x), f32(y)) * cell;
        let match_near: f32 = disocclusion(surface.normal, history_surface_at(near).xyz, distance,
          held_at(near)[channel], weight);

        if (match_near > trust) {
          trust = match_near;
          uv = near;
        }
      }
    }

    reprojection = history_radiance_at(uv).rgb;
  }

  if (trust < DISOCCLUSION_THRESHOLD) {
    return dropped;
  }

  // A mirror's ray is the same every frame, so its history would only lag behind; a rough one's is noise, held long.
  let most: f32 = select(max(8.0, MOST_SAMPLES * (1.0 - exp(-surface.roughness * 100.0))), 1.0,
    surface.roughness < MIRROR_ROUGHNESS.x);
  let samples: f32 = min(most, held_at(uv).y * trust + 1.0);
  let variance: f32 = mix(temporal_variance(fresh.rgb, reprojection), max(history_radiance_at(uv).a, 0.0), 1.0 /
    samples);

  return ReprojectedTargets(vec4<f32>(reprojection, samples), variance);
}

// The traced reflection averaged over each eight by eight block, weighed against bright pixels, a third of the way to
// the reprojected history where it holds one.
@fragment
fn fs_average(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let base: vec2<i32> = vec2<i32>(floor(in.clip.xy)) * 8;
  let last: vec2<i32> = vec2<i32>(textureDimensions(traced)) - 1;
  var sum: vec3<f32> = vec3<f32>(0.0);
  var weights: f32 = 0.0;

  for (var y: i32 = 0; y < 8; y++) {
    for (var x: i32 = 0; x < 8; x++) {
      let at: vec2<i32> = base + vec2<i32>(x, y);

      if (any(at > last)) {
        continue;
      }

      let fresh: vec4<f32> = textureLoad(traced, at, 0);
      let held: vec4<f32> = textureLoad(reprojected, at, 0);
      let radiance: vec3<f32> = select(fresh.rgb, mix(fresh.rgb, held.rgb, 0.3), held.a > 1.0);
      let weight: f32 = max(exp(-luminance(radiance) * AVERAGE_LUMINANCE_WEIGHT), 1e-2);

      if (fresh.a >= 0.0 && all(radiance == radiance) && weight <= 1e3) {
        sum += radiance * weight;
        weights += weight;
      }
    }
  }

  return vec4<f32>(sum / max(weights, 1e-3), 1.0);
}

// The eighth-size average at a traced pixel, bilinear.
fn average_at(pixel: vec2<f32>) -> vec3<f32> {
  let size: vec2<f32> = vec2<f32>(textureDimensions(average)) * 8.0;

  return textureSampleLevel(average, history_sampler, (pixel + 0.5) / size, 0.0).rgb;
}

// How much a neighbour's radiance weighs against the average, the less the more it differs, by the variance.
fn radiance_weight(around: vec3<f32>, neighbour: vec3<f32>, variance: f32) -> f32 {
  return max(exp(-(RADIANCE_WEIGHT_BIAS + variance * RADIANCE_WEIGHT_VARIANCE) * length(around - neighbour)), 1e-2);
}

// The trace filtered across fifteen taps about each pixel, each weighed by how near its normal and distance lie to the
// pixel's and its radiance to the average, the more the noisier the pixel; mirrors and pixels without variance as
// traced. Then its variance.
@fragment
fn fs_prefilter(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let pixel: vec2<f32> = floor(in.clip.xy);
  let centre: vec4<f32> = textureLoad(traced, vec2<i32>(pixel), 0);

  if (centre.a < 0.0) {
    return UNTRACED;
  }

  let spread: f32 = textureLoad(variance, vec2<i32>(pixel), 0).x;
  let surface: TracedSurface = traced_surface(pixel);

  if (spread <= 0.0 || surface.roughness < 1e-4 || surface.roughness > ROUGHNESS_LIMIT) {
    return vec4<f32>(centre.rgb, max(spread, 0.0));
  }

  let around: vec3<f32> = average_at(pixel);
  let last: vec2<i32> = vec2<i32>(textureDimensions(traced)) - 1;
  // A near-mirror is never spread across its neighbours.
  let noisy: f32 = max(PREFILTER_VARIANCE_BIAS, 1.0 - exp(-spread * PREFILTER_VARIANCE_WEIGHT)) *
    smoothstep(BLURRED_ROUGHNESS.x, BLURRED_ROUGHNESS.y, surface.roughness);
  var weights: f32 = radiance_weight(around, centre.rgb, spread);
  var sum: vec3<f32> = centre.rgb * weights;
  var variances: f32 = spread * weights * weights;

  for (var tap: u32 = 0u; tap < 15u; tap++) {
    let at: vec2<i32> = clamp(vec2<i32>(pixel) + PREFILTER_TAPS[tap], vec2<i32>(0), last);
    let neighbour: vec4<f32> = textureLoad(traced, at, 0);

    if (neighbour.a < 0.0) {
      continue;
    }

    let other: TracedSurface = traced_surface(vec2<f32>(at));
    let off_plane: f32 = abs(dot(surface.normal, other.world - surface.world));
    let weight: f32 = pow(max(dot(surface.normal, other.normal), 0.0), PREFILTER_NORMAL_SIGMA) *
      exp(-off_plane / max(surface.distance * PREFILTER_PLANE, 1e-3)) *
      radiance_weight(around, neighbour.rgb, spread) * noisy;

    weights += weight;
    sum += neighbour.rgb * weight;
    variances += weight * weight * textureLoad(variance, at, 0).x;
  }

  return vec4<f32>(sum / weights, variances / (weights * weights));
}

// This frame's history: the reflection, its variance; the surface's world normal and roughness; its distance along the
// view, the frames its reflection holds, and the distance through it to what it reflects.
struct ResolvedTargets {
  @location(0) radiance: vec4<f32>,
  @location(1) surface: vec4<f32>,
  @location(2) held: vec4<f32>,
};

// The prefiltered reflection blended with the reprojected history by the frames it holds, the history first clipped
// to the neighbourhood's spread about its mean (Playdead's box clip) and the new pulled towards the average while it
// holds few.
@fragment
fn fs_resolve(in: FullscreenVarying) -> ResolvedTargets {
  let pixel: vec2<f32> = floor(in.clip.xy);
  let centre: vec4<f32> = textureLoad(prefiltered, vec2<i32>(pixel), 0);

  if (centre.a < 0.0) {
    return ResolvedTargets(UNTRACED, vec4<f32>(0.0), vec4<f32>(0.0));
  }

  let surface: TracedSurface = traced_surface(pixel);
  let held: vec4<f32> = textureLoad(reprojected, vec2<i32>(pixel), 0);
  let length_met: f32 = max(textureLoad(traced, vec2<i32>(pixel), 0).a, 0.0);
  let kept: ResolvedTargets = ResolvedTargets(vec4<f32>(0.0), vec4<f32>(surface.normal, surface.roughness),
    vec4<f32>(surface.distance, held.a, surface.distance + length_met, 0.0));

  if (centre.r + centre.g + centre.b < RADIANCE_THRESHOLD) {
    return kept;
  }

  let samples: f32 = max(held.a, 1.0);
  let around: vec3<f32> = average_at(pixel);
  let local: Moments = neighbourhood(prefiltered, pixel);
  let spread: vec3<f32> = (sqrt(local.variance) + length(local.mean - around)) * HISTORY_CLIP * 1.4;
  let mean: vec3<f32> = mix(local.mean, around, 0.2);
  let clipped: vec3<f32> = clip_box(mean - spread, mean + spread, held.rgb);
  let weight: f32 = 1.0 - 1.0 / samples;
  let blurred: f32 = smoothstep(BLURRED_ROUGHNESS.x, BLURRED_ROUGHNESS.y, surface.roughness);
  var fresh: vec3<f32> = mix(centre.rgb, around, blurred / (samples + 1.0));

  fresh = mix(fresh, clip_box(around - spread, around + spread, fresh), blurred);

  var resolved: vec3<f32> = mix(fresh, clipped, weight);
  var variance: f32 = mix(temporal_variance(resolved, clipped), centre.a, weight);

  if (any(resolved != resolved) || variance != variance) {
    resolved = vec3<f32>(0.0);
    variance = 0.0;
  }

  return ResolvedTargets(vec4<f32>(resolved, variance), kept.surface, kept.held);
}

// A colour clipped towards a box's centre onto its surface where it lies outside it.
fn clip_box(least: vec3<f32>, most: vec3<f32>, colour: vec3<f32>) -> vec3<f32> {
  let centre: vec3<f32> = 0.5 * (most + least);
  let extent: vec3<f32> = 0.5 * (most - least) + 0.001;
  let away: vec3<f32> = colour - centre;
  let units: vec3<f32> = abs(away / extent);
  let furthest: f32 = max(max(units.x, units.y), units.z);

  return select(colour, centre + away / furthest, furthest > 1.0);
}
