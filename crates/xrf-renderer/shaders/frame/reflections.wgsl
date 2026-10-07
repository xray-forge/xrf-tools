#import "common/camera"
#import "common/octahedral"
#import "common/fullscreen"
#import "common/lighting"
#import "common/hmodel"
#import "common/occlusion"
#import "common/reflection_trace"
#import "common/reflection_end"
#import "common/scene_trace"

// Stochastic screen-space reflections on the frame's surfaces, at the size the quality traces at, a pixel per `ratio`
// by `ratio` of the frame. Each reflecting pixel's ray leaves in a direction drawn from its surface's GGX lobe (a
// puddle's water's, from the water's own height over the ground) by blue noise that turns every frame, and walks the
// nearest depth pyramid of the frame as it stood before the grass drew (`traced_depth`): a bush's or a tuft's
// billboards stand in for a volume a reflection sees through, so they never stop a ray, and what lies behind them
// answers; the trees' leaves are the static scene's and stand
// (`common/reflection_trace`); what it met is lit as the frame shows it, faded towards the screen's edges and the
// distance traced, the sky or the environment `hmodel` reflects otherwise. Then the denoiser: the last frame's
// reflection reprojected, from where the reflected point or the surface stood; an eighth-size average; the trace
// filtered across the surface, stopping at edges; and the two resolved over time, the history held to the
// neighbourhood, into this frame's history, which combine blends by `reflection_slice` and `coat_share`.
//
// Each stage marks a pixel nothing is traced at with an alpha below none.

// The G-buffer, its light and occlusion, the material table, the lighting and the settings, the nearest depth pyramid
// and blue noise, the stages before, and the last frame's reflection, surface and held distance.
#import "generated/frame/reflections"

// The sky's cubes, which a ray meeting nothing reflects.
#import "generated/common/sky"

// The share of what it reflects a pixel's specular shows at most, below which it is not traced.
const TRACE_FLOOR: f32 = 0.02;
// The roughness under which a surface is a mirror, whose ray is the same every frame and holds no history: under a
// puddle's water at rest, which holds a few.
const MIRROR_ROUGHNESS: f32 = 0.01;
// The frames a near-mirror's reflection holds: its ray hardly moves, and its hit point reprojects, so a few steady it
// without lagging a moving camera.
const MIRROR_FRAMES: f32 = 4.0;
// The share of the roughness's mip a missed ray reads the prefiltered sky at: its lobe spreads the rays already, and
// the denoiser gathers them, so the cube supplies the other half of the blur.
const MISS_BLUR: f32 = 0.5;
// The luminance what a ray meets is held to, so a lamp's glass or a sunlit highlight does not flare what reflects it.
const RADIANCE_LIMIT: f32 = 6.0;
// Metres behind a surface a ray may lie, and what of its distance more, and still have met it: thin, so a leaf stays a
// leaf and a ray aimed past its edge goes past it; further behind, the walk ends uncertain. Within `PROBABLE_BEHIND`
// more (metres, then a share of the distance: a wall met at a glancing step) the surface it stands behind is the
// probable answer, and its colour is taken with a confidence fading over that depth; beyond it the ray has passed
// something standing between, and the miss chain answers. Behind a plant the band is a bush's depth
// (`PROBABLE_BEHIND_PLANT`): its blades are billboards standing in for a volume, a ray from the ground under a bush
// leaves a metre behind its front, and behind a blade is mostly more bush.
const HIT_THICKNESS: vec2<f32> = vec2<f32>(0.05, 0.005);
const PROBABLE_BEHIND: vec2<f32> = vec2<f32>(0.5, 0.03);
const PROBABLE_BEHIND_PLANT: vec2<f32> = vec2<f32>(1.5, 0.05);
// How far along the ray the met surface's own normal may point before the hit is a back face: faded out from none to
// this, since the depth's normal at an edge leans.
const BACK_FACING: f32 = 0.2;
// The share of the screen a hit fades out over towards its edges.
const EDGE_FADE: f32 = 0.05;
// The length a ray that met nothing is given, so its reflection is carried as the sky's, far off.
const MISSED_LENGTH: f32 = 1e4;
// Metres a traced ray starts off its surface, and what of its distance more, so the triangles the G-buffer's depth
// stands a little off of do not meet it.
const TRACED_BIAS: vec2<f32> = vec2<f32>(0.02, 0.002);
// Metres a ray to the sun from a traced hit starts off it, and reaches at most.
const TRACED_SUN_BIAS: f32 = 0.05;
const TRACED_SUN_REACH: f32 = 1000.0;
// The R2 sequence, which turns a pixel's noise from one frame to the next.
const NOISE_TURN: vec2<f32> = vec2<f32>(0.7548776662, 0.5698402910);
const LUMINANCE: vec3<f32> = vec3<f32>(0.299, 0.587, 0.114);
const UNTRACED: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, -1.0);
// The roughness over which a ray's crossings fall to `ROUGH_CROSSINGS` of the quality's, its blurred reflection needing
// no finer walk.
const ROUGH_BUDGET: vec2<f32> = vec2<f32>(0.15, 0.45);
const ROUGH_CROSSINGS: f32 = 0.5;

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
// The up from which flat terrain's normal is taken as level for the denoiser, so its bumps do not reject the history;
// a puddle's always is, whatever its rings bend its normal to.
const LEVEL_UP: f32 = 0.98;
const RADIANCE_THRESHOLD: f32 = 0.0001;

