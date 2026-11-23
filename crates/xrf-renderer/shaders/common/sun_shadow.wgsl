#import "generated/structs"

// The sun's shadow as every pass lit by the sun reads it: its cascades' maps and what they were drawn with, passed in
// so each pass binds them where it likes.

// The share of a map's edge a point is kept off, so the filter never reads past it into the next cascade's edge.
const EDGE: f32 = 0.02;

// How far from its centre, in shares of the map, the last cascade starts to fade out: the engine's `border`.
const BORDER: f32 = 0.4;

// Metres a plant's point is moved along its normal at most, whatever the cascade: moved as far as a coarse cascade's
// texels, a tuft's lookup left the blades about it and lit its inside, which the finer cascade shadowed, so its shade
// stepped where a cascade's edge crossed it.
const PLANT_OFFSET_MOST: f32 = 0.04;

// What `sun_shadow_by` takes as no limit on how far a point is moved.
const OFFSET_ANY: f32 = 1e9;

// The steepest a surface's plane is followed across a map, as the tangent of its tilt from facing the sun (about 76°):
// past it the light grazes, the plane's depth runs away within a texel, and the push alone keeps the surface lit.
const RECEIVER_SLOPE_MOST: f32 = 4.0;

// How far a surface's plane may turn from the normal it is shaded by and still be followed, as their cosine (60°): past
// it the plane is a silhouette's or a wire's, read across an edge from what stands behind, and names no surface.
const RECEIVER_PLANE_AGREES: f32 = 0.5;

// The texels the filter reaches each way where a pipeline fixes it, which compiles the filter's loops to their count;
// below nought, the shadows' uniform says. Bounded by the uniform, the loops run several times slower on Vulkan.
override SUN_FILTER_REACH: i32 = -1;

// The texels the filter reaches each way: the pipeline's where it fixes them, the shadows' uniform's otherwise.
fn sun_filter_reach(shadows: Shadows) -> i32 {
  return select(i32(shadows.filter_reach), SUN_FILTER_REACH, SUN_FILTER_REACH >= 0);
}

// A texel's weight along one axis of the filter, `index` of the `last + 1` it spans, the point `fraction` past the
// first's centre: the square of texels the filter reaches slid by that fraction, so the edge texels share one weight
// between them and the shadow's edge moves with the point rather than a texel at a time.
fn filter_weight(index: i32, last: i32, fraction: f32) -> f32 {
  return select(select(1.0, fraction, index == last), 1.0 - fraction, index == 0);
}

// The sun's share at a point by one cascade alone: the comparisons over the square of texels the filter reaches each
// way, slid continuously with the point (`filter_weight`), gathered four at a time through the comparison sampler
// (`compare`, lit where the reference is no nearer the sun than the texel); the last cascade fades out towards its far
// edge, on the side the camera looks towards, as `accum_sun_far` does. Both engines weight their taps (CoP's 7 × 7 of
// hardware comparisons, Anomaly's PCSS); an unweighted square of single compares stepped its edge a texel at a time.
// Each four are compared against the depth the surface's own plane has at their corner (`slope`, a texel across and
// down, `cascade_slope`), so the filter's outer texels never read the surface a texel's rise away as its own shadow,
// and the plane's rise over the half texel each of the four stands from the corner is given to the light: a steep
// plane, a wall the sun grazes, otherwise shadowed itself across a recess.
fn cascade_lit(maps: texture_depth_2d_array, compare: sampler_comparison, shadows: Shadows, view: u32, clip: vec4<f32>,
  uv: vec2<f32>, slope: vec2<f32>) -> f32 {
  let reach: i32 = sun_filter_reach(shadows);
  let at: vec2<f32> = uv * shadows.resolution - 0.5;
  let first: vec2<f32> = floor(at) - f32(reach);
  let fraction: vec2<f32> = at - floor(at);
  let last: i32 = 2 * reach + 1;
  // Reversed depth: towards the sun is up, and a reference raised is lit sooner.
  let residual: f32 = 0.5 * (abs(slope.x) + abs(slope.y));
  var total: f32 = 0.0;

  for (var y: i32 = 0; y < last; y += 2) {
    for (var x: i32 = 0; x < last; x += 2) {
      // The four texels about the corner between texels (x, y) and (x + 1, y + 1): `w` (x, y), `z` (x + 1, y), `x`
      // (x, y + 1), `y` (x + 1, y + 1).
      let cell: vec2<f32> = first + vec2<f32>(f32(x), f32(y));
      let corner: vec2<f32> = (cell + 1.0) / shadows.resolution;
      // The surface's plane at the corner between the four, in the texels `at` counts.
      let reference: f32 = clip.z + dot(slope, cell + 0.5 - at) + residual;
      let compared: vec4<f32> = textureGatherCompare(maps, compare, corner, view, reference);
      let across: vec2<f32> = vec2<f32>(filter_weight(x, last, fraction.x), filter_weight(x + 1, last, fraction.x));
      let down: vec2<f32> = vec2<f32>(filter_weight(y, last, fraction.y), filter_weight(y + 1, last, fraction.y));

      total += dot(compared, vec4<f32>(across.x * down.y, across.y * down.y, across.y * down.x, across.x * down.x));
    }
  }

  let side: f32 = f32(2 * reach + 1);
  var lit: f32 = total / (side * side);

  if (shadows.count == view + 1u) {
    let ahead: vec4<f32> = shadows.matrices[view] * vec4<f32>(shadows.forward.xyz, 0.0);
    let offset: vec2<f32> = uv - 0.5;
    let is_ahead: bool = dot(offset, vec2<f32>(ahead.x, -ahead.y)) >= 0.0;
    let out: vec2<f32> = select(vec2<f32>(0.0), abs(offset), is_ahead);
    let kept: f32 = (1.0 - saturate((out.x - BORDER) / (0.5 - BORDER)))
      * (1.0 - saturate((out.y - BORDER) / (0.5 - BORDER)));

    lit += (1.0 - lit) * (1.0 - kept);
  }

  return lit;
}

