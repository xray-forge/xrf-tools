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

// The sun's share at a point by one cascade alone, compared over the square of texels the filter reaches; the last
// cascade fades out towards its far edge, on the side the camera looks towards, as `accum_sun_far` does.
fn cascade_lit(maps: texture_depth_2d_array, shadows: Shadows, view: u32, clip: vec4<f32>, uv: vec2<f32>) -> f32 {
  let size: i32 = i32(shadows.resolution);
  let center: vec2<i32> = vec2<i32>(uv * shadows.resolution);
  let reach: i32 = i32(shadows.filter_reach);
  var total: f32 = 0.0;
  var taps: f32 = 0.0;

  for (var x: i32 = -reach; x <= reach; x++) {
    for (var y: i32 = -reach; y <= reach; y++) {
      let at: vec2<i32> = clamp(center + vec2<i32>(x, y), vec2<i32>(0), vec2<i32>(size - 1));

      // Depth is reversed: a texel nearer the sun holds the larger value.
      total += select(0.0, 1.0, clip.z >= textureLoad(maps, at, view, 0));
      taps += 1.0;
    }
  }

  var lit: f32 = total / taps;

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
  for (var view: u32 = 0u; view < min(shadows.count, 4u); view++) {
    let clip: vec4<f32> = shadows.matrices[view] * vec4<f32>(position, 1.0);
    let uv: vec2<f32> = vec2<f32>(clip.x * 0.5 + 0.5, clip.y * -0.5 + 0.5);

    if (all(uv > vec2<f32>(EDGE)) && all(uv < vec2<f32>(1.0 - EDGE)) && clip.z > 0.0 && clip.z < 1.0) {
      return true;
    }
  }

  return false;
}

// How much of the sun reaches a point: the first cascade whose map holds it, blended into the next within its band
// (`blends`) of its edge so the switch to a coarser map is never a line; past every cascade the sun reaches it whole.
// The point is moved along its normal first so a lit surface never shadows itself, up to twice as far where the light
// grazes it, and as much again for each texel the filter reaches out: on a slope its outer taps read the surface's own
// depth a texel's rise away, which striped a flat puddle on sloped ground where the bumps hid it elsewhere.
fn sun_shadow(maps: texture_depth_2d_array, shadows: Shadows, position: vec3<f32>, normal: vec3<f32>, facing: f32)
  -> f32 {
  return sun_shadow_by(maps, shadows, position, normal, facing, OFFSET_ANY);
}

// The sun on a plant: as `sun_shadow`, its point moved no further than `PLANT_OFFSET_MOST` in any cascade.
fn plant_sun_shadow(maps: texture_depth_2d_array, shadows: Shadows, position: vec3<f32>, normal: vec3<f32>,
  facing: f32) -> f32 {
  return sun_shadow_by(maps, shadows, position, normal, facing, PLANT_OFFSET_MOST);
}

// `sun_shadow` with its point moved `most` metres at most.
fn sun_shadow_by(maps: texture_depth_2d_array, shadows: Shadows, position: vec3<f32>, normal: vec3<f32>, facing: f32,
  most: f32) -> f32 {
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

    let view_lit: f32 = cascade_lit(maps, shadows, view, clip, uv);

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