// The first fifteen of Halton(2, 3) stretched over three pixels each way, the centre left out: the prefilter's taps.
const PREFILTER_TAPS: array<vec2<i32>, 15> = array<vec2<i32>, 15>(
  vec2<i32>(0, 1), vec2<i32>(-2, 1), vec2<i32>(2, -3), vec2<i32>(-3, 0), vec2<i32>(1, 2),
  vec2<i32>(-1, -2), vec2<i32>(3, 0), vec2<i32>(-3, 3), vec2<i32>(0, -3), vec2<i32>(-1, -1),
  vec2<i32>(2, 1), vec2<i32>(-2, -2), vec2<i32>(1, 0), vec2<i32>(0, 2), vec2<i32>(3, -1),
);

// The frame's texel a traced pixel stands for this frame: the first of its block that is not a plant, from the cell
// the frame turns to.
fn traced_texel(pixel: vec2<f32>) -> vec2<i32> {
  return reflection_texel(material_target, depth_target, pixel, reflection.ratio, reflection.frame);
}

// The world's direction for one in view space.
fn world_direction(direction: vec3<f32>) -> vec3<f32> {
  return normalize((transpose(camera.view) * vec4<f32>(direction, 0.0)).xyz);
}

// A view space point's offset from the camera in the world's frame, metres.
fn world_offset(position: vec3<f32>) -> vec3<f32> {
  return (transpose(camera.view) * vec4<f32>(position, 0.0)).xyz;
}