// Whether the sun reaches a point by one tap of the first cascade whose map holds it, as `accum_volumetric_sun`'s low
// filter takes it; past every cascade it does.
fn sun_shadow_tap(maps: texture_depth_2d_array, shadows: Shadows, position: vec3<f32>) -> f32 {
  for (var view: u32 = 0u; view < min(shadows.count, 4u); view++) {
    let clip: vec4<f32> = shadows.matrices[view] * vec4<f32>(position, 1.0);
    let uv: vec2<f32> = vec2<f32>(clip.x * 0.5 + 0.5, clip.y * -0.5 + 0.5);

    if (all(uv > vec2<f32>(0.0)) && all(uv < vec2<f32>(1.0)) && clip.z > 0.0 && clip.z < 1.0) {
      let texel: vec2<i32> = vec2<i32>(uv * shadows.resolution);

      // Depth is reversed: a texel nearer the sun holds the larger value.
      return select(0.0, 1.0, clip.z >= textureLoad(maps, texel, view, 0));
    }
  }

  return 1.0;
}

// Whether a cascade's map holds a point, so its shadow there is the map's rather than nothing's.
fn is_sun_mapped(shadows: Shadows, position: vec3<f32>) -> bool {
  return sun_cascade(shadows, position) < min(shadows.count, 4u);
}

// The first cascade whose map holds a point, which shades it; the cascades' count where none does.
fn sun_cascade(shadows: Shadows, position: vec3<f32>) -> u32 {
  let count: u32 = min(shadows.count, 4u);

  for (var view: u32 = 0u; view < count; view++) {
    let clip: vec4<f32> = shadows.matrices[view] * vec4<f32>(position, 1.0);
    let uv: vec2<f32> = vec2<f32>(clip.x * 0.5 + 0.5, clip.y * -0.5 + 0.5);

    if (all(uv > vec2<f32>(EDGE)) && all(uv < vec2<f32>(1.0 - EDGE)) && clip.z > 0.0 && clip.z < 1.0) {
      return view;
    }
  }

  return count;
}

// How much of the sun reaches a point: the first cascade whose map holds it, blended into the next within its band
// (`blends`) of its edge so the switch to a coarser map is never a line; past every cascade the sun reaches it whole.
// The point is moved along its shading `normal` first so a lit surface never shadows itself, up to twice as far where
// the light grazes it, and the filter compares each of its texels against the surface's own plane there (`plane`, its
// geometric normal, never a bumped one), so its outer taps do not read the surface a texel's rise away on a slope,
// which striped a flat puddle on sloped ground. Moved only a texel or two, a fitting a few centimetres off a wall keeps
// its shadow on it. A caller whose normal is the surface's own passes it as both.
fn sun_shadow(maps: texture_depth_2d_array, compare: sampler_comparison, shadows: Shadows, position: vec3<f32>,
  normal: vec3<f32>, facing: f32, plane: vec3<f32>) -> f32 {
  return sun_shadow_by(maps, compare, shadows, position, normal, facing, OFFSET_ANY, plane, true);
}

// The sun on a plant: as `sun_shadow`, its point moved no further than `PLANT_OFFSET_MOST` in any cascade, and as much
// again for each texel the filter reaches in place of following its plane, since a blade's or a leaf's normal is bent
// for its shading and names no plane it lies in.
fn plant_sun_shadow(maps: texture_depth_2d_array, compare: sampler_comparison, shadows: Shadows, position: vec3<f32>,
  normal: vec3<f32>, facing: f32) -> f32 {
  return sun_shadow_by(maps, compare, shadows, position, normal, facing, PLANT_OFFSET_MOST, normal, false);
}

