#import "common/depth_plane"
#import "common/camera"
#import "common/scene_light"
#import "common/octahedral"
#import "common/fullscreen"
#import "common/lighting"
#import "common/hmodel"
#import "common/occlusion"
#import "common/reflection_trace"
#import "common/reflection_end"
#import "common/trace_requests"
#import "common/noise"

// Stochastic screen-space reflections on the frame's surfaces, at the size the quality traces at, a pixel per `ratio`
// by `ratio` of the frame. Each reflecting pixel's ray leaves in a direction drawn from its surface's GGX lobe (a
// puddle's water's, from the water's own height over the ground) by blue noise that turns every frame, and walks the
// nearest depth pyramid of the frame as it stood before the grass drew (`nearest_depth`, its first level that depth
// itself): a bush's or a tuft's billboards stand in for a volume a reflection sees through, so they never stop a ray,
// and what lies behind them answers; the trees' leaves are the static scene's and stand (`common/reflection_trace`);
// what it met is lit as the frame shows it, faded towards the screen's edges and the distance traced, the sky or the
// environment `hmodel` reflects otherwise. Then the denoiser: the last frame's reflection reprojected, from where the
// reflected point or the surface stood; an eighth-size average; the trace filtered across the surface, stopping at
// edges; and the two resolved over time, the history held to the neighbourhood, into this frame's history, which
// combine fades into the prefiltered sky by the surface's roughness.
//
// Each stage marks a pixel nothing is traced at with an alpha below none.

// The G-buffer, its light and occlusion, the material table, the lighting and the settings, the nearest depth pyramid
// and blue noise, the stages before, and the last frame's reflection, surface and held distance.
#import "generated/frame/reflections"

// The sky's cubes, which a ray meeting nothing reflects.
#import "generated/common/sky"

// Where the trace writes under ray budgets, as a compute pass.
#import "generated/frame/reflection_trace_out"

// The roughness under which a surface is a mirror, whose ray is the same every frame and holds no history: under a
// puddle's water at rest, which holds a few.
const MIRROR_ROUGHNESS: f32 = 0.01;
// The frames a near-mirror's reflection holds: its ray hardly moves, and its hit point reprojects, so a few steady it
// without lagging a moving camera.
const MIRROR_FRAMES: f32 = 4.0;
// The share of the roughness's mip a missed ray reads the prefiltered sky at: its lobe spreads the rays already, and
// the denoiser gathers them, so the cube supplies the other half of the blur.
const MISS_BLUR: f32 = 0.5;
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
const UNTRACED: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, -1.0);
// The roughness over which a ray's crossings fall to `ROUGH_CROSSINGS` of the quality's, its blurred reflection needing
// no finer walk.
const ROUGH_BUDGET: vec2<f32> = vec2<f32>(0.15, 0.45);
const ROUGH_CROSSINGS: f32 = 0.5;
// The share of what it reflects a pixel's specular shows under which its reflection is dim, and the frame's pixels its
// surface may have moved since the last frame for it to be still: a still, dim reflection's scene trace takes one turn
// in `DIM_TURNS` of the quality's, what it reflected held between, as the held history stays true only where the view
// stands; a moving one traces at the quality's turns.
const DIM_SHARE: f32 = 0.15;
const DIM_STILL: f32 = 2.0;
const DIM_TURNS: u32 = 4u;

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
// How far a surface held from the frame before may stand off this one, a share of its distance, and how near its
// normal must face, for its reflection to be held off the pixel's turn to trace.
const HELD_DISTANCE: f32 = 0.1;
const HELD_NORMAL: f32 = 0.8;
// How far, a share of the length, what the frame before reflected at a pixel's reflected point may stand off the
// length the pixel holds for it to be the same reflection.
const HELD_LENGTH: f32 = 0.15;
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

