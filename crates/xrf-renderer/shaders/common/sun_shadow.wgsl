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

// The texels the filter reaches each way where a pipeline fixes it, which compiles the filter's loops to their count;
// below nought, the shadows' uniform says. Bounded by the uniform, the loops run several times slower on Vulkan.
override SUN_FILTER_REACH: i32 = -1;

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
fn cascade_lit(maps: texture_depth_2d_array, compare: sampler_comparison, shadows: Shadows, view: u32, clip: vec4<f32>,
  uv: vec2<f32>) -> f32 {
  let reach: i32 = select(i32(shadows.filter_reach), SUN_FILTER_REACH, SUN_FILTER_REACH >= 0);
  let at: vec2<f32> = uv * shadows.resolution - 0.5;
  let first: vec2<f32> = floor(at) - f32(reach);
  let fraction: vec2<f32> = at - floor(at);
  let last: i32 = 2 * reach + 1;
  var total: f32 = 0.0;

  for (var y: i32 = 0; y < last; y += 2) {
    for (var x: i32 = 0; x < last; x += 2) {
      // The four texels about the corner between texels (x, y) and (x + 1, y + 1): `w` (x, y), `z` (x + 1, y), `x`
      // (x, y + 1), `y` (x + 1, y + 1).
      let corner: vec2<f32> = (first + vec2<f32>(f32(x), f32(y)) + 1.0) / shadows.resolution;
      let compared: vec4<f32> = textureGatherCompare(maps, compare, corner, view, clip.z);
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
// The point is moved along its normal first so a lit surface never shadows itself, up to twice as far where the light
// grazes it, and as much again for each texel the filter reaches out: on a slope its outer taps read the surface's own
// depth a texel's rise away, which striped a flat puddle on sloped ground where the bumps hid it elsewhere.
fn sun_shadow(maps: texture_depth_2d_array, compare: sampler_comparison, shadows: Shadows, position: vec3<f32>,
  normal: vec3<f32>, facing: f32) -> f32 {
  return sun_shadow_by(maps, compare, shadows, position, normal, facing, OFFSET_ANY);
}

// The sun on a plant: as `sun_shadow`, its point moved no further than `PLANT_OFFSET_MOST` in any cascade.
fn plant_sun_shadow(maps: texture_depth_2d_array, compare: sampler_comparison, shadows: Shadows, position: vec3<f32>,
  normal: vec3<f32>, facing: f32) -> f32 {
  return sun_shadow_by(maps, compare, shadows, position, normal, facing, PLANT_OFFSET_MOST);
}

// `sun_shadow` with its point moved `most` metres at most.
fn sun_shadow_by(maps: texture_depth_2d_array, compare: sampler_comparison, shadows: Shadows, position: vec3<f32>,
  normal: vec3<f32>, facing: f32, most: f32) -> f32 {
  let lean: f32 = 1.0 + sqrt(1.0 - saturate(facing) * saturate(facing));
  var lit: f32 = 1.0;
  var is_blending: bool = false;
  var inset: f32 = 0.0;
  var band: f32 = 0.0;

  for (var view: u32 = 0u; view < min(shadows.count, 4u); view++) {
    let offset: f32 = shadows.bias * shadows.texels[view] * lean * (1.0 + f32(shadows.filter_reach));
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

    let view_lit: f32 = cascade_lit(maps, compare, shadows, view, clip, uv);

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
