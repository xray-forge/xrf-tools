#import "common/depth_plane"
#import "common/noise"
#import "common/water_surface"
#import "common/scene_light"
#import "common/display_transform"
#import "common/water_enhanced"
#import "common/trace_requests"
#import "common/water_rays"
#import "common/water_reflection_read"
#import "generated/static/water_reflection"

// The reflection's walk over the pyramid of the frame's nearest depth, its hits and probable answers the surfaces' rays'
// (`HIT_THICKNESS`, `probable_behind`): how far it reaches, and the cells it crosses for each of the settings' march
// steps (`EnhancedWater::march_steps`).
const MARCH_REACH: f32 = 150.0;
const MARCH_CROSSINGS: u32 = 4u;

// A surface nearer than this, where a game's weapon would stand, is passed over rather than reflected.
const MARCH_NEAREST: f32 = 1.3;

// Metres in front of what lies under the water's start a surface may stand and still be the screen's answer where a
// ray runs out over the ground.
const MARCH_GROUND_FRONT: f32 = 2.0;

// How much of the reflection the last frames keep, and how much less of it where the ray met the sky.
const REFLECTION_HISTORY: f32 = 0.97;
const REFLECTION_SKY_HISTORY: f32 = 0.2;

// Pixels of motion over which the history gives way entirely: a fast turn draws the reflection afresh rather than drag
// the last frames behind it.
const HISTORY_MOTION: f32 = 40.0;

// How far a surface the screen met may face along the ray before the trace answers in its place.
const FACING_AWAY: f32 = 0.1;

// Pixels a side of the window whose fresh pixels bound what a pixel keeps of its history.
const FRESH_WINDOW: i32 = 2;

// The distance this frame's reflection gives a pixel that holds what it reflected the frame before: off its turn to
// trace, or past the list's room.
const WATER_HELD: f32 = -2.0;

// Where a pixel's point stood on the last frame's screen: along its view as far as what its ray met, or far off at the
// sky where it met nothing or holds.
fn previous_place(uv: vec2<f32>, eye: vec3<f32>, met_distance: f32) -> vec2<f32> {
  let hit_distance: f32 = select(met_distance, 1e5, met_distance <= 0.0);
  let along: vec3<f32> = eye * (hit_distance / max(-eye.z, 1e-4));
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
  let world_hit: vec4<f32> = vec4<f32>(camera.position.xyz + rotation * along, 1.0);
  let now: vec4<f32> = camera.motion_current * world_hit;
  let before: vec4<f32> = camera.motion_previous * world_hit;

  return uv - (now.xy / now.w - before.xy / before.w) * vec2<f32>(0.5, -0.5);
}

// Whether a place is off the last frame's screen.
fn is_off_screen(previous: vec2<f32>) -> bool {
  return any(previous < vec2<f32>(0.0)) || any(previous > vec2<f32>(1.0));
}

// The distance the history keeps at a place on the last frame's screen, none off it or where no water drew.
fn history_distance(previous: vec2<f32>) -> f32 {
  let size: vec2<f32> = camera.viewport.xy;
  let pixel: vec2<i32> = vec2<i32>(clamp(floor(previous * size), vec2<f32>(0.0), size - 1.0));

  return select(textureLoad(reflection_history, pixel, 0).a, 0.0, is_off_screen(previous));
}

// Where a held pixel's reflection stood on the last frame's screen, its distance unknown this frame: first as the sky
// would, then again as far as the history kept there, so a near reflection follows the eye's motion rather than lag
// behind it as the sky's would.
fn held_place(uv: vec2<f32>, eye: vec3<f32>) -> vec2<f32> {
  let far: vec2<f32> = previous_place(uv, eye, -1.0);
  let kept: f32 = history_distance(far);

  if (kept <= 0.0 || kept >= SKY_DISTANCE) {
    return far;
  }

  let near: vec2<f32> = previous_place(uv, eye, kept);

  return select(far, near, history_distance(near) > 0.0);
}

// The distance along the view of a reflection whose ray left a point of the water (in view space) and met what it met
// `travelled` on: where the mirror shows it, behind the water along the pixel's view, which its motion follows.
fn mirrored_distance(position: vec3<f32>, travelled: f32) -> f32 {
  return -position.z * (1.0 + travelled / max(length(position), 1e-4));
}