// What the level's static geometry shows along a ray the screen could not answer: traced by ray query from the
// surface, the hit lit as combine lights it, its sun through a second ray rather than the shadow maps, its fog along
// the way taken as a met point's is; the ray's length in `w`, below none where it met nothing or nothing is traced.
fn traced_radiance(position: vec3<f32>, normal: vec3<f32>, direction: vec3<f32>, roughness: f32) -> vec4<f32> {
  if (!IS_SCENE_TRACED || reflection.is_traced < 0.5) {
    return vec4<f32>(0.0, 0.0, 0.0, -1.0);
  }

  let world_normal: vec3<f32> = world_direction(normal);
  let toward: vec3<f32> = world_direction(direction);
  let distance: f32 = -position.z;
  let origin: vec3<f32> = camera.position.xyz + world_offset(position) +
    world_normal * (TRACED_BIAS.x + distance * TRACED_BIAS.y);
  // The ray's cone: a traced pixel's width where it leaves the surface, widening by the pixel's angle and its lobe's.
  let pixel_angle: f32 = 2.0 * reflection.ratio / (camera.projection[1][1] * camera.viewport.y);
  let cone: SceneCone = SceneCone(distance * pixel_angle, pixel_angle + roughness);
  let hit: SceneHit = trace_scene_ray(origin, toward, 0.0, reflection.distance, cone);

  if (hit.distance < 0.0) {
    return vec4<f32>(0.0, 0.0, 0.0, -1.0);
  }

  let met: SceneSurface = scene_surface(hit, cone.width + cone.spread * hit.distance);
  let facing: vec3<f32> = select(met.normal, -met.normal, dot(met.normal, toward) > 0.0);
  let to_sun: vec3<f32> = world_direction(lighting.to_sun.xyz);
  let half_way: vec3<f32> = normalize(to_sun - toward);
  // `plight_infinity` as the sun pass lights a surface, its shadow a ray to the sun.
  let lit: vec4<f32> = textureSampleLevel(material_lut, lut_sampler,
    vec3<f32>(dot(to_sun, facing), dot(half_way, facing), met.slice), 0.0);
  var sun: vec4<f32> = vec4<f32>(0.0);

  // The sun's ray reads the holes it passes as coarsely as the reflection read the hit.
  let sun_cone: SceneCone = SceneCone(cone.width + cone.spread * hit.distance, pixel_angle);

  if (lit.x > 0.0 && !trace_scene_blocked(met.position + facing * TRACED_SUN_BIAS, to_sun, TRACED_SUN_REACH, sun_cone)) {
    sun = vec4<f32>(lighting.sun.rgb * lit.x, lighting.sun.w * lit.y);
  }

  let occlusion: f32 = mix(1.0, met.hemi, camera.switches.z);
  let shown: vec3<f32> = hmodel(lighting, material_lut, lut_sampler, sky_environment_0, sky_environment_1, sky_clamp,
    vec4<f32>(met.albedo, 0.0), sun, facing, toward, met.slice, occlusion, vec3<f32>(1.0));
  let held: vec3<f32> = shown * min(1.0, RADIANCE_LIMIT / luminance(shown));
  let fog: f32 = path_fog(hit.distance);

  return vec4<f32>(mix(held, lighting.fog_color.rgb, fog), hit.distance);
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

// What a ray met lit as combine lights it this frame: `hmodel` over its G-buffer, its occlusion where it is searched.
// The fallback where the last frame did not show the point: no water, glass or particle stands in the G-buffer.
fn relit_radiance(texel: vec2<i32>) -> vec3<f32> {
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

  return hmodel(lighting, material_lut, lut_sampler, sky_environment_0, sky_environment_1, sky_clamp, albedo, light,
    world_direction(normal), world_direction(normalize(position)), material.z, occlusion, bounced);
}

// Fog along a path of `length` metres between two points seen, none of the camera's own offset: what a reflection
// gathers on its way from what it reflects to the surface, which combine then fogs from the camera as it fogs the
// surface.
fn path_fog(length: f32) -> f32 {
  return saturate(length * lighting.fog.y) * lighting.fog_color.w;
}

// The share of its distance the depth kept may stand off a point for the last frame to have shown it.
const SOURCE_DEPTH_TOLERANCE: f32 = 0.03;
// The fog past which a frame's colour holds too little of the point to take the fog off.
const SOURCE_FOG_LIMIT: f32 = 0.98;

// What the last frame showed where a hit's point stood, as light, and in `w` one where it did: the point taken through
// that frame's jittered camera, moved back by the sway the motion target holds over the camera's own motion, read
// where the depth kept agrees within a share of the distance, the tonemap undone at the scale that frame used and
// that frame's fog to the point taken off. None where the point was off that screen, in front of or behind what it
// showed, lost in its fog, or where no frame is kept yet.
fn remembered_radiance(texel: vec2<i32>, hit: vec3<f32>) -> vec4<f32> {
  if (reflection.has_source < 0.5) {
    return vec4<f32>(0.0);
  }

  let size: vec2<f32> = camera.viewport.xy;
  let uv: vec2<f32> = (vec2<f32>(texel) + 0.5) / size;
  let depth: f32 = textureLoad(traced_depth, texel, 0);
  let world: vec3<f32> = camera_unproject(vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0), depth);
  let previous: vec4<f32> = camera.view_projection_previous * vec4<f32>(world, 1.0);

  if (previous.w <= 0.0) {
    return vec4<f32>(0.0);
  }

  let sway: vec2<f32> = textureLoad(motion_target, texel, 0).xy - camera_motion(world, world);
  let kept_uv: vec2<f32> = (previous.xy / previous.w) * vec2<f32>(0.5, -0.5) + 0.5 - sway;

  if (any(kept_uv < vec2<f32>(0.0)) || any(kept_uv > vec2<f32>(1.0))) {
    return vec4<f32>(0.0);
  }

  let kept_texel: vec2<i32> = clamp(vec2<i32>(kept_uv * size), vec2<i32>(0), vec2<i32>(size) - 1);
  let kept: f32 = textureLoad(source_depth, kept_texel, 0);

  if (kept <= 0.0) {
    return vec4<f32>(0.0);
  }

  // The depth kept as a distance, through the perspective's depth terms, which the jitter leaves alone.
  let kept_distance: f32 = camera.projection[3][2] / (kept + camera.projection[2][2]);

  if (abs(kept_distance - previous.w) > SOURCE_DEPTH_TOLERANCE * previous.w) {
    return vec4<f32>(0.0);
  }

  let fog: f32 = fog_amount_at(lighting, hit, world.y);

  if (fog > SOURCE_FOG_LIMIT) {
    return vec4<f32>(0.0);
  }

  let scale: f32 = select(1.0, exposure.used, lighting.params.z > 0.5);
  let shown: vec3<f32> = untonemapped(textureSampleLevel(source_scene, history_sampler, kept_uv, 0.0).rgb, scale);
  let lit: vec3<f32> = (shown - fog_color_at(lighting, hit, world.y) * fog) / (1.0 - fog);

  return vec4<f32>(max(lit, vec3<f32>(0.0)), 1.0);
}

// What the last frame showed of the sky in a world direction, as light, and in `w` one where it did: the direction
// through that frame's camera, read where that frame's depth shows sky or ground lost in its fog, the tonemap undone
// at the scale that frame used. None off that screen, where it showed something nearer, or where no frame is kept.
fn remembered_sky(direction: vec3<f32>) -> vec4<f32> {
  if (reflection.has_source < 0.5) {
    return vec4<f32>(0.0);
  }

  let previous: vec4<f32> = camera.view_projection_previous * vec4<f32>(direction, 0.0);

  if (previous.w <= 0.0) {
    return vec4<f32>(0.0);
  }

  let kept_uv: vec2<f32> = (previous.xy / previous.w) * vec2<f32>(0.5, -0.5) + 0.5;

  if (any(kept_uv < vec2<f32>(0.0)) || any(kept_uv > vec2<f32>(1.0))) {
    return vec4<f32>(0.0);
  }

  let size: vec2<f32> = camera.viewport.xy;
  let kept_texel: vec2<i32> = clamp(vec2<i32>(kept_uv * size), vec2<i32>(0), vec2<i32>(size) - 1);
  let kept: f32 = textureLoad(source_depth, kept_texel, 0);

  if (kept > 0.0 && path_fog(camera.projection[3][2] / (kept + camera.projection[2][2])) < SOURCE_FOG_LIMIT) {
    return vec4<f32>(0.0);
  }

  let scale: f32 = select(1.0, exposure.used, lighting.params.z > 0.5);

  return vec4<f32>(sky_as_light(textureSampleLevel(source_scene, history_sampler, kept_uv, 0.0).rgb, scale,
    lighting.reflections.w), 1.0);
}

