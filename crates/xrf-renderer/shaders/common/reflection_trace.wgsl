#import "common/camera"
#import "common/octahedral"

// Screen-space reflection rays, for any surface that reflects: a ray's way from its surface's normal and roughness,
// marched over the frame's nearest depth at the size it is traced at, and how far a hit can be trusted. The ray runs on
// the screen in texture coordinates and stored depth, both of which a straight line in view space keeps straight
// (McGuire and Mara, "Efficient GPU Screen-Space Ray Tracing", JCGT 2014), so a step anywhere along it reads its depth
// exactly. Steps crowd towards where the ray leaves and spread towards its end, each a little off by noise, and a ray
// that passes behind something deeper than the thickness marches on, so a fence or a post does not end it. The depth
// and the frame's targets are passed in, so each pass binds them where it likes.

// The share of the near plane's distance a ray towards the camera may come.
const TRACE_NEAR_MARGIN: f32 = 1.05;
// Steps along a ray go as this power of an even spread: dense where it leaves, where most hits are.
const TRACE_STEP_POWER: f32 = 1.8;
// Halvings that refine a hit between the step in front of the surface and the one behind it.
const TRACE_REFINEMENTS: u32 = 5u;
// Texels of the traced depth a hit must lie from where its ray left, or the ray met its own surface.
const TRACE_SELF_TEXELS: f32 = 1.5;
// The share of the screen's height over which a hit fades out towards its edges.
const TRACE_EDGE_FADE: f32 = 0.06;
// The share of a ray's length over which a hit fades out towards its end.
const TRACE_END_FADE: f32 = 0.25;
// How far along the ray a surface met may face before it fades out as its back, and where it is gone: generous,
// since the normal is bumped, and a ray grazing a crest within the thickness meets what the crest shows.
const TRACE_BACK_FACE: vec2<f32> = vec2<f32>(0.0, 0.2);

// A ray in view space: where it leaves, the way it goes, and the metres it is traced at most.
struct ReflectionRay {
  origin: vec3<f32>,
  direction: vec3<f32>,
  length: f32,
};

// What tracing a ray met: where it stopped on the screen, as texture coordinates and stored depth, the share of its
// length it went, and whether it went behind a surface there by no more than the thickness.
struct ReflectionHit {
  position: vec3<f32>,
  along: f32,
  is_hit: bool,
};

// How a ray is marched: the traced depth's texels across the viewport (the viewport's size over the frame's pixels a
// texel stands for), the steps it takes, the metres behind a surface it may pass and still have met it (and what that
// grows by each metre from the camera), and the noise from zero to one its steps are moved by.
struct ReflectionMarch {
  base: vec2<f32>,
  steps: u32,
  thickness: f32,
  thickness_growth: f32,
  noise: f32,
};

// A view space point on the screen: texture coordinates, `y` down, and stored depth, reversed.
fn reflection_screen(position: vec3<f32>) -> vec3<f32> {
  let clip: vec4<f32> = camera.projection * vec4<f32>(position, 1.0);
  let ndc: vec3<f32> = clip.xyz / clip.w;

  return vec3<f32>(ndc.xy * vec2<f32>(0.5, -0.5) + 0.5, ndc.z);
}

// The distance along the view of what a stored depth shows, from the perspective's depth terms.
fn reflection_distance(depth: f32) -> f32 {
  return camera.projection[3][2] / (depth + camera.projection[2][2]);
}

// The near plane's distance: where the reversed depth reads one.
fn reflection_near() -> f32 {
  return reflection_distance(1.0);
}

// A way about the mirror direction off a view space `normal`, seen along `to_point`, spread by `roughness` (a Phong
// lobe of exponent `2 / roughness² - 2`) and picked by two noises from zero to one; the mirror direction where the lobe
// is narrower than it matters or the way picked dips under the surface.
fn reflection_lobe(normal: vec3<f32>, to_point: vec3<f32>, roughness: f32, noise: vec2<f32>) -> vec3<f32> {
  let mirror: vec3<f32> = reflect(to_point, normal);

  if (roughness < 0.01) {
    return mirror;
  }

  let exponent: f32 = 2.0 / (roughness * roughness) - 2.0;
  let cos_theta: f32 = pow(max(noise.x, 1e-6), 1.0 / (exponent + 1.0));
  let sin_theta: f32 = sqrt(max(0.0, 1.0 - cos_theta * cos_theta));
  let phi: f32 = noise.y * 6.2831853;
  let up: vec3<f32> = select(vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(1.0, 0.0, 0.0), abs(mirror.z) > 0.9);
  let across: vec3<f32> = normalize(cross(up, mirror));
  let along: vec3<f32> = cross(mirror, across);
  let tilted: vec3<f32> = across * (cos(phi) * sin_theta) + along * (sin(phi) * sin_theta);
  let picked: vec3<f32> = normalize(tilted + mirror * cos_theta);

  return select(mirror, picked, dot(picked, normal) > 0.0);
}

// A ray's length held short of the near plane, so a ray towards the camera never projects from behind it.
fn reflection_length(ray: ReflectionRay) -> f32 {
  if (ray.direction.z <= 0.0) {
    return ray.length;
  }

  let limit: f32 = -reflection_near() * TRACE_NEAR_MARGIN;

  return max(0.0, min(ray.length, (limit - ray.origin.z) / ray.direction.z));
}