// Whether a pixel the screen leaves unanswered traces against the level's geometry this frame: once in its
// `trace_turns`, `scale` times as many for a still, dim reflection, stretched by the list's wait where the rays asked
// overrun it; each pixel's turn its own (`trace_turn_of`), so a held pixel has traced ones about it, which the resolve
// clips it to. Tiles turning together kept a held tile's lag from its traced neighbours as a chequer in motion.
fn is_trace_turn(pixel: vec2<f32>, scale: u32) -> bool {
  let turns: u32 = max(reflection.trace_turns, 1u) * scale * trace_request_wait();

  return (trace_turn_of(vec2<u32>(pixel), turns) + reflection.frame) % turns == 0u;
}

// What a pixel reflected the frame before, off its turn to trace: read where its reflected point stood then, through
// the surface by the length it held, where the surface there is this one, its distance within `HELD_DISTANCE` of it
// and its normal within `HELD_NORMAL`, as the guide describes surfaces; in `w` that length, below none where the frame
// before held nothing here, which traces instead. Looser than the reprojection's match: the traced texel of a block
// moves from one frame to the next, so the history at a surface is a neighbour's. The reflected point holds only where
// the frame before showed something as far through the surface there (`held_reflection`): a reflection the view moved
// off the pixel would stay behind as a ghost of it.
fn held_radiance(texel: vec2<i32>) -> vec4<f32> {
  let none: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, -1.0);

  if (reflection.is_traced < 0.5 || reflection.has_history < 0.5) {
    return none;
  }

  let surface: TracedSurface = traced_geometry(texel);
  let surface_uv: vec2<f32> = previous_uv(surface.world);

  if (any(surface_uv <= vec2<f32>(0.0)) || any(surface_uv >= vec2<f32>(1.0))) {
    return none;
  }

  let held: vec4<f32> = held_at(surface_uv);
  let length_held: f32 = held.z - held.x;
  let is_same: bool = abs(held.x - surface.distance) <= surface.distance * HELD_DISTANCE &&
    dot(normalize(history_surface_at(surface_uv).xyz + 1e-6), surface.normal) >= HELD_NORMAL;

  if (!is_same || length_held <= 0.0) {
    return none;
  }

  let toward: vec3<f32> = surface.world - camera.position.xyz;
  let first: vec4<f32> = held_reflection(toward, length_held);

  // Off the screen the frame before, the surface's own reflection holds.
  if (first.w == 0.0) {
    return vec4<f32>(history_radiance_at(surface_uv).rgb, length_held);
  }

  if (first.w > 0.0) {
    return first;
  }

  // What the frame before showed there instead, the sky about a reflection the view moved off, holds where it agrees.
  let second: vec4<f32> = held_reflection(toward, -first.w);

  return select(none, second, second.w > 0.0);
}

// What the frame before showed through the surface `toward` the pixel `reach` on: its radiance there and the reach,
// where what it reflected there stood within `HELD_LENGTH` of as far; else, below none, the length it held there, or
// none where that point stood off the screen.
fn held_reflection(toward: vec3<f32>, reach: f32) -> vec4<f32> {
  let hit_uv: vec2<f32> = previous_uv(camera.position.xyz + normalize(toward) * (length(toward) + reach));

  if (any(hit_uv <= vec2<f32>(0.0)) || any(hit_uv >= vec2<f32>(1.0))) {
    return vec4<f32>(0.0);
  }

  let there: vec4<f32> = held_at(hit_uv);
  let length_there: f32 = there.z - there.x;

  if (abs(length_there - reach) > reach * HELD_LENGTH) {
    return vec4<f32>(0.0, 0.0, 0.0, -max(length_there, 1e-3));
  }

  return vec4<f32>(history_radiance_at(hit_uv).rgb, reach);
}