// The share of what it reflects under which a water's pixel shows little of its reflection (its fresnel cubed by the
// reflectivity, as the surface mixes it in, over its flat normal), and the frame's pixels its point may have moved
// since the last frame for it to be still: a still, dim reflection waits `dim_turns` times its turns between rays, as
// the surfaces' do, its history held between; a moving one traces at its turns.
const WATER_DIM_SHARE: f32 = 0.15;
const WATER_DIM_STILL: f32 = 2.0;

// Frames a water's pixel at `world`, its flat surface facing `normal`, waits between its rays, stretched by the list's
// `wait` where the rays asked overrun its room.
fn water_trace_turns(world: vec3<f32>, normal: vec3<f32>, wait: u32) -> u32 {
  let turns: u32 = max(enhanced.trace_turns, 1u) * wait;

  if (enhanced.dim_turns <= 1u) {
    return turns;
  }

  let to_point: vec3<f32> = normalize(world - camera.position.xyz);
  let fresnel: f32 = pow(saturate(dot(reflect(to_point, normal), to_point)), 3.0);
  let now: vec4<f32> = camera.motion_current * vec4<f32>(world, 1.0);
  let before: vec4<f32> = camera.motion_previous * vec4<f32>(world, 1.0);
  let moved: f32 = length((now.xy / now.w - before.xy / before.w) * 0.5 * camera.viewport.xy);
  let is_still_dim: bool = fresnel * enhanced.reflectivity < WATER_DIM_SHARE && moved < WATER_DIM_STILL;

  return select(turns, turns * enhanced.dim_turns, is_still_dim);
}

// Whether a pixel the screen does not answer traces this frame: once in its `turns`, each pixel's turn its own
// (`trace_turn_of`), so every frame traces pixels spread over the whole water and a held pixel has fresh ones about it.
// The scene pass traces the listed rays packed, so scattered turns cost its waves nothing.
fn is_water_trace_turn(pixel: vec2<f32>, turns: u32) -> bool {
  return is_on_trace_turn(vec2<u32>(pixel), turns, enhanced.trace_frame);
}

// The ray a water's pixel leaves to the scene pass where the screen answers it only in part or not at all: from its
// point along its flat reflection, lifted off the water, as far as the trace reaches, a pixel wide where it leaves and
// widening by the pixel's angle; with what the screen answered (`scene` weighed by its `share`) and the distance along
// the view of what it met, none where it met nothing; whether the list took it.
fn request_water_trace(pixel: vec2<f32>, world: vec3<f32>, normal: vec3<f32>, toward: vec3<f32>, distance: f32,
  answer: vec4<f32>, share: f32) -> bool {
  let pixel_angle: f32 = 2.0 / (camera.projection[1][1] * camera.viewport.y);
  var request: TraceRequest;

  request.origin = pack_trace_request_origin(world + normal * TRACED_LIFT);
  request.reach = pack2x16float(vec2<f32>(enhanced.trace_distance, answer.w));
  request.cone = pack2x16float(vec2<f32>(distance * pixel_angle, pixel_angle));
  request.toward = pack2x16snorm(octahedral_encode(toward));
  request.pixel = u32(pixel.x) | (u32(pixel.y) << 16u);
  request.screen = trace_request_screen(answer.rgb, share);
  request.hit_end = TRACE_REQUEST_WATER | (pack2x16float(vec2<f32>(water.reflection, 0.0)) & 0xffffu);

  return list_trace_request(request);
}

// The record a pixel leaves the scene pass while every pixel traces every frame, in place of a request: its ray's
// direction, what the screen answered (`scene` weighed by its `share`) and the distance along the view of what it met,
// none where it met nothing, on this frame (`common/water_rays`).
fn write_water_ray(pixel: vec2<u32>, toward: vec3<f32>, answer: vec4<f32>, share: f32) {
  let at: u32 = water_ray_at(pixel);

  if (at + WATER_RAY_WORDS > arrayLength(&water_rays)) {
    return;
  }

  let screen: vec2<u32> = trace_request_screen(answer.rgb, share);

  water_rays[at] = pack2x16snorm(octahedral_encode(toward));
  water_rays[at + 1u] = screen.x;
  water_rays[at + 2u] = screen.y;
  water_rays[at + 3u] = water_ray_last(answer.w, enhanced.trace_frame);
}

