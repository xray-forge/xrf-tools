#import "common/camera"
#import "common/reflection_end"

// Screen-space reflection rays: a reflected direction drawn from a surface's GGX lobe, walked across the screen up and
// down a pyramid of the frame's nearest depth, skipping whole cells it passes over and stepping down where it meets a
// surface, until it stands behind one texel's surface at the finest level, met within a thin thickness or uncertain
// beyond it; and the specular a surface shows of what it reflects on Enhanced, the split-sum environment BRDF over
// its reflectance head on and roughness, which combine weighs the reflection by and the trace gates by alike, with the
// prefiltered sky it reads where a ray meets nothing. The pyramid is passed in, so each pass binds it where it likes.

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

// Where a ray from a screen point along a screen direction ends against the frame's nearest depth, walked over at most
// `crossings` cells of the pyramid: the standard hierarchical traversal. At each step the ray stands in a cell of some
// level; in front of the cell's nearest depth it moves to the cell's far edge and climbs a level, so empty space is
// crossed in few steps; behind it, it descends a level without moving, until at the finest level it stands behind one
// texel's surface. There it has met the surface where it lies behind it by no more than `thickness` (metres, then a
// share of the distance); further behind, nothing on the screen can say what it met, a twig, a wire or a bush's crown
// standing nearer than what the ray went for, and the walk ends uncertain: the miss chain answers, never a guess from
// the pixels about it. Its second part is whether it met a surface, never where it left the screen, ran out of
// crossings or ended uncertain.
struct RayWalk {
  position: vec3<f32>,
  is_met: bool,
  // How it ended (`common/reflection_end`).
  end: u32,
  // Metres behind the surface it met the ray stood, none where it crossed into it.
  behind: f32,
};

fn ray_walk(pyramid: texture_2d<f32>, origin: vec3<f32>, direction: vec3<f32>, crossings: u32,
  thickness: vec2<f32>) -> RayWalk {
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
  // Whether the last step landed on a cell's depth plane: the ray crossed into the surface rather than entering the
  // cell already behind it.
  var was_depth_crossed: bool = false;

  while (count < crossings && level >= 0) {
    // Past its reach or the far plane, nothing is looked for; off the screen, there is nothing left to meet.
    if (along >= 1.0 || position.z <= 0.0) {
      return RayWalk(position, false, END_FAR, 0.0);
    }

    if (any(position.xy < vec2<f32>(0.0)) || any(position.xy > vec2<f32>(1.0))) {
      return RayWalk(position, false, END_OFF_SCREEN, 0.0);
    }

    let at: vec2<f32> = cells * position.xy;
    let surface: f32 = ray_cell_depth(pyramid, vec2<i32>(at), level);
    let is_above: bool = surface < position.z;

    if (!is_above) {
      // Behind the cell's nearest: at the finest level the ray stands behind one texel's surface, and the thickness
      // says whether it met it; above it, the cell holds nearer texels than the ray's, and only descending can tell.
      if (level == 0) {
        let shown: vec3<f32> = ray_view_point(vec3<f32>(position.xy, surface));
        let ray: vec3<f32> = ray_view_point(position);
        let behind: f32 = shown.z - ray.z;
        let reach: f32 = thickness.x - shown.z * thickness.y;

        if (behind <= reach) {
          return RayWalk(vec3<f32>(position.xy, surface), true, select(END_MET_THICK, END_MET_CROSSED, was_depth_crossed),
            max(behind, 0.0));
        }

        return RayWalk(position, false, END_UNCERTAIN, behind);
      }

      level -= 1;
      cells *= 2.0;
      count++;

      continue;
    }

    // In front of the cell's nearest: to its far edge, or onto its depth plane where the ray reaches that first (a
    // reversed depth: the plane stops only a ray going away from the camera), and up a level, since the cells about
    // this one are likelier empty too.
    let plane: vec2<f32> = (floor(at) + floor_offset) / cells + nudge;
    var crossing: vec3<f32> = vec3<f32>(plane, surface) * inverse - origin * inverse;

    crossing.z = select(RAY_FAR, crossing.z, direction.z < 0.0);

    let nearest: f32 = min(min(crossing.x, crossing.y), crossing.z);

    was_depth_crossed = bitcast<u32>(nearest) == bitcast<u32>(crossing.z);
    along = max(along, nearest);
    position = origin + along * direction;

    if (was_depth_crossed) {
      // Onto the depth plane: at the finest level that is the surface itself, met where the ray crossed into it.
      if (level == 0) {
        return RayWalk(vec3<f32>(position.xy, surface), true, END_MET_CROSSED, 0.0);
      }

      level -= 1;
      cells *= 2.0;
    } else if (level < RAY_TOP_LEVEL) {
      level += 1;
      cells *= 0.5;
    }

    count++;
  }

  return RayWalk(position, false, END_SPENT, 0.0);
}