// What a ray met looks like as light from the surface it was reflected off: what the last frame showed there, or where
// that is not kept, its G-buffer relit this frame, and in `w` one where it was the last frame's; fogged along the ray
// from the surface, its luminance held to `RADIANCE_LIMIT` so a lamp's glass or a sunlit highlight does not flare what
// reflects it.
fn met_radiance(texel: vec2<i32>, hit: vec3<f32>, origin: vec3<f32>) -> vec4<f32> {
  let remembered: vec4<f32> = remembered_radiance(texel, hit);
  var lit: vec3<f32>;

  if (remembered.w > 0.5) {
    lit = remembered.rgb;
  } else {
    lit = relit_radiance(texel);
  }

  let fogged: vec3<f32> = mix(lit, lighting.fog_color.rgb, path_fog(length(hit - origin)));

  return vec4<f32>(fogged * min(1.0, RADIANCE_LIMIT / luminance(fogged)), remembered.w);
}

// The view space point the walked depth shows at a frame texel, held to the frame.
fn shown_point(texel: vec2<i32>) -> vec3<f32> {
  let held: vec2<i32> = clamp(texel, vec2<i32>(0), vec2<i32>(camera.viewport.xy) - 1);

  return camera_view_position(vec2<f32>(held) + 0.5, max(textureLoad(traced_depth, held, 0), 1e-7));
}

// The view space normal of the surface the depth shows at a texel, from the points about it, the nearer neighbour
// each way so an edge does not lean it; towards the camera. The G-buffer's normal is bumped by tens of degrees, which
// would turn a facing test into speckle.
fn depth_normal(texel: vec2<i32>, point: vec3<f32>) -> vec3<f32> {
  let right: vec3<f32> = shown_point(texel + vec2<i32>(1, 0)) - point;
  let left: vec3<f32> = point - shown_point(texel - vec2<i32>(1, 0));
  let down: vec3<f32> = shown_point(texel + vec2<i32>(0, 1)) - point;
  let up: vec3<f32> = point - shown_point(texel - vec2<i32>(0, 1));
  let across: vec3<f32> = select(left, right, dot(right, right) < dot(left, left));
  let along: vec3<f32> = select(up, down, dot(down, down) < dot(up, up));
  let normal: vec3<f32> = normalize(cross(across, along));

  return select(normal, -normal, dot(normal, point) > 0.0);
}

// How sure a ray's end is a surface it met: none off the screen or on the sky; faded by how far behind the surface it
// stood of the thickness, out where the surface's own normal points along the ray (a back face, which no reflection
// shows), towards the screen's edges and out to the distance traced.
fn hit_confidence(walk: RayWalk, origin: vec3<f32>, reach: f32) -> f32 {
  let size: vec2<f32> = camera.viewport.xy;
  let hit: vec3<f32> = walk.position;

  if (any(hit.xy < vec2<f32>(0.0)) || any(hit.xy > vec2<f32>(1.0))) {
    return 0.0;
  }

  let texel: vec2<i32> = vec2<i32>(hit.xy * size);
  let surface: f32 = textureLoad(traced_depth, texel, 0);

  if (surface <= 0.0) {
    return 0.0;
  }

  let met: vec3<f32> = ray_view_point(hit);
  let direction: vec3<f32> = normalize(met - origin);
  let facing: f32 = dot(depth_normal(texel, met), direction);
  let thick: f32 = 1.0 - smoothstep(0.0, reach, walk.behind);
  let edges: vec2<f32> = smoothstep(vec2<f32>(0.0), vec2<f32>(EDGE_FADE * size.y / size.x, EDGE_FADE), hit.xy) *
    (1.0 - smoothstep(1.0 - vec2<f32>(EDGE_FADE * size.y / size.x, EDGE_FADE), vec2<f32>(1.0), hit.xy));
  let far: f32 = 1.0 - smoothstep(reflection.distance * 0.75, reflection.distance, length(met - origin));

  return thick * thick * (1.0 - smoothstep(0.0, BACK_FACING, facing)) * edges.x * edges.y * far;
}

// The normal of a traced pixel's whole block, averaged: a puddle's rain rings are finer than the trace's blocks, and
// one texel's normal aliases them where the block's mean follows the water.
fn block_normal(pixel: vec2<f32>) -> vec3<f32> {
  let last: vec2<i32> = vec2<i32>(camera.viewport.xy) - 1;
  let first: vec2<i32> = min(vec2<i32>(pixel * reflection.ratio), last);
  let side: i32 = i32(reflection.ratio);
  var sum: vec3<f32> = vec3<f32>(0.0);

  for (var y: i32 = 0; y < side; y++) {
    for (var x: i32 = 0; x < side; x++) {
      sum += octahedral_decode(textureLoad(normal_target, min(first + vec2<i32>(x, y), last), 0).xy);
    }
  }

  return normalize(sum);
}

// What the trace writes: what is reflected, then the ray's length; and how the ray ended.
struct TraceTargets {
  @location(0) traced: vec4<f32>,
  @location(1) end: f32,
};