// The ray a pixel leaves to the scene pass: it starts off the surface as far as the depth's own offset needs, reaches
// the reflections' distance, and widens by the traced pixel's angle and the surface's lobe.
fn scene_ray(pixel: vec2<f32>, position: vec3<f32>, normal: vec3<f32>, direction: vec3<f32>, roughness: f32)
  -> TraceRequest {
  let distance: f32 = -position.z;
  let origin: vec3<f32> = camera.position.xyz + world_offset(position) +
    world_direction(normal) * (TRACED_BIAS.x + distance * TRACED_BIAS.y);
  let pixel_angle: f32 = 2.0 * reflection.ratio / (camera.projection[1][1] * camera.viewport.y);
  var request: TraceRequest;

  request.origin = vec4<f32>(origin, reflection.distance);
  request.cone = vec2<f32>(distance * pixel_angle, pixel_angle + roughness);
  request.toward = pack2x16snorm(octahedral_encode(world_direction(direction)));
  request.pixel = u32(pixel.x) | (u32(pixel.y) << 16u);

  return request;
}

// The ray this invocation's pixel leaves to the scene pass, where it leaves one, and the frame's texel it stands for.
var<private> scene_request: TraceRequest;
var<private> is_scene_requested: bool = false;
var<private> scene_texel: vec2<i32>;
// The corners of the pixel's quad that take its answer under ray budgets, which its request carries to the scene pass.
var<private> scene_mates: u32 = 0u;

// Lists the pixel's ray for the scene pass with what the screen answered (`answer`'s colour weighed by its `share`,
// its length) and the reason `hit_end` its ends take where the ray meets something; whether the list took it.
fn request_scene(answer: vec4<f32>, share: f32, hit_end: u32) -> bool {
  var request: TraceRequest = scene_request;

  request.screen = trace_request_screen(answer.rgb, share);
  request.screen_length = answer.w;
  request.hit_end = hit_end | (scene_mates << TRACE_REQUEST_MATES_SHIFT);

  return list_trace_request(request);
}

// What the trace writes for a pixel: what the screen answered (`answer`, its colour, and its length where its `share`
// holds most) over `fallback`, the sky and the length a missed ray takes, and the pixel's reason; and, where the pixel
// leaves its ray to the scene pass, its request, the reason its ends take on a hit `hit_end`; where the list has no
// room for it, what the pixel reflected the frame before in the sky's place, where that holds.
fn answer_trace(fallback: vec4<f32>, answer: vec4<f32>, share: f32, end: u32, hit_end: u32) -> TraceTargets {
  var behind: vec4<f32> = fallback;
  var reason: u32 = end;

  if (is_scene_requested && !request_scene(answer, share, hit_end)) {
    let held: vec4<f32> = held_radiance(scene_texel);

    if (held.w >= 0.0) {
      behind = held;
      reason = select(end, END_HELD, share <= 0.5);
    }
  }

  return trace_targets(vec4<f32>(mix(behind.rgb, answer.rgb, share), select(behind.w, answer.w, share > 0.5)), reason);
}