@vertex
fn vs_water(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> WaterVarying {
  return water_vertex(vertex_index, instance_index, water);
}

// The view space point at a place on the screen, a distance along the view: through the whole projection, its
// off-centre terms the temporal jitter moves included, as `ray_screen_point` projects back.
fn view_point(at: vec2<f32>, distance: f32) -> vec3<f32> {
  let ndc: vec2<f32> = (at * 2.0 - 1.0) * vec2<f32>(1.0, -1.0);
  let centred: vec2<f32> = ndc + vec2<f32>(camera.projection[2][0], camera.projection[2][1]);

  return vec3<f32>(centred.x / camera.projection[0][0], centred.y / camera.projection[1][1], -1.0) * distance;
}

// The view space point the screen shows at a place.
fn screen_point(at: vec2<f32>) -> vec3<f32> {
  return view_point(at, scene_distance(at, depth_target));
}

// Whether the surface the screen shows at a place faces away from a ray that met it: a face the mirror cannot see, such
// as a stone's sunlit top over the water, which the march meets only because the eye sees it. Its normal is the depth's
// slope towards the nearer neighbour each way, so an edge does not tilt it.
fn is_facing_away(at: vec2<f32>, direction: vec3<f32>) -> bool {
  let texel: vec2<f32> = 1.0 / camera.viewport.xy;
  let centre: vec3<f32> = screen_point(at);
  let normal: vec3<f32> = surface_normal(
    centre,
    vec4<f32>(screen_point(at - vec2<f32>(texel.x, 0.0)), 1.0),
    vec4<f32>(screen_point(at + vec2<f32>(texel.x, 0.0)), 1.0),
    vec4<f32>(screen_point(at - vec2<f32>(0.0, texel.y)), 1.0),
    vec4<f32>(screen_point(at + vec2<f32>(0.0, texel.y)), 1.0),
  );

  return dot(normal, direction) > FACING_AWAY;
}

// The ray from a point of the water (in view space) along its reflection, walked over the pyramid of the frame's
// nearest depth as the surfaces' rays are (`ray_walk`), so a wire or a cable a pixel wide stops it where fixed steps
// passed over it, and a ray passing behind one goes on; but a plant's texel is the crown or the reeds it stands in
// for, and a ray gone behind it has met it, so a poplar drawn a few texels wide at a low drawn size holds rather than
// let the jitter choose which rays slip through to the bank. The surface the screen answers with, its place and
// distance, then how sure the screen is of it: sure where the ray met it; as sure as it is probably still in what it
// went behind and stayed behind (a wall, a hill); sure of the ground where the ray ran its reach out over it, the
// screen's last answer. Nothing where it left the screen, met only the sky, ran its crossings out, or met a surface
// nearer than `MARCH_NEAREST`. Squeezed across near the screen's sides, unless the eye looks down.
fn march_reflection(start: vec3<f32>, direction: vec3<f32>, uv: vec2<f32>) -> vec4<f32> {
  // A ray turning towards the eye stops short of the near plane rather than wrap behind it.
  let reach: f32 = select(MARCH_REACH, min(MARCH_REACH, (-0.1 - start.z) / direction.z), direction.z > 0.0);
  let origin: vec3<f32> = ray_screen_point(start);
  var toward: vec3<f32> = ray_screen_point(start + direction * max(reach, 1e-3)) - origin;
  let edges: vec2<f32> = 1.0 - smoothstep(vec2<f32>(0.9), vec2<f32>(1.0), vec2<f32>(uv.x, 1.0 - uv.x));
  let looking_down: f32 = saturate(-camera.view[1].z * 3.0);

  toward.x *= saturate(edges.x * edges.y + looking_down);

  let walk: RayWalk = ray_walk(nearest_depth, material_target, true, origin, toward,
    enhanced.march_steps * MARCH_CROSSINGS, HIT_THICKNESS);
  let at: vec2<f32> = walk.position.xy;

  if (walk.end == END_OFF_SCREEN || any(at < vec2<f32>(0.0)) || any(at > vec2<f32>(1.0))) {
    return vec4<f32>(0.0);
  }

  let pixel: vec2<i32> = walk_pixel(at);
  let distance: f32 = scene_distance(at, depth_target);

  if (textureLoad(depth_target, pixel, 0) <= 0.0 || distance <= MARCH_NEAREST) {
    return vec4<f32>(0.0);
  }

  if (walk.is_met) {
    return vec4<f32>(at, distance, 1.0);
  }

  // Behind no plant: those the walk met.
  if (walk.end == END_UNCERTAIN) {
    return vec4<f32>(at, distance,
      probable_behind(walk.behind, HIT_THICKNESS.x + distance * HIT_THICKNESS.y, distance, false));
  }

  // Only a ray that ran its reach out over the ground ends in front of it; one that ran its crossings out may stand
  // behind anything, a cable it was passing included, and is left to the trace or the sky.
  let is_ground: bool = walk.end == END_FAR && scene_distance(origin.xy, depth_target) - MARCH_GROUND_FRONT < distance;

  return select(vec4<f32>(0.0), vec4<f32>(at, distance, 1.0), is_ground);
}

// The drawn texel a walk's place on the screen falls in.
fn walk_pixel(at: vec2<f32>) -> vec2<i32> {
  let size: vec2<f32> = camera.viewport.xy;

  return vec2<i32>(clamp(floor(at * size), vec2<f32>(0.0), size - 1.0));
}

// The scene the water's flat surface reflects, marched over the screen as it stood before the water and faded towards
// the top of the screen and into the fog, the sky where the ray met nothing, with the distance along the view of what
// it met; a ray the screen answers only in part listed for the scene pass, whose hit replaces the sky.
@fragment
fn fs_water_reflection(in: WaterVarying) -> @location(0) vec4<f32> {
  let position: vec3<f32> = (camera.view * vec4<f32>(in.world, 1.0)).xyz;
  let fog: f32 = select(0.0, fog_amount(lighting, position), lighting.params.y > 0.5);
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);

  // A fragment listing rays writes storage, which holds the depth test back until after it shades: one behind the
  // scene is left out here instead, before it marches or lists anything.
  if (fog >= 1.0 || in.clip.z < textureLoad(nearest_water, texel, 0) || in.clip.z <= textureLoad(depth_target, texel, 0)) {
    discard;
  }

  let size: vec2<f32> = camera.viewport.xy;
  let uv: vec2<f32> = (floor(in.clip.xy) + 0.5) / size;
  let normal: vec3<f32> = normalize(in.normal);
  let eye: vec3<f32> = normalize(position);
  let normal_view: vec3<f32> = normalize((camera.view * vec4<f32>(normal, 0.0)).xyz);
  let reflected: vec3<f32> = reflect(eye, normal_view);
  let to_point: vec3<f32> = normalize(in.world - camera.position.xyz);
  let flat_reflected: vec3<f32> = reflect(to_point, normal);
  // Rays towards the eye are not marched: the screen holds nothing they reach, and looking down they only mess the
  // reflection; the screen's answer fades out towards them. The trace takes every ray, however little the flat surface
  // shows of it: rays towards the eye left to the sky make a circle of one viewing angle about the eye that cuts a
  // bridge's reflection off short.
  let away: f32 = smoothstep(-0.1, -0.4, dot(-eye, reflected));
  let is_away: bool = away > 0.0;
  // A branch rather than a select, which would march every ray to throw most away.
  let is_traced: bool = enhanced.trace_distance > 0.0;
  var hit: vec4<f32> = vec4<f32>(0.0);

  if (is_away) {
    hit = march_reflection(position, reflected, uv);
  }

  // Fully traced, the screen answers only the plants past the grass's cards (`plant_reach`, from the eye), which the
  // level's trace does not hold; every other ray is traced, so a cable's or a pipe's reflection is what the ray meets
  // lit, not the side the eye sees, and a reed's its blades, not its screen copy stretched down behind it.
  if (is_traced && enhanced.fully_traced != 0u && hit.w > 0.0 &&
    (!has_mark(textureLoad(material_target, walk_pixel(hit.xy), 0).a, MARK_PLANT) ||
      length(view_point(hit.xy, hit.z)) < enhanced.plant_reach)) {
    hit = vec4<f32>(0.0);
  }
  // The sky's picture as the frame's display brings it in, so the reflection holds light throughout, as what the
  // screen and the trace answer is.
  let sky: vec3<f32> = picture_for_display(
    enhanced_sky(flat_reflected, lighting, sky_cube_0, sky_cube_1, sky_clamp) * water.reflection,
    lighting.display,
  );
  // The screen answers first, as it does untraced: the trace takes only what it misses (off the screen, the sky, past
  // its reach), and a surface the screen met facing away from the ray, which the mirror cannot see.
  let is_met: bool = hit.w > 0.0 && !(is_traced && is_away && is_facing_away(hit.xy, reflected));
  let screen_share: f32 = select(0.0, saturate(hit.y * 5.0) * away * hit.w, is_met);
  var scene: vec3<f32> = vec3<f32>(0.0);
  var share: f32 = 0.0;

  if (screen_share > 0.0) {
    scene = scene_shown(textureLoad(water_scene, vec2<i32>(clamp(floor(hit.xy * size), vec2<f32>(0.0), size - 1.0)), 0).rgb);
    share = screen_share *
      (1.0 - saturate((length(vec3<f32>(position.xy, hit.z)) * lighting.fog.y + lighting.fog.x) * 1.4));
  }

  // The distance along the view of what the ray met on the screen, where the mirror shows it; none where it met nothing
  // there.
  let travelled: f32 = distance(view_point(hit.xy, hit.z), position);
  var met_distance: f32 = select(-1.0, mirrored_distance(position, travelled), screen_share > 0.0);

  // Where the screen answers the ray only in part or not at all, the scene pass traces it on its turn, a hit written
  // over the sky; off its turn, or past the list's room, the pixel holds what it reflected the frame before. Only turns
  // trace, so the list always has room: water with no history to hold (newly in view, a fast turn's edge) takes its
  // fresh neighbours' instead, since tracing it too would overflow the list wherever much of the water lost its
  // history. Every pixel tracing every frame leaves its record at its place instead.
  if (screen_share < 1.0 && is_traced) {
    let pixel: vec2<f32> = floor(in.clip.xy);
    let answer: vec4<f32> = vec4<f32>(scene, met_distance);

    if (enhanced.fully_traced != 0u) {
      write_water_ray(vec2<u32>(pixel), flat_reflected, answer, share);
    } else if (!(is_water_trace_turn(pixel, water_trace_turns(in.world, normal, trace_request_wait())) &&
      request_water_trace(pixel, in.world, normal, flat_reflected, -position.z, answer, share))) {
      met_distance = WATER_HELD;
    }
  }

  return vec4<f32>(mix(sky, scene, share), met_distance);
}

