#import "common/camera"
#import "common/octahedral"
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
// The least of a puddle a point is for its water to be the surface a reflection leaves: level under its rings for the
// denoiser, its normal the trace's block's.
const PUDDLE_LEAST: f32 = 0.01;
// The pyramid's coarsest level a ray climbs to.
const RAY_TOP_LEVEL: i32 = 6;
// Finest cells a ray may stand behind a surface in a row and still be passing behind something thin: a wire or a
// lattice bar is a cell or two across at the traced size, a twig a few; a trunk or a wall is many.
const PASS_CELLS: u32 = 4u;
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
// from the surface's lobe at `alpha` (`lobe_alpha` of its roughness), or about its own normal where it is a mirror.
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
// share of the distance). Further behind, it has passed behind something nearer than what it went for: a wire, a
// lattice bar or a twig, which the ray goes on past, out from behind it within `PASS_CELLS` finest cells; or a wall,
// a hill or a bush's crown it stays behind, where nothing on the screen can say what it met, and the walk ends
// uncertain where it first went behind: the miss chain answers, never a guess from the pixels about it. Without the
// passing, every ray climbing to the sky over a power line ended uncertain at the wire and the cube answered for the
// whole puddle under the lines. Its second part is whether it met a surface, never where it left the screen, ran out
// of crossings or ended uncertain.
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
  // Finest cells the ray has stood behind a surface in a row, and where it first went behind.
  var passed: u32 = 0u;
  var uncertain: RayWalk = RayWalk(position, false, END_UNCERTAIN, 0.0);

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

        // Further behind: on past a thin surface to the cell's far edge, or uncertain where the ray stays behind.
        if (passed == 0u) {
          uncertain = RayWalk(position, false, END_UNCERTAIN, behind);
        }

        passed++;

        if (passed > PASS_CELLS) {
          return uncertain;
        }
      } else {
        level -= 1;
        cells *= 2.0;
        count++;

        continue;
      }
    } else {
      passed = 0u;
    }

    // In front of the cell's nearest, or passing behind a thin one: to its far edge, or onto its depth plane where the
    // ray in front reaches that first (a reversed depth: the plane stops only a ray going away from the camera), and
    // up a level, since the cells about this one are likelier empty too.
    let plane: vec2<f32> = (floor(at) + floor_offset) / cells + nudge;
    var crossing: vec3<f32> = vec3<f32>(plane, surface) * inverse - origin * inverse;

    crossing.z = select(RAY_FAR, crossing.z, direction.z < 0.0 && is_above);

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

// The gloss past which a surface its texture's material marks a metal rises from a dielectric's reflectance head on
// towards its own colour, and the share of metal in that material it takes to be marked one. X-Ray's data holds no
// metalness, only the material its `hmodel` blends between (the G-buffer's `material.z`: 2 to 3 Phong to Metal, 3 to 4
// Metal to Oren-Nayar), authored for that look, so both must agree: glass, screens and polished paint under Blinn-Phong
// never turn metal, a rough wall tagged a fifth metal stays a dielectric (Skadovsk's television glass at gloss 0.64
// reflected as a metal and read as wet on gloss alone).
const METAL_GLOSS: vec2<f32> = vec2<f32>(0.6, 0.9);
const METAL_TAGGED: vec2<f32> = vec2<f32>(0.45, 0.55);
// The roughness past which a surface is not traced and reflects the prefiltered sky alone; and the roughness over
// which combine fades from the traced reflection to that sky, so a texel near the cut blends instead of flipping as
// its wet gloss crosses it.
const TRACED_ROUGHNESS_LIMIT: f32 = 0.6;
const TRACED_ROUGHNESS_FADE: vec2<f32> = vec2<f32>(0.4, 0.6);

// The variance a G-buffer normal has across a pixel, as a slope's: its differences to the next texel each way,
// scaled by one over pi (an independent take on Kaplanyan's 2016 stable specular highlights, his two sigma squared)
// and held to `GEOMETRIC_MOST` of the lobe's alpha squared, under which the roughness stays traceable, so the lobe
// widens over a bumpy normal map seen at grazing instead of the sky sparkling in each bump as foil, and a wet road's
// reflection blurs rather than goes.
const GEOMETRIC_SCALE: f32 = 0.318;
const GEOMETRIC_MOST: f32 = 0.04;

fn geometric_variance(normals: texture_2d<f32>, texel: vec2<i32>) -> f32 {
  let last: vec2<i32> = vec2<i32>(textureDimensions(normals)) - 1;
  let here: vec3<f32> = octahedral_decode(textureLoad(normals, texel, 0).xy);
  let across: vec3<f32> = octahedral_decode(textureLoad(normals, min(texel + vec2<i32>(1, 0), last), 0).xy) - here;
  let down: vec3<f32> = octahedral_decode(textureLoad(normals, min(texel + vec2<i32>(0, 1), last), 0).xy) - here;

  return min(GEOMETRIC_SCALE * (dot(across, across) + dot(down, down)), GEOMETRIC_MOST);
}

