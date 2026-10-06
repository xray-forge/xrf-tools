#import "common/camera"

// Screen-space reflection rays: a reflected direction drawn from a surface's GGX lobe, walked across the screen up and
// down a pyramid of the frame's nearest depth, skipping whole cells it passes over and stepping down where it meets a
// surface, until it stands on one at the finest level; and how much of what a surface reflects a traced reflection
// shows, of its own reflection's slice and as a puddle's clear coat of water, which combine blends and the trace gates
// by alike. The pyramid is passed in, so each pass binds it where it likes.

// Water's reflectance head on, from its index of refraction of 1.33, and a dry dielectric's.
const COAT_F0: f32 = 0.02;
const DIELECTRIC_F0: f32 = 0.04;
// The least of a puddle a point is for its water to be traced as a mirror.
const PUDDLE_LEAST: f32 = 0.01;
// The pyramid's coarsest level a ray climbs to.
const RAY_TOP_LEVEL: i32 = 6;
const RAY_FAR: f32 = 3.402823466e+38;
const PI: f32 = 3.14159265358979;

// A point on the screen: texture coordinates, then the stored depth, reversed.
fn ray_screen_point(view: vec3<f32>) -> vec3<f32> {
  let clip: vec4<f32> = camera.projection * vec4<f32>(view, 1.0);
  let ndc: vec3<f32> = clip.xyz / clip.w;

  return vec3<f32>(ndc.xy * vec2<f32>(0.5, -0.5) + 0.5, ndc.z);
}

// The view space point at a screen point.
fn ray_view_point(screen: vec3<f32>) -> vec3<f32> {
  return camera_view_position(screen.xy * camera.viewport.xy, max(screen.z, 1e-7));
}

// A microfacet normal drawn from the GGX distribution of visible normals (Heitz 2018) about +z, seen from `view`, with a
// roughness `alpha` and two uniform numbers.
fn ggx_visible_normal(view: vec3<f32>, alpha: f32, numbers: vec2<f32>) -> vec3<f32> {
  let stretched: vec3<f32> = normalize(vec3<f32>(alpha * view.x, alpha * view.y, view.z));
  let length_squared: f32 = dot(stretched.xy, stretched.xy);
  let first: vec3<f32> = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(-stretched.y, stretched.x, 0.0) *
    inverseSqrt(max(length_squared, 1e-12)), length_squared > 0.0);
  let second: vec3<f32> = cross(stretched, first);
  let radius: f32 = sqrt(numbers.x);
  let angle: f32 = 2.0 * PI * numbers.y;
  let across: f32 = radius * cos(angle);
  let mixed: f32 = 0.5 * (1.0 + stretched.z);
  let along: f32 = (1.0 - mixed) * sqrt(1.0 - across * across) + mixed * radius * sin(angle);
  let normal: vec3<f32> = across * first + along * second +
    sqrt(max(0.0, 1.0 - across * across - along * along)) * stretched;

  return normalize(vec3<f32>(alpha * normal.x, alpha * normal.y, max(0.0, normal.z)));
}

// The direction a ray from a surface leaves in, in view space: the view's reflection about a microfacet normal drawn
// from the surface's lobe, or about its own normal where it is a mirror.
fn reflected_direction(toward: vec3<f32>, normal: vec3<f32>, alpha: f32, numbers: vec2<f32>) -> vec3<f32> {
  if (alpha < 1e-4) {
    return reflect(toward, normal);
  }

  let tangent: vec3<f32> = select(
    normalize(vec3<f32>(normal.y, -normal.x, 0.0)),
    normalize(vec3<f32>(0.0, -normal.z, normal.y)),
    abs(normal.z) > 0.0,
  );
  let bitangent: vec3<f32> = cross(normal, tangent);
  let basis: mat3x3<f32> = mat3x3<f32>(tangent, bitangent, normal);
  // Into the surface's frame, the view towards the camera.
  let view: vec3<f32> = -toward * basis;
  let microfacet: vec3<f32> = ggx_visible_normal(view, alpha, numbers);

  return basis * reflect(-view, microfacet);
}

// Where a ray first crosses the edge of the cell it starts in at a level, so it does not meet its own surface.
fn ray_first_cell(origin: vec3<f32>, direction: vec3<f32>, inverse: vec3<f32>, cells: vec2<f32>, floor_offset: vec2<f32>,
  nudge: vec2<f32>) -> f32 {
  let plane: vec2<f32> = (floor(cells * origin.xy) + floor_offset) / cells + nudge;
  let along: vec2<f32> = plane * inverse.xy - origin.xy * inverse.xy;

  return min(along.x, along.y);
}