fn trace_targets(traced: vec4<f32>, end: u32) -> TraceTargets {
  return TraceTargets(traced, reflection_end_value(end));
}

@fragment
fn fs_trace(in: FullscreenVarying) -> TraceTargets {
  let pixel: vec2<f32> = floor(in.clip.xy);
  let texel: vec2<i32> = traced_texel(pixel);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    return trace_targets(UNTRACED, END_UNTRACED);
  }

  let material: vec4<f32> = textureLoad(material_target, texel, 0);

  // Grass and leaves reflect the environment as `hmodel` does, and are never traced.
  if (has_mark(material.a, MARK_PLANT)) {
    return trace_targets(UNTRACED, END_UNTRACED);
  }

  let albedo: vec4<f32> = textureLoad(albedo_target, texel, 0);
  let wet: vec4<f32> = textureLoad(wet_surface, texel, 0);
  let roughness: f32 = traced_roughness(albedo.a, wet.a + surface_variance(normal_target, depth_target, texel, wet.g));

  // Past the limit the surface reflects the prefiltered sky alone, which combine reads for it.
  if (roughness > TRACED_ROUGHNESS_LIMIT) {
    return trace_targets(UNTRACED, END_ROUGH);
  }

  var normal: vec3<f32>;

  if (wet.g > PUDDLE_LEAST && reflection.ratio > 1.0) {
    normal = block_normal(pixel);
  } else {
    normal = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  }

  // A puddle's ray leaves its water, which stands over the ground the depth shows by the water's height there.
  let position: vec3<f32> = camera_view_position(vec2<f32>(texel) + 0.5, depth) + normal * wet.b;
  let toward: vec3<f32> = normalize(position);
  let facing: f32 = dot(normal, -toward);
  let brdf: vec3<f32> = environment_brdf(reflectance_head_on(albedo.rgb, albedo.a, wet.r, material.z), roughness, facing) *
    reflection.intensity;

  // Traced only where combine would show enough of it.
  if (max(max(brdf.r, brdf.g), brdf.b) <= TRACE_FLOOR) {
    return trace_targets(UNTRACED, END_UNTRACED);
  }

  let direction: vec3<f32> = reflected_direction(toward, normal, lobe_alpha(roughness), pixel_numbers(pixel));
  // What a ray meeting nothing reflects: the sky prefiltered part way to the surface's roughness along the ray's own
  // direction, occluded only as the screen's occlusion sees the surface's surroundings: the ray itself went clear to
  // the screen's edge or its distance, and the baked hemisphere, an ambient term over half the sky on an open road,
  // is the rough fallback's in combine.
  let searched: vec2<i32> = min(texel / 2, vec2<i32>(textureDimensions(occlusion_target)) - 1);
  let visible: f32 = select(1.0, textureLoad(occlusion_target, searched, 0).x, lighting.params.w > 0.5);
  let aimed: vec3<f32> = surroundings_direction(sky_prefiltered, sky_clamp, world_offset(position), world_direction(direction));
  var sky: vec3<f32> = prefiltered_sky(sky_prefiltered, sky_clamp, aimed, roughness * MISS_BLUR) *
    specular_occlusion(facing, visible, roughness);

  // The ray's end, the distance traced out, or short of the near plane where it comes back towards the camera: its
  // line across the screen is as long as its reach, so a reach as short as the surface is near cut a puddle's
  // reflection off at a line that moved with the camera.
  let near: f32 = camera.projection[3][2] / (1.0 + camera.projection[2][2]);
  let reach: f32 = select(reflection.distance, min(reflection.distance, (-position.z - near * 1.05) / direction.z),
    direction.z > 0.0);
  let origin: vec3<f32> = ray_screen_point(position);
  let toward_screen: vec3<f32> = ray_screen_point(position + direction * max(reach, 1e-3)) - origin;
  // A rough surface's reflection is blurred over its lobe: half the crossings find what it shows.
  let budget: u32 = u32(f32(reflection.crossings) * mix(1.0, ROUGH_CROSSINGS, smoothstep(ROUGH_BUDGET.x, ROUGH_BUDGET.y,
    roughness)));
  let walk: RayWalk = ray_walk(nearest_depth, origin, toward_screen, max(budget, 8u), HIT_THICKNESS);
  let distance_met: f32 = -ray_view_point(walk.position).z;
  let thickness: f32 = HIT_THICKNESS.x + distance_met * HIT_THICKNESS.y;
  let confidence: f32 = select(0.0, hit_confidence(walk, position, thickness), walk.is_met);
  var end: u32 = walk.end;
  var missed_length: f32 = MISSED_LENGTH;

  // Where the screen's answer is less than sure, the level's geometry answers in the sky's place.
  if (confidence < 1.0) {
    let traced: vec4<f32> = traced_radiance(position, normal, direction, roughness);

    if (traced.w >= 0.0) {
      sky = traced.rgb;
      missed_length = traced.w;
      end = END_TRACED;
    }
  }

  // A ray that ran its reach over the screen's own sky, or ground so far the fog has it, reflects the sky as the
  // screen shows it, exact in colour and fog, in place of the cube's, whose texel is coarser and whose sky is the
  // frame's sky at best.
  if (end == END_FAR && all(walk.position.xy >= vec2<f32>(0.0)) && all(walk.position.xy <= vec2<f32>(1.0))) {
    let size: vec2<i32> = vec2<i32>(camera.viewport.xy);
    let end_texel: vec2<i32> = clamp(vec2<i32>(walk.position.xy * camera.viewport.xy), vec2<i32>(0), size - 1);
    let end_depth: f32 = textureLoad(depth_target, end_texel, 0);

    if (end_depth <= 0.0 || path_fog(camera.projection[3][2] / (end_depth + camera.projection[2][2])) >= SOURCE_FOG_LIMIT) {
      let seen: vec4<f32> = remembered_sky(world_direction(direction));

      if (seen.w > 0.5) {
        sky = seen.rgb * specular_occlusion(facing, visible, roughness);
        end = END_SKY;
      }
    }
  }

  // Uncertain a little behind a surface: that surface's colour is the probable answer, the sky's the further behind.
  if (walk.end == END_UNCERTAIN) {
    let behind_texel: vec2<i32> = vec2<i32>(walk.position.xy * camera.viewport.xy);
    let is_plant: bool = has_mark(textureLoad(material_target, behind_texel, 0).a, MARK_PLANT);
    let band: vec2<f32> = select(PROBABLE_BEHIND, PROBABLE_BEHIND_PLANT, is_plant);
    let probable: f32 = 1.0 - smoothstep(thickness, thickness + band.x + distance_met * band.y, walk.behind);

    if (probable > 0.0) {
      let shown: vec3<f32> = shown_point(behind_texel);
      let taken: vec4<f32> = met_radiance(behind_texel, shown, position);

      return trace_targets(
        vec4<f32>(mix(sky, taken.rgb, probable), select(missed_length, length(shown - position), probable > 0.5)),
        select(END_UNCERTAIN, END_PROBABLE, probable > 0.5) | select(END_RELIT, 0u, taken.w > 0.5),
      );
    }
  }

  if (confidence <= 0.0) {
    return trace_targets(vec4<f32>(sky, missed_length), select(end, END_UNTRUSTED, walk.is_met && end != END_TRACED));
  }

  let hit: vec3<f32> = ray_view_point(walk.position);
  let met: vec4<f32> = met_radiance(vec2<i32>(walk.position.xy * camera.viewport.xy), hit, position);
  let length_met: f32 = length(hit - position);

  return trace_targets(
    vec4<f32>(mix(sky, met.rgb, confidence), select(missed_length, length_met, confidence > 0.5)),
    walk.end | select(END_RELIT, 0u, met.w > 0.5),
  );
}

