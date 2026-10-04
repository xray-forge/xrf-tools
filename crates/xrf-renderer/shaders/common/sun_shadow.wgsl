// The sun's shadow as every pass lit by the sun reads it: its cascades' maps and what they were drawn with, passed in
// so each pass binds them where it likes.

struct Shadows {
  // Renderer space into each cascade's clip space, as its map was last drawn.
  matrices: array<mat4x4<f32>, 4>,
  // Metres one texel of each cascade's map is across.
  texels: vec4<f32>,
  // xyz: where the camera looks, which the last cascade fades out towards.
  forward: vec4<f32>,
  count: u32,
  filter_reach: u32,
  resolution: f32,
  bias: f32,
  blend: f32,
  pad0: f32,
  pad1: f32,
  pad2: f32,
};

// The share of a map's edge a point is kept off, so the filter never reads past it into the next cascade's edge.
const EDGE: f32 = 0.02;

// How far from its centre, in shares of the map, the last cascade starts to fade out: the engine's `border`.
const BORDER: f32 = 0.4;

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

// How much of the sun reaches a point: the first cascade whose map holds it, blended into the next within `blend` of
// its edge so the switch to a coarser map is never a line; past every cascade the sun reaches it whole. The point is
// moved along its normal first so a lit surface never shadows itself, up to twice as far where the light grazes it.
fn sun_shadow(maps: texture_depth_2d_array, shadows: Shadows, position: vec3<f32>, normal: vec3<f32>, facing: f32)
  -> f32 {
  let lean: f32 = 1.0 + sqrt(1.0 - saturate(facing) * saturate(facing));
  var lit: f32 = 1.0;
  var is_blending: bool = false;
  var inset: f32 = 0.0;

  for (var view: u32 = 0u; view < min(shadows.count, 4u); view++) {
    let moved: vec3<f32> = position + normal * (shadows.bias * shadows.texels[view] * lean);
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
      return mix(view_lit, lit, saturate(inset / shadows.blend));
    }

    lit = view_lit;
    inset = min(min(uv.x, uv.y), min(1.0 - uv.x, 1.0 - uv.y)) - EDGE;

    // Blended into the next only inside the band, and only where there is a next.
    if (inset < shadows.blend && shadows.count > view + 1u) {
      is_blending = true;
    } else {
      break;
    }
  }

  return lit;
}