// What this frame's fresh pixels (traced, or answered by the screen) show about a pixel: the colours they span, their
// mean colour and distance (the sky's for a ray that met nothing), and whether there are any.
struct FreshBounds {
  low: vec3<f32>,
  high: vec3<f32>,
  mean: vec3<f32>,
  distance: f32,
  is_found: bool,
};

fn fresh_bounds(pixel: vec2<i32>) -> FreshBounds {
  let last: vec2<i32> = vec2<i32>(textureDimensions(water_current)) - 1;
  var bounds: FreshBounds = FreshBounds(vec3<f32>(1e9), vec3<f32>(-1e9), vec3<f32>(0.0), 0.0, false);
  var count: f32 = 0.0;

  for (var y: i32 = -FRESH_WINDOW; y <= FRESH_WINDOW; y++) {
    for (var x: i32 = -FRESH_WINDOW; x <= FRESH_WINDOW; x++) {
      let fresh: vec4<f32> = textureLoad(water_current, clamp(pixel + vec2<i32>(x, y), vec2<i32>(0), last), 0);

      if (fresh.a != 0.0 && fresh.a != WATER_HELD) {
        bounds.low = min(bounds.low, fresh.rgb);
        bounds.high = max(bounds.high, fresh.rgb);
        bounds.mean += fresh.rgb;
        bounds.distance += select(fresh.a, SKY_DISTANCE, fresh.a <= 0.0);
        count += 1.0;
      }
    }
  }

  bounds.is_found = count > 0.0;
  bounds.mean /= max(count, 1.0);
  bounds.distance /= max(count, 1.0);

  return bounds;
}