// The nearest depth stored at a cell of a level, none outside it.
fn ray_cell_depth(pyramid: texture_2d<f32>, cell: vec2<i32>, level: i32) -> f32 {
  let size: vec2<i32> = vec2<i32>(textureDimensions(pyramid, level));

  return textureLoad(pyramid, clamp(cell, vec2<i32>(0), size - 1), level).x;
}

// Where a ray from a screen point along a screen direction meets the frame's nearest depth, walked over at most
// `crossings` cells; its second part is whether it ended on a surface, never where it left the screen or ran out.
//
// A surface the ray lies behind by no more than `thickness` (metres, then a share of the distance) it met. Further
// behind, it met it only where it crossed into it from the pixel it left: in front of the surface there, and that
// surface running on into this one (their depths no further apart than the ray moved), as ground rising or a wall going
// up from its foot. Otherwise the surface stands nearer than the ray, unconnected to what the ray came over, a twig, a
// wire, a tree's crown before the sky: the ray passes behind it and walks on.
//
// Nothing under the plane it was reflected off stops it (`mirror`: that plane's view space normal and its own point's
// distance along it; `slack`: metres over it a surface may stand and still lie beneath it, a little for anything, more
// for what stands in a puddle, `wet`'s `g`, or a plant there): a mirror's ray cannot meet what lies beneath it, as the
// ground under a puddle's level water or a bush's foot in it, which the depth shows a little above the water.
struct RayWalk {
  position: vec3<f32>,
  is_met: bool,
};

fn ray_walk(pyramid: texture_2d<f32>, material: texture_2d<f32>, wet: texture_2d<f32>, origin: vec3<f32>,
  direction: vec3<f32>, crossings: u32, thickness: vec2<f32>, mirror: vec4<f32>, slack: vec2<f32>) -> RayWalk {
  let screen: vec2<f32> = camera.viewport.xy;
  let inverse: vec3<f32> = select(vec3<f32>(RAY_FAR), 1.0 / direction, direction != vec3<f32>(0.0));
  var level: i32 = 0;
  var cells: vec2<f32> = screen;
  // A hair past each cell's edge, so a step lands inside the next.
  let nudge: vec2<f32> = select(vec2<f32>(0.005), vec2<f32>(-0.005), direction.xy < vec2<f32>(0.0)) / screen;
  let floor_offset: vec2<f32> = select(vec2<f32>(1.0), vec2<f32>(0.0), direction.xy < vec2<f32>(0.0));
  var along: f32 = ray_first_cell(origin, direction, inverse, cells, floor_offset, nudge);
  var position: vec3<f32> = origin + along * direction;
  var count: u32 = 0u;
  // Ray parameter a frame pixel spans along the ray, for the pixel it came from.
  let pixel_along: f32 = 1.0 / max(max(abs(direction.x) * screen.x, abs(direction.y) * screen.y), 1e-6);

  while (count < crossings && level >= 0) {
    // Off the screen, or past the far plane, there is nothing left to meet.
    if (any(position.xy < vec2<f32>(0.0)) || any(position.xy > vec2<f32>(1.0)) || position.z <= 0.0) {
      return RayWalk(position, false);
    }

    let at: vec2<f32> = cells * position.xy;
    let surface: f32 = ray_cell_depth(pyramid, vec2<i32>(at), level);
    let plane: vec2<f32> = (floor(at) + floor_offset) / cells + nudge;
    var crossing: vec3<f32> = vec3<f32>(plane, surface) * inverse - origin * inverse;

    // Reversed depth: the surface's plane only stops a ray going away from the camera.
    crossing.z = select(RAY_FAR, crossing.z, direction.z < 0.0);

    let nearest: f32 = min(min(crossing.x, crossing.y), crossing.z);
    let is_above: bool = surface < position.z;
    let is_skipped: bool = bitcast<u32>(nearest) != bitcast<u32>(crossing.z) && is_above;

    // Behind a frame pixel's surface: met, beneath the mirror, or passed behind.
    if (!is_above && level == 0) {
      let texel: vec2<i32> = vec2<i32>(at);
      let shown: vec3<f32> = ray_view_point(vec3<f32>(position.xy, surface));
      let ray: vec3<f32> = ray_view_point(position);
      let reach: f32 = thickness.x - shown.z * thickness.y;
      let is_in_puddle: bool = textureLoad(wet, texel, 0).g > 0.5 ||
        has_mark(textureLoad(material, texel, 0).a, MARK_PLANT);
      let is_beneath: bool = dot(mirror.xyz, shown) < mirror.w + select(slack.x, slack.y, is_in_puddle);
      var is_passed: bool = is_beneath;

      if (!is_beneath && length(shown - ray) > reach) {
        let came: vec3<f32> = origin + max(along - pixel_along, 0.0) * direction;
        let came_ray: vec3<f32> = ray_view_point(came);
        let came_surface: vec3<f32> = ray_view_point(vec3<f32>(came.xy,
          ray_cell_depth(pyramid, vec2<i32>(came.xy * screen), 0)));
        let was_in_front: bool = -came_ray.z <= -came_surface.z + reach;
        let is_connected: bool = abs(came_surface.z - shown.z) <= abs(ray.z - came_ray.z) + reach;

        if (was_in_front && is_connected) {
          return RayWalk(vec3<f32>(position.xy, surface), true);
        }

        is_passed = true;
      }

      if (is_passed) {
        along = min(crossing.x, crossing.y);
        position = origin + along * direction;
        count++;

        continue;
      }
    }

    along = select(along, nearest, is_above);
    position = origin + along * direction;

    if (!(is_skipped && level >= RAY_TOP_LEVEL)) {
      level += select(-1, 1, is_skipped);
      cells *= select(2.0, 0.5, is_skipped);
    }

    count++;
  }

  return RayWalk(position, level < 0);
}