// A surface's roughness for its reflection's lobe: the engine's gloss as `reflection_roughness` reads it, the engine's
// wetness and a puddle's water's written into the gloss by the wet patch, widened by the variance of the ripples the
// patch's filtering folded away (Toksvig), so a far puddle's rings soften its reflection instead of vanishing. The
// variance is a slope's, which adds to the lobe's alpha squared, and what comes back is the roughness that alpha is
// the square of.
fn traced_roughness(gloss: f32, variance: f32) -> f32 {
  let alpha: f32 = lobe_alpha(reflection_roughness(gloss));

  return sqrt(sqrt(alpha * alpha + max(variance, 0.0)));
}

// The GGX lobe's alpha for a roughness: its square, as the Disney and Unreal convention has it, so a roughness of
// 0.02 is a lobe a fiftieth of a degree wide and not a degree, which read as the whole puddle jittering.
fn lobe_alpha(roughness: f32) -> f32 {
  return roughness * roughness;
}

// The share of metal in a texture's material, as the G-buffer's `material.z` holds it (`float(material) + weight`):
// rising over Phong to Metal, falling over Metal to Oren-Nayar, none in any other.
fn material_metal(slice: f32) -> f32 {
  return select(0.0, slice - 2.0, slice >= 2.0 && slice < 3.0) + select(0.0, 4.0 - slice, slice >= 3.0 && slice < 4.0);
}

// A surface's reflectance head on: a dielectric's, lifted towards its own colour as its gloss passes the metals' where
// its material (`slice`, the G-buffer's `material.z`) marks it a metal, and water's over the share of it a puddle's
// clear coat covers.
fn reflectance_head_on(albedo: vec3<f32>, gloss: f32, coat: f32, slice: f32) -> vec3<f32> {
  let metal: f32 = smoothstep(METAL_GLOSS.x, METAL_GLOSS.y, gloss) *
    smoothstep(METAL_TAGGED.x, METAL_TAGGED.y, material_metal(slice));

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

// Times a reflection re-aims through the surroundings cube's depth; and the metres past which a texel of it is as far
// as the sky, re-aimed by nothing.
const PARALLAX_STEPS: u32 = 3u;
const PARALLAX_FAR: f32 = 1000.0;

// The direction a reflection reads the surroundings cube along: not its own from the camera, which pasted every
// surrounding tree at infinity, so a puddle's tree swung across it as the camera moved, but from the cube's centre
// through the point the ray would meet. `origin` is the reflecting point's offset from the camera
// in renderer space, `direction` the ray's; the cube's first mip holds each texel's metres from the camera (none for
// the sky), and the point is found by a few re-aims: the ray's length to the depth read along the last aim, projected
// onto the ray. Over the sky, or where the cube holds no depth, the direction is the ray's own.
fn surroundings_direction(cube: texture_cube<f32>, cube_sampler: sampler, origin: vec3<f32>, direction: vec3<f32>) -> vec3<f32> {
  var aim: vec3<f32> = direction;
  var along: f32 = textureSampleLevel(cube, cube_sampler, aim, 0.0).a;

  if (along <= 0.0 || along >= PARALLAX_FAR) {
    return direction;
  }

  for (var step: u32 = 0u; step < PARALLAX_STEPS; step++) {
    aim = normalize(origin + direction * along);

    let depth: f32 = textureSampleLevel(cube, cube_sampler, aim, 0.0).a;

    if (depth <= 0.0 || depth >= PARALLAX_FAR) {
      return aim;
    }

    along = max(dot(aim * depth - origin, direction), 0.0);
  }

  return normalize(origin + direction * along);
}

// The sky a reflection reads where it meets nothing, along a renderer-space direction at a roughness: the prefiltered
// cube at the mip the roughness names, its last mip at roughness one whatever its size.
fn prefiltered_sky(cube: texture_cube<f32>, cube_sampler: sampler, direction: vec3<f32>, roughness: f32) -> vec3<f32> {
  let last: f32 = f32(textureNumLevels(cube) - 1u);

  return textureSampleLevel(cube, cube_sampler, direction, saturate(roughness) * last).rgb;
}

// A surface's roughness for its reflection's lobe, from the engine's gloss: X-Ray's terrain runs from none on bare
// ground to about 0.37 on asphalt (roughness 0.15); the wet film's 0.43 is a roughness of 0.12, a puddle's water's
// 0.74 one of 0.02.
fn reflection_roughness(gloss: f32) -> f32 {
  let rough: f32 = 1.0 - sqrt(saturate(gloss));

  return rough * rough;
}