// Where a pixel reads its history this frame, from nothing to one each way: hashed from the pixel and the trace's
// frame as whole numbers, so it stays a pixel's own however long the view runs; a clock in seconds scaled up to hash
// steps coarser than a pixel late in its period.
fn resolve_jitter(pixel: vec2<u32>) -> vec2<f32> {
  return vec2<f32>(pcg3d(vec3<u32>(pixel, enhanced.trace_frame)).xy >> vec2<u32>(8u)) / 16777216.0;
}

// This frame's reflection, its traced hits written in, kept over the frames before, read where this one's point stood
// in the last: along this pixel's view as far as what the ray met, or far off at the sky, or for a held pixel as far
// as the history kept; with that distance, which the next frame's held pixels follow.
@fragment
fn fs_water_resolve(in: WaterVarying) -> @location(0) vec4<f32> {
  let position: vec3<f32> = (camera.view * vec4<f32>(in.world, 1.0)).xyz;
  let fog: f32 = select(0.0, fog_amount(lighting, position), lighting.params.y > 0.5);

  if (fog >= 1.0 || in.clip.z < textureLoad(nearest_water, vec2<i32>(in.clip.xy), 0)) {
    discard;
  }

  let size: vec2<f32> = camera.viewport.xy;
  let uv: vec2<f32> = (floor(in.clip.xy) + 0.5) / size;
  let eye: vec3<f32> = normalize(position);
  let current: vec4<f32> = textureLoad(water_current, vec2<i32>(in.clip.xy), 0);
  let reflection: vec3<f32> = current.rgb;
  let is_held: bool = current.a == WATER_HELD;
  let met_distance: f32 = select(current.a, -1.0, is_held);
  let previous: vec2<f32> = select(previous_place(uv, eye, met_distance), held_place(uv, eye), is_held);
  let moved: vec2<f32> = uv - previous;
  let is_off: bool = is_off_screen(previous);
  // A pixel on its turn brings the only fresh ray of its turns, so it keeps of the history what as many frames would;
  // the list's wait is not read here, so a stretched turn keeps a little less.
  let turns: u32 = water_trace_turns(in.world, normalize(in.normal), 1u);
  let frames: f32 = select(1.0, f32(turns),
    enhanced.trace_distance > 0.0 && is_water_trace_turn(floor(in.clip.xy), turns));
  let kept: f32 =
    pow(saturate(REFLECTION_HISTORY - select(0.0, REFLECTION_SKY_HISTORY, met_distance <= 0.0) - f32(is_off)), frames) *
    enhanced.history;
  let jitter: vec2<f32> = (resolve_jitter(vec2<u32>(in.clip.xy)) * 2.0 - 1.0) / size * 0.25;
  // The history read off its water alone (`read_water_reflection`): where no water drew it holds nothing, which a
  // plain linear read beside a reed or a pipe would mix in as black for the history to keep.
  let history_read: vec4<f32> = read_water_reflection(reflection_history, previous + jitter);
  // The history held within what this frame's fresh pixels about it show, so a reflection that moved onto a pixel,
  // which its history never had, stands at once rather than when its turn comes.
  let bounds: FreshBounds = fresh_bounds(vec2<i32>(in.clip.xy));
  let history: vec3<f32> =
    select(history_read.rgb, clamp(history_read.rgb, bounds.low, bounds.high), bounds.is_found);
  // Whether the last frame drew water about there: newly seen water keeps none of the history.
  let is_history: f32 = f32(history_read.w > 0.0 && !is_off);
  let settled: f32 = saturate(1.0 - length(moved * size) / HISTORY_MOTION);

  // A held pixel keeps the history whole where it holds a frame there; where it does not, it takes what its fresh
  // neighbours show, and where there are none either it shows its sky fallback but leaves no history of it, so the
  // answer its next ray brings stands at once rather than fading in over the white the history would have kept.
  let weight: f32 = select(kept * is_history * settled, is_history * enhanced.history, is_held);
  let is_unheld: bool = is_held && weight < 0.5;
  let shown: vec3<f32> = select(mix(reflection, history, weight), bounds.mean, is_unheld && bounds.is_found);
  var kept_distance: f32 = select(select(met_distance, SKY_DISTANCE, met_distance <= 0.0), history_distance(previous),
    is_held);

  kept_distance = select(kept_distance, select(0.0, bounds.distance, bounds.is_found), is_unheld);

  return vec4<f32>(shown, min(kept_distance, SKY_DISTANCE));
}
