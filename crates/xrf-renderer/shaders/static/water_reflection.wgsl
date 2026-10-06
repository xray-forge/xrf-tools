#import "common/water_surface"
#import "common/water_enhanced"
#import "generated/static/water_reflection"

// The reflection's march: its steps, its refinements of a hit, how thick
// a surface it takes for one, and how far it reaches.
const MARCH_STEPS: i32 = 16;
const MARCH_REFINES: i32 = 2;
const MARCH_THICKNESS: f32 = 3.0;
const MARCH_REACH: f32 = 150.0;

// The flat surface's fresnel, cubed and by the reflectivity, below which the surface would show too little of a hit to
// march for it: looking that steeply down, the waves' normals cannot raise it into sight.
const MARCH_LEAST_SHOWN: f32 = 0.002;

// How much of the reflection the last frames keep, and how much less of it where the ray met the sky.
const REFLECTION_HISTORY: f32 = 0.97;
const REFLECTION_SKY_HISTORY: f32 = 0.2;

// Pixels of motion over which the history gives way entirely: a fast turn draws the reflection afresh rather than drag
// the last frames behind it.
const HISTORY_MOTION: f32 = 40.0;

@vertex
fn vs_water(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> WaterVarying {
  return water_vertex(vertex_index, instance_index, water);
}

// Two values in nothing to one, from a point: the jitter the reflection reads its history with.
fn hash22(point: vec2<f32>) -> vec2<f32> {
  var p: vec3<f32> = fract(vec3<f32>(point.xyx) * vec3<f32>(0.1031, 0.103, 0.0973));

  p += dot(p, p.yzx + 33.33);

  return fract((p.xx + p.yz) * p.zy);
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

// The ray from a point of the water along its reflection, marched over the screen and refined
// where it passes behind a surface; where it hit, then that surface's distance, or nothing where it left the screen.
// A surface nearer than 1.3 m, where a game's weapon would stand, is passed through.
fn march_reflection(start: vec3<f32>, direction: vec3<f32>, noise: f32, uv: vec2<f32>) -> vec3<f32> {
  // A ray turning towards the eye stops short of the near plane rather than wrap behind it.
  let reach: f32 = select(MARCH_REACH, min(MARCH_REACH, (-0.1 - start.z) / direction.z), direction.z > 0.0);
  let end: vec3<f32> = start + direction * max(reach, 0.0);
  let screen_start: vec2<f32> = view_to_uv(start);
  let screen_end: vec2<f32> = view_to_uv(end);
  var ray: ReflectionRay;

  ray.start = screen_start;
  ray.step = (screen_end - screen_start) / f32(MARCH_STEPS);
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

  for (var step: i32 = 1; step <= MARCH_STEPS; step++) {
    if (any(at < vec2<f32>(0.0)) || any(at > vec2<f32>(1.0))) {
      return vec3<f32>(0.0);
    }

    var check: vec2<f32> = ray_behind(ray, at);
    let is_far: bool = check.y > 1.3;

    check.x *= f32(is_far);

    if (check.x > 0.0) {
      if (check.x <= MARCH_THICKNESS || start_distance + 40.0 < check.y) {
        return vec3<f32>(at, check.y);
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

        if (abs(check.x) <= MARCH_THICKNESS) {
          return vec3<f32>(at, check.y);
        }
      }

      at = kept;
      ray.step = kept_step;
    } else {
      behind = vec3<f32>(at, check.y) * f32(start_distance - 2.0 < check.y && is_far);
    }

    let is_passed: bool = !is_far && check.y > 0.01 && f32(step) > f32(MARCH_STEPS) * 0.4;

    at += ray.step * select(1.0, 3.5, is_passed);
  }

  return behind;
}

// The scene the water's flat surface reflects, marched over the screen as it
// stood before the water, faded towards the top of the screen and into the fog, the sky where the ray met nothing; then
// kept over the frames before, read where this one's point stood in the last.
@fragment
fn fs_water_reflection(in: WaterVarying) -> @location(0) vec4<f32> {
  let position: vec3<f32> = (camera.view * vec4<f32>(in.world, 1.0)).xyz;
  let fog: f32 = select(0.0, fog_amount(lighting, position), lighting.params.y > 0.5);

  if (fog >= 1.0 || in.clip.z < textureLoad(nearest_water, vec2<i32>(in.clip.xy), 0)) {
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
  // Rays towards the eye are not traced: looking down they only mess the reflection. Nor are those the surface would
  // show too little of.
  let shown: f32 = pow(saturate(dot(flat_reflected, to_point)), 3.0) * enhanced.reflectivity;
  let is_away: bool = dot(-eye, reflected) <= -0.3 && shown >= MARCH_LEAST_SHOWN;
  let noise_uv: vec2<f32> = (uv * 1.33 + water.time * 0.02) * vec2<f32>(size.x / size.y, 1.0);
  let noise: f32 = textureSampleLevel(blue_noise, texture_sampler, noise_uv, 0.0).x * 1.5;
  // A branch rather than a select, which would march every ray to throw most away.
  var hit: vec3<f32> = vec3<f32>(0.0);

  if (is_away) {
    hit = march_reflection(position, reflected, noise, uv);
  }
  let sky: vec3<f32> = enhanced_sky(flat_reflected, lighting, sky_cube_0, sky_cube_1, sky_clamp) * water.reflection;
  var reflection: vec3<f32> = sky;

  if (all(hit.xy != vec2<f32>(0.0))) {
    let scene: vec3<f32> = textureLoad(water_scene, vec2<i32>(clamp(floor(hit.xy * size), vec2<f32>(0.0), size - 1.0)), 0)
      .rgb;
    let fogged: f32 = saturate((length(vec3<f32>(position.xy, hit.z)) * lighting.fog.y + lighting.fog.x) * 1.4);

    reflection = mix(sky, scene, saturate(hit.y * 5.0 * f32(is_away) * (1.0 - fogged)));
  }

  // The point the frames before are read at: along this pixel's view as far as what the ray met, or far off at the sky.
  let hit_distance: f32 = select(hit.z, 1e5, hit.z <= 0.0);
  let along: vec3<f32> = eye * (hit_distance / max(-eye.z, 1e-4));
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
  let world_hit: vec4<f32> = vec4<f32>(camera.position.xyz + rotation * along, 1.0);
  let now: vec4<f32> = camera.motion_current * world_hit;
  let before: vec4<f32> = camera.motion_previous * world_hit;
  let moved: vec2<f32> = (now.xy / now.w - before.xy / before.w) * vec2<f32>(0.5, -0.5);
  let previous: vec2<f32> = uv - moved;
  let is_off: bool = any(previous < vec2<f32>(0.0)) || any(previous > vec2<f32>(1.0));
  let kept: f32 = saturate(REFLECTION_HISTORY - select(0.0, REFLECTION_SKY_HISTORY, hit.z <= 0.0) - f32(is_off)) *
    enhanced.history;
  let jitter: vec2<f32> = (hash22(uv * 100.0 + water.time * 100.0) * 2.0 - 1.0) / size * 0.25;
  // The history's alpha says whether the last frame drew water there: newly seen water keeps none of it.
  let history: vec4<f32> = textureSampleLevel(reflection_history, sky_clamp, previous + jitter, 0.0);
  let settled: f32 = saturate(1.0 - length(moved * size) / HISTORY_MOTION);

  return vec4<f32>(mix(reflection, history.rgb, kept * history.a * settled), 1.0);
}
