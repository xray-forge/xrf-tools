#import "common/camera"

// Screen-space reflection rays, for any surface that reflects: a ray from a view space point marched in even steps
// across the screen towards where it ends, the depth it should have at each step against the frame's, and how much of
// a surface's colour its reflection takes. Rays leaving the screen sideways are turned back from its edge; the depth is
// passed in, so each pass binds it where it likes.

// How far along the view the frame's surface may lie and still be one a ray meets: nearer is the camera's own.
const RAY_NEAREST: f32 = 1.3;
// How far along the view a ray's depth may lie from the frame's after a halving and still have met it.
const RAY_REFINED: f32 = 1.25;
// How much nearer than where a ray left the last surface it passed in front of may lie and still stand for a hit.
const RAY_PASSED: f32 = 2.0;
// How much of a reflection grass and leaves take.
const FLORA_SHARE: f32 = 0.33;

// A ray across the screen: where it is, its step, where it left and how far it goes there, in texture coordinates; and
// its distances along the view where it leaves and ends.
struct ReflectionRay {
  position: vec2<f32>,
  step: vec2<f32>,
  start: vec2<f32>,
  length: f32,
  depth_start: f32,
  depth_end: f32,
};

// Where a ray ended: where on the screen it met something (none where it met nothing), how far along the view that
// lies, and the height on the screen where it last saw the sky.
struct ReflectionHit {
  position: vec2<f32>,
  depth: f32,
  sky: f32,
};

// A view space point's place on the screen, in texture coordinates.
fn reflection_screen(point: vec3<f32>) -> vec2<f32> {
  let clip: vec4<f32> = camera.projection * vec4<f32>(point, 1.0);

  return (clip.xy / clip.w) * vec2<f32>(0.5, -0.5) + 0.5;
}

// How far along the view the frame's surface lies at a place on the screen, nothing where only the sky is.
fn reflection_scene_depth(depth_target: texture_depth_2d, uv: vec2<f32>) -> f32 {
  let size: vec2<f32> = camera.viewport.xy;
  let stored: f32 = textureLoad(depth_target, vec2<i32>(clamp(floor(uv * size), vec2<f32>(0.0), size - 1.0)), 0);

  return select(camera.projection[3][2] / (stored + camera.projection[2][2]), 0.0, stored <= 0.0);
}

// A ray from a view space point along a direction for at most a distance, cut short of the near plane, in `steps`
// steps, its first at `offset` steps out.
fn reflection_ray(start: vec3<f32>, direction: vec3<f32>, distance: f32, steps: u32, offset: f32) -> ReflectionRay {
  let near: f32 = camera.projection[3][2] / (1.0 + camera.projection[2][2]);
  let towards: f32 = max(direction.z, 0.0);
  let reach: f32 = select(distance, min(distance, (-start.z - near * 1.05) / towards), towards > 1e-5);
  let end: vec3<f32> = start + direction * max(reach, 0.0);
  let origin: vec2<f32> = reflection_screen(start);
  let span: vec2<f32> = reflection_screen(end) - origin;
  let step: vec2<f32> = span / f32(steps);

  return ReflectionRay(origin + step * offset, step, origin, length(span), -start.z, -end.z);
}

// How far behind the frame's surface the ray's depth lies where it stands, then that surface's depth.
fn reflection_intersect(depth_target: texture_depth_2d, ray: ReflectionRay) -> vec2<f32> {
  let along: f32 = length(ray.position - ray.start) / max(ray.length, 1e-6);
  let ray_depth: f32 = (ray.depth_start * ray.depth_end) / mix(ray.depth_end, ray.depth_start, along);
  let scene: f32 = reflection_scene_depth(depth_target, ray.position);

  return vec2<f32>(ray_depth - scene, scene);
}

// A ray marched until it lies behind a surface by no more than `limit`, once halved back where `is_refined` asks;
// where it never does, the last surface it passed in front of no nearer than where it left, or nothing.
fn reflection_march(depth_target: texture_depth_2d, start: vec3<f32>, direction: vec3<f32>, distance: f32,
  steps: u32, limit: f32, is_refined: bool) -> ReflectionHit {
  var ray: ReflectionRay = reflection_ray(start, direction, distance, steps, 2.0);
  let depth_start: f32 = reflection_scene_depth(depth_target, ray.start);
  var check: vec2<f32> = vec2<f32>(0.0);
  var passed: vec2<f32> = vec2<f32>(0.0);
  var sky: f32 = 0.0;

  for (var index: u32 = 0u; index < steps; index++) {
    if (ray.position.y < 0.0 || ray.position.y > 1.0) {
      return ReflectionHit(vec2<f32>(0.0), 0.0, 0.0);
    }

    // Off the screen sideways, the ray turns back from its edge.
    if (ray.position.x < 0.0 || ray.position.x > 1.0) {
      ray.position -= ray.step;
      ray.step.x = -ray.step.x;
      ray.position += ray.step;
    }

    check = reflection_intersect(depth_target, ray);

    let is_surface: bool = check.y > RAY_NEAREST;
    let behind: f32 = select(0.0, check.x, is_surface);

    if (behind > 0.0) {
      if (behind <= limit) {
        return ReflectionHit(ray.position, check.y, 0.0);
      }

      if (is_refined) {
        let held: vec4<f32> = vec4<f32>(ray.position, ray.step);

        ray.step *= -0.5;
        ray.position += ray.step;

        let refined: vec2<f32> = reflection_intersect(depth_target, ray);

        if (abs(refined.x) <= RAY_REFINED) {
          return ReflectionHit(ray.position, refined.y, 0.0);
        }

        ray.position = held.xy;
        ray.step = held.zw;
      }
    } else {
      if (check.y <= 0.0) {
        sky = ray.position.y;
      }

      passed = select(vec2<f32>(0.0), ray.position, depth_start - RAY_PASSED < check.y && is_surface);
    }

    ray.position += ray.step;
  }

  return ReflectionHit(passed, check.y, sky);
}

// How much of a surface's colour its reflection takes, seen along `to_point`: its gloss by a cubed Fresnel of how far
// the reflection turns from the view, a third of it for grass and leaves, by `intensity`, at most all of it; then
// faded by the square of what the fog leaves. Directions in any one space.
fn reflection_share(gloss: f32, normal: vec3<f32>, to_point: vec3<f32>, intensity: f32, is_flora: bool, fog: f32)
  -> f32 {
  let fresnel: f32 = saturate(dot(reflect(to_point, normal), to_point));
  let share: f32 = saturate(gloss * fresnel * fresnel * fresnel * select(1.0, FLORA_SHARE, is_flora) * intensity);
  let clear: f32 = 1.0 - fog;

  return share * clear * clear;
}
