#import "common/depth_plane"
#import "common/noise"
#import "common/water_surface"
#import "common/scene_light"
#import "common/water_enhanced"
#import "common/trace_requests"
#import "generated/static/water_reflection"

// The reflection's march, its steps the settings' (`EnhancedWater::march_steps`): its refinements of a hit, how thick a
// surface it takes for one, traced or not, and how far it reaches.
const MARCH_REFINES: i32 = 2;
const MARCH_THICKNESS: f32 = 3.0;
const MARCH_REACH: f32 = 150.0;

// How much of the reflection the last frames keep, and how much less of it where the ray met the sky.
const REFLECTION_HISTORY: f32 = 0.97;
const REFLECTION_SKY_HISTORY: f32 = 0.2;

// Pixels of motion over which the history gives way entirely: a fast turn draws the reflection afresh rather than drag
// the last frames behind it.
const HISTORY_MOTION: f32 = 40.0;

// Metres a traced ray starts off the water, so its own surface's triangles beneath do not meet it.
const TRACED_LIFT: f32 = 0.05;

// How far a surface the screen met may face along the ray before the trace answers in its place.
const FACING_AWAY: f32 = 0.1;

// Pixels a side of the window whose fresh pixels bound what a pixel keeps of its history.
const FRESH_WINDOW: i32 = 2;

// The distance this frame's reflection gives a pixel that holds what it reflected the frame before: off its turn to
// trace, or past the list's room.
const WATER_HELD: f32 = -2.0;

// The distance along the view the history keeps for a pixel whose ray met the sky; the history keeps none where no
// water drew.
const SKY_DISTANCE: f32 = 60000.0;

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
  return (trace_turn_of(vec2<u32>(pixel), turns) + enhanced.trace_frame) % turns == 0u;
}

// The ray a water's pixel leaves to the scene pass where the screen answers it only in part or not at all: from its
// point along its flat reflection, lifted off the water, as far as the trace reaches, a pixel wide where it leaves and
// widening by the pixel's angle; with what the screen answered (`scene` weighed by its `share`) and the distance along
// the view of what it met, none where it met nothing; whether the list took it.
fn request_water_trace(pixel: vec2<f32>, world: vec3<f32>, normal: vec3<f32>, toward: vec3<f32>, distance: f32,
  answer: vec4<f32>, share: f32) -> bool {
  let pixel_angle: f32 = 2.0 / (camera.projection[1][1] * camera.viewport.y);
  var request: TraceRequest;

  request.origin = vec4<f32>(world + normal * TRACED_LIFT, enhanced.trace_distance);
  request.cone = vec2<f32>(distance * pixel_angle, pixel_angle);
  request.toward = pack2x16snorm(octahedral_encode(toward));
  request.pixel = u32(pixel.x) | (u32(pixel.y) << 16u);
  request.screen = trace_request_screen(answer.rgb, share);
  request.screen_length = answer.w;
  request.hit_end = TRACE_REQUEST_WATER | (pack2x16float(vec2<f32>(water.reflection, 0.0)) & 0xffffu);

  return list_trace_request(request);
}