// The share of a screen ray from `origin` along `direction` before it leaves the screen, at most all of it.
fn reflection_on_screen(origin: vec2<f32>, direction: vec2<f32>) -> f32 {
  let bound: vec2<f32> = select(vec2<f32>(0.0), vec2<f32>(1.0), direction > vec2<f32>(0.0));
  let leaving: vec2<f32> = select(vec2<f32>(1e9), (bound - origin) / direction, abs(direction) > vec2<f32>(1e-9));

  return clamp(min(leaving.x, leaving.y), 0.0, 1.0);
}

// How far behind what `nearest` shows under a screen point the ray stands there, in metres along the view: below none
// in front of it.
fn reflection_behind(nearest: texture_2d<f32>, march: ReflectionMarch, point: vec3<f32>) -> f32 {
  let last: vec2<i32> = vec2<i32>(textureDimensions(nearest)) - 1;
  let surface: f32 = textureLoad(nearest, clamp(vec2<i32>(point.xy * march.base), vec2<i32>(0), last), 0).x;

  if (surface <= 0.0) {
    return -1e9;
  }

  return reflection_distance(point.z) - reflection_distance(surface);
}

// Marches a ray over `nearest`, the frame's nearest depth at the size it is traced at (with reversed depth the
// largest), until a step stands behind a surface by no more than the thickness, then halves the last step down to
// where it went behind; none where it leaves the screen or its length first.
fn reflection_march(nearest: texture_2d<f32>, march: ReflectionMarch, ray: ReflectionRay) -> ReflectionHit {
  let reach: f32 = reflection_length(ray);
  let origin: vec3<f32> = reflection_screen(ray.origin);
  let direction: vec3<f32> = reflection_screen(ray.origin + ray.direction * reach) - origin;
  let missed: ReflectionHit = ReflectionHit(origin, 1.0, false);

  if (reach <= 0.0 || all(abs(direction.xy) < vec2<f32>(1e-7))) {
    return missed;
  }

  let end: f32 = reflection_on_screen(origin.xy, direction.xy);
  // The share of the ray a texel of the traced depth is: no two steps lie closer, whatever the noise.
  let first: f32 = 1.0 / max(length(direction.xy * march.base), 1.0);
  var before: f32 = 0.0;

  for (var step: u32 = 0u; step < march.steps; step++) {
    let spread: f32 = pow((f32(step) + march.noise) / f32(march.steps), TRACE_STEP_POWER);
    let t: f32 = max(spread * end, first * f32(step + 1u));

    if (t > end) {
      break;
    }

    let point: vec3<f32> = origin + direction * t;
    let behind: f32 = reflection_behind(nearest, march, point);
    let thickness: f32 = march.thickness + march.thickness_growth * reflection_distance(point.z);

    // Within a texel or so of where it left, what the ray stands behind is its own surface.
    let is_clear: bool = t * length(direction.xy * march.base) >= TRACE_SELF_TEXELS;

    if (is_clear && behind >= 0.0 && behind <= thickness) {
      var low: f32 = before;
      var high: f32 = t;

      for (var refinement: u32 = 0u; refinement < TRACE_REFINEMENTS; refinement++) {
        let middle: f32 = 0.5 * (low + high);

        if (reflection_behind(nearest, march, origin + direction * middle) >= 0.0) {
          high = middle;
        } else {
          low = middle;
        }
      }

      return ReflectionHit(origin + direction * high, high, true);
    }

    before = t;
  }

  return ReflectionHit(origin + direction * end, end, false);
}

// How far a hit can be trusted, from none to one: none off the screen, on the sky, on the ray's own surface or on the
// back of what it met; fading as the ray stands deeper behind what `nearest` shows, out to the thickness, and towards
// the screen's edges and the ray's end. `left` is where the ray left on the screen, in texture coordinates.
fn reflection_confidence(nearest: texture_2d<f32>, depth_target: texture_depth_2d, normal_target: texture_2d<f32>,
  hit: ReflectionHit, ray: ReflectionRay, left: vec2<f32>, march: ReflectionMarch) -> f32 {
  if (!hit.is_hit) {
    return 0.0;
  }

  let uv: vec2<f32> = hit.position.xy;
  let travelled: vec2<f32> = abs(uv - left) * march.base;

  if (all(travelled < vec2<f32>(TRACE_SELF_TEXELS))) {
    return 0.0;
  }

  let size: vec2<f32> = camera.viewport.xy;
  let texel: vec2<i32> = clamp(vec2<i32>(uv * size), vec2<i32>(0), vec2<i32>(size) - 1);
  let stored: f32 = textureLoad(depth_target, texel, 0);

  if (stored <= 0.0) {
    return 0.0;
  }

  let met: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let facing: f32 = 1.0 - smoothstep(TRACE_BACK_FACE.x, TRACE_BACK_FACE.y, dot(met, ray.direction));
  let reached: f32 = reflection_distance(hit.position.z);
  let thickness: f32 = march.thickness + march.thickness_growth * reached;
  let behind: f32 = 1.0 - smoothstep(0.0, thickness, abs(reflection_behind(nearest, march, hit.position)));
  let edge: vec2<f32> = vec2<f32>(TRACE_EDGE_FADE * size.y / size.x, TRACE_EDGE_FADE);
  let border: vec2<f32> = smoothstep(vec2<f32>(0.0), edge, uv) * (1.0 - smoothstep(1.0 - edge, vec2<f32>(1.0), uv));
  let end: f32 = 1.0 - smoothstep(1.0 - TRACE_END_FADE, 1.0, hit.along);

  return facing * behind * border.x * border.y * end;
}