// Two uniform numbers for a pixel this frame: the blue noise there and half its tile away, turned by the frame.
fn pixel_numbers(pixel: vec2<f32>) -> vec2<f32> {
  let size: vec2<u32> = textureDimensions(blue_noise);
  let at: vec2<u32> = vec2<u32>(pixel) % size;
  let noise: vec2<f32> = vec2<f32>(
    textureLoad(blue_noise, at, 0).r,
    textureLoad(blue_noise, (at + size / 2u) % size, 0).r,
  );

  return fract(noise + f32(reflection.frame) * R2_STEPS);
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
  let depth: f32 = textureLoad(nearest_depth, texel, 0).x;
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
  let shown: vec3<f32> = scene_light(textureSampleLevel(source_scene, history_sampler, kept_uv, 0.0).rgb, scale);
  return vec4<f32>(source_unfogged(shown, fog, fog_color_at(lighting, hit, world.y)), 1.0);
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

  if (kept > 0.0 && path_fog(lighting, camera.projection[3][2] / (kept + camera.projection[2][2])) < SOURCE_FOG_LIMIT) {
    return vec4<f32>(0.0);
  }

  let scale: f32 = select(1.0, exposure.used, lighting.params.z > 0.5);

  return vec4<f32>(sky_as_light(scene_shown(textureSampleLevel(source_scene, history_sampler, kept_uv, 0.0).rgb), scale,
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

  let fogged: vec3<f32> = mix(lit, lighting.fog_color.rgb, path_fog(lighting, length(hit - origin)));

  return vec4<f32>(fogged * min(1.0, RADIANCE_LIMIT / traced_luminance(fogged)), remembered.w);
}

// The frame texel a walk's position on the screen stands at, held to the frame: one at the closed edge, 1, is the last
// texel.
fn walk_texel(position: vec2<f32>) -> vec2<i32> {
  return clamp(vec2<i32>(position * camera.viewport.xy), vec2<i32>(0), vec2<i32>(camera.viewport.xy) - 1);
}

// The view space point the walked depth shows at a frame texel, held to the frame, one in `w` where something is
// drawn there.
fn shown_point(texel: vec2<i32>) -> vec4<f32> {
  let held: vec2<i32> = clamp(texel, vec2<i32>(0), vec2<i32>(camera.viewport.xy) - 1);
  let depth: f32 = textureLoad(nearest_depth, held, 0).x;

  return vec4<f32>(camera_view_position(vec2<f32>(held) + 0.5, max(depth, 1e-7)), select(0.0, 1.0, depth > 0.0));
}

// The view space normal of the surface the depth shows at a texel, from the points about it, the nearer neighbour
// each way so an edge does not lean it; towards the camera. The G-buffer's normal is bumped by tens of degrees, which
// would turn a facing test into speckle.
fn depth_normal(texel: vec2<i32>, point: vec3<f32>) -> vec3<f32> {
  return surface_normal(
    point,
    shown_point(texel - vec2<i32>(1, 0)),
    shown_point(texel + vec2<i32>(1, 0)),
    shown_point(texel - vec2<i32>(0, 1)),
    shown_point(texel + vec2<i32>(0, 1)),
  );
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

  let texel: vec2<i32> = walk_texel(hit.xy);
  let surface: f32 = textureLoad(nearest_depth, texel, 0).x;

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

// A traced pixel's surface as its ray leaves it: the frame texel it stands for, its roughness, normal (a puddle's
// block's), view space point (a puddle's water's) and the view's direction to it, how squarely it faces the view, and
// the share of what it reflects its specular shows; or, where nothing is traced, the reason.
struct TraceStart {
  is_traced: bool,
  end: u32,
  texel: vec2<i32>,
  roughness: f32,
  normal: vec3<f32>,
  position: vec3<f32>,
  toward: vec3<f32>,
  facing: f32,
  shown: f32,
  is_puddle: bool,
};

// A pixel nothing is traced at, for the reason `end`.
fn untraced_start(end: u32) -> TraceStart {
  var start: TraceStart;

  start.is_traced = false;
  start.end = end;

  return start;
}

// The surface a traced pixel's ray leaves, or why nothing is traced there: the sky, a plant, a surface past the
// roughness limit, or one combine would show too little of.
fn trace_start(pixel: vec2<f32>) -> TraceStart {
  let texel: vec2<i32> = traced_texel(pixel);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    return untraced_start(END_UNTRACED);
  }

  let material: vec4<f32> = textureLoad(material_target, texel, 0);

  // Grass and leaves reflect the environment as `hmodel` does, and are never traced.
  if (has_mark(material.a, MARK_PLANT)) {
    return untraced_start(END_UNTRACED);
  }

  let albedo: vec4<f32> = textureLoad(albedo_target, texel, 0);
  let wet: vec4<f32> = load_frame_target(wet_surface, texel);
  let roughness: f32 = traced_roughness(albedo.a, wet.a + surface_variance(normal_target, depth_target, texel, wet.g));

  // Past the limit the surface reflects the prefiltered sky alone, which combine reads for it.
  if (roughness > lighting.reflection_cut.y) {
    return untraced_start(END_ROUGH);
  }

  let is_puddle: bool = wet.g > PUDDLE_LEAST;
  var normal: vec3<f32>;

  if (is_puddle && reflection.ratio > 1.0) {
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

  let shown: f32 = max(max(brdf.r, brdf.g), brdf.b);

  // Traced only where combine would show enough of it (`ReflectionUniform::trace_floor`): on a dry level, by default,
  // only what reflects more than a dielectric's 4%.
  if (shown <= reflection.trace_floor) {
    return untraced_start(END_UNTRACED);
  }

  return TraceStart(true, END_UNTRACED, texel, roughness, normal, position, toward, facing, shown, is_puddle);
}

@fragment
fn fs_trace(in: FullscreenVarying) -> TraceTargets {
  return trace_pixel(floor(in.clip.xy));
}

// What a traced pixel reflects: its ray walked over the nearest depth's pyramid, what it met lit, the sky where it met
// nothing, the level's geometry asked to answer where the screen is less than sure.
fn trace_pixel(pixel: vec2<f32>) -> TraceTargets {
  let start: TraceStart = trace_start(pixel);

  is_scene_requested = false;

  if (!start.is_traced) {
    return trace_targets(UNTRACED, start.end);
  }

  let texel: vec2<i32> = start.texel;
  let roughness: f32 = start.roughness;
  let normal: vec3<f32> = start.normal;
  let position: vec3<f32> = start.position;
  let toward: vec3<f32> = start.toward;
  let facing: f32 = start.facing;
  let shown: f32 = start.shown;
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

  // Where the screen's answer is less than sure, the level's geometry answers in the sky's place: what the pixel
  // reflected the frame before, off its turn, or the scene pass's trace, on its turn or where that cannot be held.
  if (confidence < 1.0 && reflection.is_traced > 0.5) {
    var held: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, -1.0);

    let was: vec2<f32> = previous_uv(camera.position.xyz + world_offset(position)) * camera.viewport.xy;
    let is_still_dim: bool = shown < DIM_SHARE && length(was - (vec2<f32>(texel) + 0.5)) < DIM_STILL;

    if (!is_trace_turn(pixel, select(1u, DIM_TURNS, is_still_dim))) {
      held = held_radiance(texel);
    }

    if (held.w >= 0.0) {
      sky = held.rgb;
      missed_length = held.w;
      end = END_HELD;
    } else {
      scene_request = scene_ray(pixel, position, normal, direction, roughness);
      is_scene_requested = true;
      scene_texel = texel;
    }
  }

  // A ray that ran its reach over the screen's own sky, or ground so far the fog has it, reflects the sky as the
  // screen shows it, exact in colour and fog, in place of the cube's, whose texel is coarser and whose sky is the
  // frame's sky at best.
  if (end == END_FAR && all(walk.position.xy >= vec2<f32>(0.0)) && all(walk.position.xy <= vec2<f32>(1.0))) {
    let end_texel: vec2<i32> = walk_texel(walk.position.xy);
    let end_depth: f32 = textureLoad(depth_target, end_texel, 0);

    if (end_depth <= 0.0 || path_fog(lighting, camera.projection[3][2] / (end_depth + camera.projection[2][2])) >= SOURCE_FOG_LIMIT) {
      let seen: vec4<f32> = remembered_sky(world_direction(direction));

      if (seen.w > 0.5) {
        sky = seen.rgb * specular_occlusion(facing, visible, roughness);
        end = END_SKY;
      }
    }
  }

  // Uncertain a little behind a surface: that surface's colour is the probable answer, the sky's the further behind.
  if (walk.end == END_UNCERTAIN) {
    let behind_texel: vec2<i32> = walk_texel(walk.position.xy);
    let is_plant: bool = has_mark(textureLoad(material_target, behind_texel, 0).a, MARK_PLANT);
    let band: vec2<f32> = select(PROBABLE_BEHIND, PROBABLE_BEHIND_PLANT, is_plant);
    let probable: f32 = 1.0 - smoothstep(thickness, thickness + band.x + distance_met * band.y, walk.behind);

    if (probable > 0.0) {
      let shown: vec3<f32> = shown_point(behind_texel).xyz;
      let taken: vec4<f32> = met_radiance(behind_texel, shown, position);

      let probable_end: u32 = select(END_UNCERTAIN, END_PROBABLE, probable > 0.5) | select(END_RELIT, 0u, taken.w > 0.5);

      return answer_trace(vec4<f32>(sky, missed_length), vec4<f32>(taken.rgb, length(shown - position)), probable,
        probable_end, probable_end);
    }
  }

  if (confidence <= 0.0) {
    return answer_trace(vec4<f32>(sky, missed_length), vec4<f32>(0.0), 0.0,
      select(end, END_UNTRUSTED, walk.is_met && end != END_HELD), END_TRACED);
  }

  let hit: vec3<f32> = ray_view_point(walk.position);
  let met: vec4<f32> = met_radiance(walk_texel(walk.position.xy), hit, position);
  let length_met: f32 = length(hit - position);

  let met_end: u32 = walk.end | select(END_RELIT, 0u, met.w > 0.5);

  return answer_trace(vec4<f32>(sky, missed_length), vec4<f32>(met.rgb, length_met), confidence, met_end, met_end);
}

// Ray budgets: rays a quad of traced pixels takes of its four by what its pixels show, Battlefield V's tiles' and AMD
// SSSR's quads': every pixel where a puddle or a bright, sharp reflection stands, whose detail its neighbours would
// smear; one where its reflection is dim or rough, which the denoiser blurs across them anyway; two between. A pixel
// left takes the answer of the traced pixel of its quad whose surface stands nearest its own and faces most alike, as
// SSSR copies its rays to their neighbours, the scene pass's hit too (`TRACE_REQUEST_MATES_SHIFT`); the pixels traced
// turn each frame, so each is walked in turn and the history gathers them. A pixel the last frame left noisy asks a
// step more, as SSSR adds rays where its variance stayed high.
//
// The roughness under which a bright reflection is sharp, and over which any is blurred.
const SHARP_ROUGHNESS: f32 = 0.1;
const SHARED_ROUGHNESS: f32 = 0.3;
// The cosine of the view to a surface under which it is seen at a grazing angle, its reflection stretched along the
// view, and never shared four ways.
const GRAZING_FACING: f32 = 0.25;
// The variance the last frame's reflection kept over which a pixel asks a step more.
const NOISY_VARIANCE: f32 = 0.05;
// Traced pixels a side each workgroup of the budgeted trace takes, as quads one an invocation.
const BUDGET_TILE: u32 = 16u;
// The order a quad's pixels take its rays in, the diagonals first, so two rays fall across each other.
const QUAD_ORDER: array<u32, 4> = array<u32, 4>(0u, 3u, 1u, 2u);

// The rays of its quad's four a traced pixel asks for.
fn ray_need(start: TraceStart, pixel: vec2<u32>) -> u32 {
  var rays: u32 = 2u;

  if (start.is_puddle || (start.roughness < SHARP_ROUGHNESS && start.shown >= DIM_SHARE)) {
    rays = 4u;
  } else if ((start.shown < DIM_SHARE || start.roughness > SHARED_ROUGHNESS) && start.facing >= GRAZING_FACING) {
    rays = 1u;
  }

  if (rays < 4u && reflection.has_history > 0.5) {
    let last: vec2<u32> = textureDimensions(history_radiance) - 1u;

    if (textureLoad(history_radiance, min(pixel, last), 0).a > NOISY_VARIANCE) {
      rays *= 2u;
    }
  }

  return rays;
}

// How alike two traced pixels' surfaces stand for one to take the other's answer: their normals' cosine raised as the
// upscale raises it, over how far apart they stand of the first's distance.
fn surface_likeness(a: TraceStart, b: TraceStart) -> f32 {
  let difference: f32 = abs(a.position.z - b.position.z) / max(-a.position.z * PREFILTER_PLANE, 1e-3);

  return pow(saturate(dot(a.normal, b.normal)), UPSAMPLE_NORMAL_POWER) / (1.0 + difference * difference);
}

fn store_trace(pixel: vec2<u32>, written: TraceTargets) {
  textureStore(traced_out, pixel, written.traced);
  textureStore(ends_out, pixel, vec4<f32>(written.end, 0.0, 0.0, 0.0));
}

// Each listed ray: its pixel's place in the tile, four bits a side, then from the eighth bit the corners of its quad
// that take its answer.
var<workgroup> budget_rays: array<u32, 256>;
var<workgroup> budget_count: atomic<u32>;

// The trace under ray budgets: a tile of traced pixels a workgroup, each invocation one quad's. Each quad finds what
// its pixels show and how many rays they ask, writes its untraced pixels, gives each pixel its budget leaves to the
// most alike of those it traces, and lists those; then the workgroup's invocations walk the listed rays packed, so a
// quad tracing one ray holds no lanes idle, and each writes its answer to its pixel and the pixels it stands for.
@compute @workgroup_size(64)
fn cs_trace_budgeted(@builtin(workgroup_id) group: vec3<u32>, @builtin(local_invocation_index) local: u32) {
  if (local == 0u) {
    atomicStore(&budget_count, 0u);
  }

  workgroupBarrier();

  let size: vec2<u32> = textureDimensions(traced_out);
  let tile: vec2<u32> = group.xy * BUDGET_TILE;
  let place: vec2<u32> = vec2<u32>(local % (BUDGET_TILE / 2u), local / (BUDGET_TILE / 2u)) * 2u;
  var starts: array<TraceStart, 4>;
  var needs: array<u32, 4> = array<u32, 4>(0u, 0u, 0u, 0u);
  var budget: u32 = 0u;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let pixel: vec2<u32> = tile + place + vec2<u32>(corner & 1u, corner >> 1u);

    starts[corner] = untraced_start(END_UNTRACED);

    if (any(pixel >= size)) {
      continue;
    }

    starts[corner] = trace_start(vec2<f32>(pixel));

    if (!starts[corner].is_traced) {
      store_trace(pixel, trace_targets(UNTRACED, starts[corner].end));
      continue;
    }

    needs[corner] = ray_need(starts[corner], pixel);
    budget = max(budget, needs[corner]);
  }

  // The quad's first pixel to trace turns each frame from a place its noise picks; two rays take a diagonal.
  let noise_size: vec2<u32> = textureDimensions(blue_noise);
  let picked: u32 = u32(textureLoad(blue_noise, ((tile + place) / 2u) % noise_size, 0).r * 4.0);
  let turn: u32 = (reflection.frame + picked) % 4u;
  let first: u32 = select(turn, turn & 2u, budget == 2u);
  var is_traced: array<bool, 4> = array<bool, 4>(false, false, false, false);
  var taken: u32 = 0u;

  for (var step: u32 = 0u; step < 4u; step++) {
    let corner: u32 = QUAD_ORDER[(first + step) % 4u];

    if (needs[corner] > 0u && taken < budget) {
      is_traced[corner] = true;
      taken++;
    }
  }

  var mates: array<u32, 4> = array<u32, 4>(0u, 0u, 0u, 0u);

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    if (needs[corner] == 0u || is_traced[corner]) {
      continue;
    }

    var most: f32 = -1.0;
    var nearest: u32 = 0u;

    for (var other: u32 = 0u; other < 4u; other++) {
      let likeness: f32 = surface_likeness(starts[corner], starts[other]);

      if (is_traced[other] && likeness > most) {
        most = likeness;
        nearest = other;
      }
    }

    mates[nearest] |= 1u << corner;
  }

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    if (is_traced[corner]) {
      let at: vec2<u32> = place + vec2<u32>(corner & 1u, corner >> 1u);

      budget_rays[atomicAdd(&budget_count, 1u)] = at.x | (at.y << 4u) | (mates[corner] << 8u);
    }
  }

  workgroupBarrier();

  let count: u32 = atomicLoad(&budget_count);

  for (var index: u32 = local; index < count; index += 64u) {
    let packed: u32 = budget_rays[index];
    let pixel: vec2<u32> = tile + vec2<u32>(packed & 15u, (packed >> 4u) & 15u);
    let quad: vec2<u32> = pixel & vec2<u32>(~1u);

    scene_mates = packed >> 8u;

    let written: TraceTargets = trace_pixel(vec2<f32>(pixel));

    store_trace(pixel, written);

    for (var corner: u32 = 0u; corner < 4u; corner++) {
      if ((scene_mates & (1u << corner)) != 0u) {
        store_trace(quad + vec2<u32>(corner & 1u, corner >> 1u),
          TraceTargets(written.traced, reflection_end_value(END_SHARED)));
      }
    }
  }
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
  let wet: vec4<f32> = load_frame_target(wet_surface, texel);

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
    (normal.y > LEVEL_UP || load_frame_target(wet_surface, texel).g > PUDDLE_LEAST);
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
  let before: f32 = traced_luminance(held);
  let now: f32 = traced_luminance(fresh);
  let difference: f32 = abs(before - now) / max(max(before, now), 0.5);

  return difference * difference;
}