@vertex
fn vs_water(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> WaterVarying {
  return water_vertex(vertex_index, instance_index, water);
}

// A view space point's place on the screen, in texture coordinates.
fn view_to_uv(point: vec3<f32>) -> vec2<f32> {
  let clip: vec4<f32> = camera.projection * vec4<f32>(point, 1.0);

  return clip.xy / clip.w * vec2<f32>(0.5, -0.5) + 0.5;
}

// The reflection's ray as it marches the screen: where it starts and steps, how long it runs on the screen, and how far
// along the view its ends lie.
struct ReflectionRay {
  start: vec2<f32>,
  step: vec2<f32>,
  length: f32,
  near: f32,
  far: f32,
};

// How far the ray, at a place on the screen, lies past the surface there, and that surface's
// distance.
fn ray_behind(ray: ReflectionRay, at: vec2<f32>) -> vec2<f32> {
  let share: f32 = length(at - ray.start) / max(ray.length, 1e-6);
  let along: f32 = ray.near * ray.far / mix(ray.far, ray.near, share);
  let scene: f32 = scene_distance(at, depth_target);

  return vec2<f32>(along - scene, scene);
}

// The view space point at a place on the screen, a distance along the view: through the whole projection, its
// off-centre terms the temporal jitter moves included, as `view_to_uv` projects back.
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

// The ray from a point of the water along its reflection, marched over the screen and refined
// where it passes behind a surface `thickness` thick at most; where it hit, then that surface's distance, then one
// where it crossed the surface or nought where it ran out of steps short of one and takes the last it passed in front
// of; or nothing where it left the screen. A surface nearer than 1.3 m, where a game's weapon would stand, is passed
// through.
fn march_reflection(start: vec3<f32>, direction: vec3<f32>, noise: f32, uv: vec2<f32>, thickness: f32) -> vec4<f32> {
  // A ray turning towards the eye stops short of the near plane rather than wrap behind it.
  let reach: f32 = select(MARCH_REACH, min(MARCH_REACH, (-0.1 - start.z) / direction.z), direction.z > 0.0);
  let end: vec3<f32> = start + direction * max(reach, 0.0);
  let screen_start: vec2<f32> = view_to_uv(start);
  let screen_end: vec2<f32> = view_to_uv(end);
  var ray: ReflectionRay;

  ray.start = screen_start;
  ray.step = (screen_end - screen_start) / f32(enhanced.march_steps);
  ray.length = length(screen_end - screen_start);
  ray.near = -start.z;
  ray.far = -end.z;

  // Squeezed across near the screen's sides, unless the eye looks down.
  let edges: vec2<f32> = 1.0 - smoothstep(vec2<f32>(0.9), vec2<f32>(1.0), vec2<f32>(uv.x, 1.0 - uv.x));
  let looking_down: f32 = saturate(-camera.view[1].z * 3.0);

  ray.step.x *= saturate(edges.x * edges.y + looking_down);

  var at: vec2<f32> = ray.start + ray.step * noise;
  let start_distance: f32 = scene_distance(ray.start, depth_target);
  var behind: vec3<f32> = vec3<f32>(0.0);

  for (var step: i32 = 1; step <= i32(enhanced.march_steps); step++) {
    if (any(at < vec2<f32>(0.0)) || any(at > vec2<f32>(1.0))) {
      return vec4<f32>(0.0);
    }

    var check: vec2<f32> = ray_behind(ray, at);
    let is_far: bool = check.y > 1.3;

    check.x *= f32(is_far);

    if (check.x > 0.0) {
      if (check.x <= thickness || start_distance + 40.0 < check.y) {
        return vec4<f32>(at, check.y, 1.0);
      }

      let kept: vec2<f32> = at;
      let kept_step: vec2<f32> = ray.step;
      var last_sign: f32 = -1.0;

      for (var refine: i32 = 0; refine < MARCH_REFINES; refine++) {
        if (sign(check.x) != last_sign) {
          ray.step *= -0.5;
          last_sign = sign(check.x);
        }

        at += ray.step;
        check = ray_behind(ray, at);

        if (abs(check.x) <= thickness) {
          return vec4<f32>(at, check.y, 1.0);
        }
      }

      at = kept;
      ray.step = kept_step;
    } else {
      behind = vec3<f32>(at, check.y) * f32(start_distance - 2.0 < check.y && is_far);
    }

    let is_passed: bool = !is_far && check.y > 0.01 && f32(step) > f32(enhanced.march_steps) * 0.4;

    at += ray.step * select(1.0, 3.5, is_passed);
  }

  return vec4<f32>(behind, 0.0);
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
  let noise_uv: vec2<f32> = (uv * 1.33 + water.time * 0.02) * vec2<f32>(size.x / size.y, 1.0);
  let noise: f32 = textureSampleLevel(blue_noise, texture_sampler, noise_uv, 0.0).x * 1.5;
  // A branch rather than a select, which would march every ray to throw most away.
  let is_traced: bool = enhanced.trace_distance > 0.0;
  var hit: vec4<f32> = vec4<f32>(0.0);

  if (is_away) {
    hit = march_reflection(position, reflected, noise, uv, MARCH_THICKNESS);
  }
  let sky: vec3<f32> = enhanced_sky(flat_reflected, lighting, sky_cube_0, sky_cube_1, sky_clamp) * water.reflection;
  // The screen answers first, as it does untraced: the trace takes only what it misses (off the screen, the sky, past
  // its reach), and a surface the screen met facing away from the ray, which the mirror cannot see.
  let is_met: bool = all(hit.xy != vec2<f32>(0.0)) && !(is_traced && is_away && is_facing_away(hit.xy, reflected));
  let screen_share: f32 = select(0.0, saturate(hit.y * 5.0) * away, is_met);
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
  // history.
  if (screen_share < 1.0 && is_traced) {
    let pixel: vec2<f32> = floor(in.clip.xy);
    let answer: vec4<f32> = vec4<f32>(scene, met_distance);

    if (!(is_water_trace_turn(pixel, water_trace_turns(in.world, normal, trace_request_wait())) &&
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
  let jitter: vec2<f32> = (hash22(uv * 100.0 + water.time * 100.0) * 2.0 - 1.0) / size * 0.25;
  let sampled: vec3<f32> = textureSampleLevel(reflection_history, sky_clamp, previous + jitter, 0.0).rgb;
  // The history held within what this frame's fresh pixels about it show, so a reflection that moved onto a pixel,
  // which its history never had, stands at once rather than when its turn comes.
  let bounds: FreshBounds = fresh_bounds(vec2<i32>(in.clip.xy));
  let history: vec3<f32> = select(sampled, clamp(sampled, bounds.low, bounds.high), bounds.is_found);
  // The history's distance says whether the last frame drew water there: newly seen water keeps none of it.
  let is_history: f32 = f32(history_distance(previous) > 0.0);
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