// The gloss past which a surface's reflectance head on rises from a dielectric's towards its own colour: X-Ray's
// glossiest surfaces are its metals and glass, and its data holds no metalness.
const METAL_GLOSS: vec2<f32> = vec2<f32>(0.6, 0.9);
// The roughness past which a surface is not traced and reflects the prefiltered sky alone.
const TRACED_ROUGHNESS_LIMIT: f32 = 0.5;
// Mips of the prefiltered sky cube past its first, the last at roughness one.
const SKY_PREFILTER_MIPS: f32 = 5.0;

// How a surface's lobe is traced: a mirror wherever a puddle stands, its water traced along one ray and kept unblurred
// by the denoiser, its rim fading by how much of it there is rather than roughening; its own roughness elsewhere.
fn traced_roughness(gloss: f32, puddle: f32) -> f32 {
  return select(reflection_roughness(gloss), 0.0, puddle > PUDDLE_LEAST);
}

// A surface's reflectance head on: a dielectric's, lifted towards its own colour as its gloss passes the metals', and
// water's over the share of it a puddle's clear coat covers.
fn reflectance_head_on(albedo: vec3<f32>, gloss: f32, coat: f32) -> vec3<f32> {
  let metal: f32 = smoothstep(METAL_GLOSS.x, METAL_GLOSS.y, gloss);

  return mix(mix(vec3<f32>(DIELECTRIC_F0), albedo, metal), vec3<f32>(COAT_F0), saturate(coat));
}

// What share of the light arriving along a surface's reflection it sends to the eye, at a cosine of the view to its
// normal: the split-sum environment BRDF, Karis's fit of the DFG term over its reflectance head on and its roughness,
// which holds the Fresnel's rise at grazing angles and its fall with roughness both.
fn environment_brdf(head_on: vec3<f32>, roughness: f32, facing: f32) -> vec3<f32> {
  let r: vec4<f32> = roughness * vec4<f32>(-1.0, -0.0275, -0.572, 0.022) + vec4<f32>(1.0, 0.0425, 1.04, -0.04);
  let a004: f32 = min(r.x * r.x, exp2(-9.28 * saturate(facing))) * r.x + r.y;
  let ab: vec2<f32> = vec2<f32>(-1.04, 1.04) * a004 + r.zw;

  return head_on * ab.x + ab.y;
}

// How much of the sky a surface's reflection sees past what stands about it, from how much of the hemisphere lights the
// surface (its baked and its screen occlusion together) at its roughness and facing: Lagarde and de Rousiers', a rough
// surface's wide lobe occluded as its hemisphere is, a mirror's only where the view grazes.
fn specular_occlusion(facing: f32, occlusion: f32, roughness: f32) -> f32 {
  return saturate(pow(saturate(facing) + occlusion, exp2(-16.0 * roughness - 1.0)) - 1.0 + occlusion);
}

// The sky a reflection reads where it meets nothing, along a renderer-space direction at a roughness: the prefiltered
// cube at the mip the roughness names.
fn prefiltered_sky(cube: texture_cube<f32>, cube_sampler: sampler, direction: vec3<f32>, roughness: f32) -> vec3<f32> {
  return textureSampleLevel(cube, cube_sampler, direction, saturate(roughness) * SKY_PREFILTER_MIPS).rgb;
}

// A surface's roughness for its reflection's lobe, from the engine's gloss: a puddle's 0.6 is nearly a mirror, rain's
// 0.2 a soft blur, a dry surface's 0.1 rough.
fn reflection_roughness(gloss: f32) -> f32 {
  let rough: f32 = 1.0 - sqrt(saturate(gloss));

  return rough * rough;
}