// The history reprojected and how many frames it holds, then its variance.
struct ReprojectedTargets {
  @location(0) reprojected: vec4<f32>,
  @location(1) variance: f32,
};

// The last frame's reflection at a traced pixel: read where the reflected point stood (the view's ray carried on
// through the surface by the hit's length) where the normals there match as a mirror's would, else where the surface
// stood if it keeps near the neighbourhood; searched about where the history disagrees, dropped where it still does,
// and for a mirror. Holds more frames the rougher the surface.
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
      let weight: f32 = max(exp(-traced_luminance(radiance) * AVERAGE_LUMINANCE_WEIGHT), 1e-2);

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

  if (spread <= 0.0 || surface.roughness < 1e-4 || surface.roughness > lighting.reflection_cut.y) {
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

// How far apart two surfaces' normals may face and their roughnesses stand before a traced pixel stops speaking for a
// frame's pixel: the normals' cosine raised to `UPSAMPLE_NORMAL_POWER`, the roughness by `UPSAMPLE_ROUGHNESS_SCALE` a
// unit of difference.
const UPSAMPLE_NORMAL_POWER: f32 = 8.0;
const UPSAMPLE_ROUGHNESS_SCALE: f32 = 20.0;

// The resolve brought up to a frame pixel by the guide: the four traced pixels about it by their bilinear share, how
// near the distance each one's surface stands to the pixel's, how alike its world normal (terrain's levelled, as the
// guide keeps it) faces to `normal`, and how near its roughness stands to `roughness`, each read from the guide in one
// load. Untraced, alpha below none, where none speaks for it.
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

// Whether any of the four traced pixels about a frame pixel holds a reflection; over a dry frame none does, and the
// pixel is untraced before anything else is read for it.
fn is_any_traced(pixel: vec2<f32>) -> bool {
  let last: vec2<f32> = vec2<f32>(textureDimensions(resolved)) - 1.0;
  let base: vec2<f32> = floor(pixel / reflection.ratio);

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let offset: vec2<f32> = vec2<f32>(f32(corner & 1u), f32(corner >> 1u));

    if (textureLoad(resolved, vec2<i32>(clamp(base + offset, vec2<f32>(0.0), last)), 0).a >= 0.0) {
      return true;
    }
  }

  return false;
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

  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let distance: f32 = -position.z;
  // At the frame's own size the resolve already holds the pixel: read, not upsampled.
  var current: vec4<f32>;

  if (reflection.ratio <= 1.0) {
    current = textureLoad(resolved, min(texel, vec2<i32>(textureDimensions(resolved)) - 1), 0);
  } else if (!is_any_traced(pixel)) {
    return UNTRACED;
  } else {
    let wet: vec4<f32> = load_frame_target(wet_surface, texel);
    let roughness: f32 = traced_roughness(textureLoad(albedo_target, texel, 0).a,
      wet.a + surface_variance(normal_target, depth_target, texel, wet.g));

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