// The roughness over which a surface's own reflection is no longer sharpened by a traced one: rough wet ground keeps
// the engine's blurred sheen, steady as the camera moves, and only near-mirrors reflect what stands about them.
const SHARP_ROUGHNESS: vec2<f32> = vec2<f32>(0.06, 0.2);

// How much of a surface's own reflection a traced one sharpens, by its roughness.
fn sharpened(roughness: f32) -> f32 {
  return 1.0 - smoothstep(SHARP_ROUGHNESS.x, SHARP_ROUGHNESS.y, roughness);
}

// How a surface's lobe is traced: a mirror wherever a puddle stands, its water traced along one ray and kept unblurred
// by the denoiser, its rim fading by how much of it there is rather than roughening; its own roughness elsewhere.
fn traced_roughness(gloss: f32, puddle: f32) -> f32 {
  return select(reflection_roughness(gloss), 0.0, puddle > PUDDLE_LEAST);
}

// How much a dielectric reflects at a cosine of the view to its normal: Schlick's Fresnel from its reflectance head
// on, its grazing peak lowered by the roughness (Fdez-Agüera's), so a rough surface never turns to a mirror.
fn dielectric_fresnel(facing: f32, roughness: f32, head_on: f32) -> f32 {
  let away: f32 = 1.0 - saturate(facing);
  let away_squared: f32 = away * away;

  return head_on + (max(1.0 - roughness, head_on) - head_on) * away_squared * away_squared * away;
}

// How much a surface's own reflection a traced one replaces, of its weight: no more than a dielectric of its roughness
// reflects at that cosine of the view to its normal, by the intensity, faded out as it roughens and none under a puddle,
// whose coat reflects instead. The rest keeps the environment the surface reflects.
fn reflection_slice(weight: vec3<f32>, facing: f32, gloss: f32, puddle: f32, intensity: f32) -> vec3<f32> {
  let roughness: f32 = reflection_roughness(gloss);

  return min(weight, vec3<f32>(dielectric_fresnel(facing, roughness, DIELECTRIC_F0) * intensity)) *
    (1.0 - saturate(puddle)) * sharpened(roughness);
}

// How much of what a puddle's clear coat of water covers reflects, at a cosine of the view to its normal: the coat's
// coverage by water's Fresnel, a mirror's, by the intensity.
fn coat_share(coverage: f32, facing: f32, intensity: f32) -> f32 {
  return saturate(coverage * dielectric_fresnel(facing, 0.0, COAT_F0) * intensity);
}

// A surface's roughness for its reflection's lobe, from the engine's gloss: a puddle's 0.6 is nearly a mirror, rain's
// 0.2 a soft blur, a dry surface's 0.1 rough.
fn reflection_roughness(gloss: f32) -> f32 {
  let rough: f32 = 1.0 - sqrt(saturate(gloss));

  return rough * rough;
}