// A surface's plane as the sun sees it, the same in every cascade, whose maps share the light's turn: the unit normal
// across and up the maps in `xy`, towards the sun in `z` (reversed depth rises towards it). A map's matrix is a scale
// times a turn, so its rows, each scaled to a unit, are the light's axes.
fn light_plane(shadows: Shadows, plane: vec3<f32>) -> vec3<f32> {
  let to_map: mat4x4<f32> = shadows.matrices[0];
  let across: vec3<f32> = vec3<f32>(to_map[0].x, to_map[1].x, to_map[2].x);
  let up: vec3<f32> = vec3<f32>(to_map[0].y, to_map[1].y, to_map[2].y);
  let toward: vec3<f32> = vec3<f32>(to_map[0].z, to_map[1].z, to_map[2].z);

  return vec3<f32>(dot(across, plane) / length(across), dot(up, plane) / length(up), dot(toward, plane) / length(toward));
}

// The share of a plane's tilt from the sun that is followed (`lit_plane`, from `light_plane`): all of it up to
// `RECEIVER_SLOPE_MOST`, less as the light grazes past it.
fn plane_followed(lit_plane: vec3<f32>) -> f32 {
  return min(1.0, RECEIVER_SLOPE_MOST * lit_plane.z / max(length(lit_plane.xy), 1e-6));
}

// How the depth of a plane facing the sun (`lit_plane`, from `light_plane`) changes a texel across and down one
// cascade's map, `followed` of it: the plane's lean in the light, carried into the map by the ratio of how its depth
// and its width are scaled.
fn cascade_slope(shadows: Shadows, view: u32, lit_plane: vec3<f32>, followed: f32) -> vec2<f32> {
  let to_map: mat4x4<f32> = shadows.matrices[view];
  let across: f32 = length(vec3<f32>(to_map[0].x, to_map[1].x, to_map[2].x));
  let up: f32 = length(vec3<f32>(to_map[0].y, to_map[1].y, to_map[2].y));
  let toward: f32 = length(vec3<f32>(to_map[0].z, to_map[1].z, to_map[2].z));
  // Clip space spans two across a map, its `y` up where the map's rows run down.
  let per_texel: vec2<f32> = vec2<f32>(2.0, -2.0) / shadows.resolution;

  return -lit_plane.xy / lit_plane.z * vec2<f32>(toward / across, toward / up) * per_texel * followed;
}

// `sun_shadow` with its point moved `most` metres at most, following the surface's `plane` where `is_planar`, the plane
// faces the sun and agrees with the shading normal: moved as much again for each texel the filter reaches as far as
// the plane is not followed, as where the light grazes or no plane is.
fn sun_shadow_by(maps: texture_depth_2d_array, compare: sampler_comparison, shadows: Shadows, position: vec3<f32>,
  normal: vec3<f32>, facing: f32, most: f32, plane: vec3<f32>, is_planar: bool) -> f32 {
  let lean: f32 = 1.0 + sqrt(1.0 - saturate(facing) * saturate(facing));
  var lit_plane: vec3<f32> = vec3<f32>(0.0, 0.0, 1.0);

  if (is_planar) {
    lit_plane = light_plane(shadows, plane);
  }

  let is_followed: bool = is_planar && lit_plane.z > 0.0 && dot(plane, normal) > RECEIVER_PLANE_AGREES;
  let followed: f32 = select(0.0, plane_followed(lit_plane), is_followed);
  let reaches: f32 = 1.0 + f32(sun_filter_reach(shadows)) * (1.0 - followed);
  var lit: f32 = 1.0;
  var is_blending: bool = false;
  var inset: f32 = 0.0;
  var band: f32 = 0.0;

  for (var view: u32 = 0u; view < min(shadows.count, 4u); view++) {
    let offset: f32 = shadows.bias * shadows.texels[view] * lean * reaches;
    let moved: vec3<f32> = position + normal * min(offset, most);
    let clip: vec4<f32> = shadows.matrices[view] * vec4<f32>(moved, 1.0);
    let uv: vec2<f32> = vec2<f32>(clip.x * 0.5 + 0.5, clip.y * -0.5 + 0.5);
    let is_inside: bool = all(uv > vec2<f32>(EDGE)) && all(uv < vec2<f32>(1.0 - EDGE)) && clip.z > 0.0 && clip.z < 1.0;

    if (!is_inside) {
      // The next does not hold it either: the one that does alone.
      if (is_blending) {
        break;
      }

      continue;
    }

    var slope: vec2<f32> = vec2<f32>(0.0);

    if (is_followed) {
      slope = cascade_slope(shadows, view, lit_plane, followed);
    }

    let view_lit: f32 = cascade_lit(maps, compare, shadows, view, clip, uv, slope);

    if (is_blending) {
      // At the edge the next cascade's, at the band's inner side the one that holds it.
      return mix(view_lit, lit, saturate(inset / band));
    }

    lit = view_lit;
    inset = min(min(uv.x, uv.y), min(1.0 - uv.x, 1.0 - uv.y)) - EDGE;
    band = shadows.blends[view];

    // Blended into the next only inside the band, and only where there is a next.
    if (inset < band && shadows.count > view + 1u) {
      is_blending = true;
    } else {
      break;
    }
  }

  return lit;
}