// A traced pixel's surface: its world normal, roughness and distance along the view, as the history keeps them.
struct TracedSurface {
  normal: vec3<f32>,
  roughness: f32,
  distance: f32,
  world: vec3<f32>,
};

// The surface at the frame's texel a traced pixel stands for, its roughness with it, which the guide keeps.
fn traced_surface_at(texel: vec2<i32>) -> TracedSurface {
  var surface: TracedSurface = traced_geometry(texel);
  let wet: vec4<f32> = textureLoad(wet_surface, texel, 0);

  surface.roughness = traced_roughness(textureLoad(albedo_target, texel, 0).a,
    wet.a + surface_variance(normal_target, depth_target, texel, wet.g));

  return surface;
}

// The surface at the frame's texel a traced pixel stands for, its roughness left at none: its normal, distance and
// place alone, all a prefilter tap is weighed by, so the fifteen taps a pixel read nothing more.
fn traced_geometry(texel: vec2<i32>) -> TracedSurface {
  let depth: f32 = max(textureLoad(depth_target, texel, 0), 1e-7);
  let normal: vec3<f32> = world_direction(octahedral_decode(textureLoad(normal_target, texel, 0).xy));
  let is_level: bool = has_mark(textureLoad(material_target, texel, 0).a, MARK_TERRAIN) &&
    (normal.y > LEVEL_UP || textureLoad(wet_surface, texel, 0).g > PUDDLE_LEAST);
  let uv: vec2<f32> = (vec2<f32>(texel) + 0.5) / camera.viewport.xy;
  let world: vec3<f32> = camera_unproject(vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0), depth);

  return TracedSurface(select(normal, vec3<f32>(0.0, 1.0, 0.0), is_level), 0.0,
    -camera_view_position(vec2<f32>(texel) + 0.5, depth).z, world);
}

// The first frame texel of a traced pixel's block, which the guide's texel is counted from.
fn guide_block(pixel: vec2<i32>) -> vec2<i32> {
  return min(vec2<i32>(vec2<f32>(pixel) * reflection.ratio), vec2<i32>(camera.viewport.xy) - 1);
}

// The frame texel a traced pixel stands for, from the guide's `w`: its place in the pixel's block.
fn guide_texel(pixel: vec2<i32>, packed: f32) -> vec2<f32> {
  let place: u32 = u32(packed);

  return vec2<f32>(guide_block(pixel) + vec2<i32>(i32(place & 15u), i32(place >> 4u)));
}

// A world normal as the guide's `x` holds it, twelve bits an octahedral axis.
fn guide_normal(packed: f32) -> vec3<f32> {
  let bits: u32 = u32(packed);

  return octahedral_decode(vec2<f32>(vec2<u32>(bits >> 12u, bits & 4095u)) / 4095.0 * 2.0 - 1.0);
}

// The guide a traced pixel's surface is read from by the stages after the trace, worked out once a frame: its world
// normal, terrain's levelled, in `x`; its roughness; its depth; the frame texel it stands for, as its place in the
// pixel's block, in `w`. The packed values are whole numbers a float holds exactly, never bit patterns a target's write
// may bend into a NaN or flush as a denormal. What every stage and tap would otherwise find again from the G-buffer,
// its block's search included.
@fragment
fn fs_guide(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let pixel: vec2<i32> = vec2<i32>(floor(in.clip.xy));
  let texel: vec2<i32> = traced_texel(vec2<f32>(pixel));
  let surface: TracedSurface = traced_surface_at(texel);
  let place: vec2<u32> = vec2<u32>(texel - guide_block(pixel));
  let normal: vec2<u32> = vec2<u32>(round((octahedral_encode(surface.normal) * 0.5 + 0.5) * 4095.0));

  return vec4<f32>(f32(normal.x * 4096u + normal.y), surface.roughness, textureLoad(depth_target, texel, 0),
    f32(place.x + place.y * 16u));
}

// The surface a traced pixel stands for, read from this frame's guide.
fn guided_surface(pixel: vec2<f32>) -> TracedSurface {
  let last: vec2<i32> = vec2<i32>(textureDimensions(guide)) - 1;
  let at: vec2<i32> = clamp(vec2<i32>(pixel), vec2<i32>(0), last);
  let read: vec4<f32> = textureLoad(guide, at, 0);
  let texel: vec2<f32> = guide_texel(at, read.w) + 0.5;
  let depth: f32 = max(read.z, 1e-7);
  let uv: vec2<f32> = texel / camera.viewport.xy;
  let world: vec3<f32> = camera_unproject(vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0), depth);

  return TracedSurface(guide_normal(read.x), read.y,
    -camera_view_position(texel, depth).z, world);
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
// it keeps near the neighbourhood; searched about where the history disagrees, dropped where it still does, and for a
// mirror. Holds more frames the rougher the surface.
@fragment
fn fs_reproject(in: FullscreenVarying) -> ReprojectedTargets {
  let pixel: vec2<f32> = floor(in.clip.xy);
  let fresh: vec4<f32> = textureLoad(traced, vec2<i32>(pixel), 0);
  if (fresh.a < 0.0) {
    return ReprojectedTargets(vec4<f32>(0.0), 0.0);
  }

  let surface: TracedSurface = guided_surface(pixel);
  // Without history a rough reflection is all noise, a mirror's little.
  let dropped: ReprojectedTargets = ReprojectedTargets(vec4<f32>(0.0, 0.0, 0.0, 1.0),
    mix(0.1, 1.0, smoothstep(BLURRED_ROUGHNESS.x, BLURRED_ROUGHNESS.y, surface.roughness)));

  // A mirror's ray is the same every frame, so its history would only lag behind it.
  if (reflection.has_history < 0.5 || surface.roughness < MIRROR_ROUGHNESS) {
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

  // A rough reflection is noise, held long; a near-mirror's a few frames.
  let most: f32 = mix(MIRROR_FRAMES, max(8.0, MOST_SAMPLES * (1.0 - exp(-surface.roughness * 100.0))),
    smoothstep(BLURRED_ROUGHNESS.x, BLURRED_ROUGHNESS.y, surface.roughness));
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
  let surface: TracedSurface = guided_surface(pixel);

  if (spread <= 0.0 || surface.roughness < 1e-4 || surface.roughness > TRACED_ROUGHNESS_LIMIT) {
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

  // At the frame's own size its pixels lie four times as dense as at half: the first eight taps span as much.
  let taps: u32 = select(15u, 8u, reflection.ratio <= 1.0);

  for (var tap: u32 = 0u; tap < taps; tap++) {
    let at: vec2<i32> = clamp(vec2<i32>(pixel) + PREFILTER_TAPS[tap], vec2<i32>(0), last);
    let neighbour: vec4<f32> = textureLoad(traced, at, 0);

    if (neighbour.a < 0.0) {
      continue;
    }

    let other: TracedSurface = guided_surface(vec2<f32>(at));
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

  let surface: TracedSurface = guided_surface(pixel);
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

// The resolve brought up to a frame pixel by the guide: the four traced pixels about it by their bilinear share, how
// near the distance each one's surface stands to the pixel's, how alike its world normal (terrain's levelled, as the
// guide keeps it) faces to `normal`, and how near its roughness stands to `roughness`, as `upsampled_reflection` weighs
// them from the G-buffer, read from the guide in one load each. Untraced, alpha below none, where none speaks for it.
fn guided_upsample(pixel: vec2<f32>, normal: vec3<f32>, distance: f32, roughness: f32) -> vec4<f32> {
  let ratio: f32 = reflection.ratio;
  let last: vec2<f32> = vec2<f32>(textureDimensions(resolved)) - 1.0;
  let base: vec2<f32> = floor(pixel / ratio);
  let fraction: vec2<f32> = (pixel - base * ratio) / ratio;
  var sum: vec4<f32> = vec4<f32>(0.0);
  var weights: f32 = 0.0;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let offset: vec2<f32> = vec2<f32>(f32(corner & 1u), f32(corner >> 1u));
    let at: vec2<i32> = vec2<i32>(clamp(base + offset, vec2<f32>(0.0), last));
    let texel: vec4<f32> = textureLoad(resolved, at, 0);
    let read: vec4<f32> = textureLoad(guide, at, 0);
    let shown: vec2<f32> = guide_texel(at, read.w) + 0.5;
    let there: f32 = -camera_view_position(shown, max(read.z, 1e-7)).z;
    let facing: f32 = pow(saturate(dot(guide_normal(read.x), normal)),
      UPSAMPLE_NORMAL_POWER);
    let alike: f32 = 1.0 / (1.0 + UPSAMPLE_ROUGHNESS_SCALE * abs(read.y - roughness));
    let bilinear: f32 = mix(1.0 - fraction.x, fraction.x, offset.x) * mix(1.0 - fraction.y, fraction.y, offset.y);
    let difference: f32 = abs(there - distance) / max(distance, 1e-3);
    let weight: f32 = select(0.0, bilinear * facing * alike / (difference * difference + 1e-3),
      texel.a >= 0.0 && read.z > 0.0);

    sum += texel * weight;
    weights += weight;
  }

  if (weights <= 1e-6) {
    return UNTRACED;
  }

  return sum / weights;
}

// The full-size history's share of this frame's reflection a frame where it holds, and how far, a share of the
// distance, the point it held may stand from where this frame shows it before it is dropped: four frames bring each
// cell of a block its own ray, so the history keeps about as many.
const UPSCALE_FRESH: f32 = 0.25;
const UPSCALE_MOVED: f32 = 0.1;

// The reflection brought up to the frame's size and held there over time: this frame's resolve upsampled by depth,
// normal and roughness, the cells it was traced from turning every frame, blended into the last frame's full-size
// history reprojected by the surface's motion and clipped to the resolve's neighbourhood, so the history rebuilds
// what the half-size trace cannot hold in one frame, with or without the frame's own antialiasing. What is reflected,
// then the pixel's distance along the view, which the next frame's reprojection checks; below none where untraced.
@fragment
fn fs_upscale(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let pixel: vec2<f32> = floor(in.clip.xy);
  let texel: vec2<i32> = vec2<i32>(pixel);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    return UNTRACED;
  }

  let normal: vec3<f32> = normalize(octahedral_decode(textureLoad(normal_target, texel, 0).xy));
  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let distance: f32 = -position.z;
  let wet: vec4<f32> = textureLoad(wet_surface, texel, 0);
  let roughness: f32 = traced_roughness(textureLoad(albedo_target, texel, 0).a,
    wet.a + surface_variance(normal_target, depth_target, texel, wet.g));
  // At the frame's own size the resolve already holds the pixel: read, not upsampled.
  var current: vec4<f32>;

  if (reflection.ratio <= 1.0) {
    current = textureLoad(resolved, min(texel, vec2<i32>(textureDimensions(resolved)) - 1), 0);
  } else {
    current = guided_upsample(pixel, traced_geometry(texel).normal, distance, roughness);
  }

  if (current.a < 0.0) {
    return UNTRACED;
  }

  if (reflection.has_history < 0.5) {
    return vec4<f32>(current.rgb, distance);
  }

  let size: vec2<f32> = camera.viewport.xy;
  let ndc: vec2<f32> = vec2<f32>(in.clip.x / size.x * 2.0 - 1.0, 1.0 - in.clip.y / size.y * 2.0);
  let clip: vec4<f32> = camera.motion_previous * vec4<f32>(camera_unproject(ndc, depth), 1.0);
  let uv: vec2<f32> = (clip.xy / clip.w) * vec2<f32>(0.5, -0.5) + 0.5;

  if (clip.w <= 0.0 || any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
    return vec4<f32>(current.rgb, distance);
  }

  let held: vec4<f32> = textureSampleLevel(history_full, history_sampler, uv, 0.0);

  if (held.a < 0.0 || abs(held.a - clip.w) > clip.w * UPSCALE_MOVED) {
    return vec4<f32>(current.rgb, distance);
  }

  // The resolve's three by three about the pixel's block, which the history is clipped to.
  let last: vec2<i32> = vec2<i32>(textureDimensions(resolved)) - 1;
  let block: vec2<i32> = vec2<i32>(pixel / reflection.ratio);
  var least: vec3<f32> = current.rgb;
  var most: vec3<f32> = current.rgb;

  for (var y: i32 = -1; y <= 1; y++) {
    for (var x: i32 = -1; x <= 1; x++) {
      let around: vec4<f32> = textureLoad(resolved, clamp(block + vec2<i32>(x, y), vec2<i32>(0), last), 0);

      if (around.a >= 0.0) {
        least = min(least, around.rgb);
        most = max(most, around.rgb);
      }
    }
  }

  let kept: vec3<f32> = clip_box(least, most, held.rgb);
  let blended: vec3<f32> = mix(kept, current.rgb, UPSCALE_FRESH);

  return vec4<f32>(select(current.rgb, blended, all(blended == blended)), distance);
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
