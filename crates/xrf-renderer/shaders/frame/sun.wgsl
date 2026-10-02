#import "common/camera"
#import "common/octahedral"
#import "common/lighting"
#import "common/fullscreen"

// `accum_sun`: the sun at every drawn pixel, `Ldynamic_color * plight_infinity(m, P, N, L)`, times how much of it
// reaches the pixel through the cascades' maps. Diffuse in colour, specular in alpha, into the light the frame
// accumulates.

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

@group(1) @binding(0) var normal_target: texture_2d<f32>;
@group(1) @binding(1) var material_target: texture_2d<f32>;
@group(1) @binding(2) var depth_target: texture_depth_2d;
@group(1) @binding(3) var material_lut: texture_3d<f32>;
@group(1) @binding(4) var lut_sampler: sampler;
@group(1) @binding(5) var<uniform> lighting: Lighting;
@group(1) @binding(6) var shadow_maps: texture_depth_2d_array;
@group(1) @binding(7) var<uniform> shadows: Shadows;

// The share of a map's edge a point is kept off, so the filter never reads past it into the next cascade's edge.
const EDGE: f32 = 0.02;

// How far from its centre, in shares of the map, the last cascade starts to fade out: the engine's `border`.
const BORDER: f32 = 0.4;

// The sun's share at a point by one cascade alone, compared over the square of texels the filter reaches; the last
// cascade fades out towards its far edge, on the side the camera looks towards, as `accum_sun_far` does.
fn cascade_lit(view: u32, clip: vec4<f32>, uv: vec2<f32>) -> f32 {
  let size: i32 = i32(shadows.resolution);
  let center: vec2<i32> = vec2<i32>(uv * shadows.resolution);
  let reach: i32 = i32(shadows.filter_reach);
  var total: f32 = 0.0;
  var taps: f32 = 0.0;

  for (var x: i32 = -reach; x <= reach; x++) {
    for (var y: i32 = -reach; y <= reach; y++) {
      let at: vec2<i32> = clamp(center + vec2<i32>(x, y), vec2<i32>(0), vec2<i32>(size - 1));

      // Depth is reversed: a texel nearer the sun holds the larger value.
      total += select(0.0, 1.0, clip.z >= textureLoad(shadow_maps, at, view, 0));
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

// How much of the sun reaches a point: the first cascade whose map holds it, blended into the next within `blend` of
// its edge so the switch to a coarser map is never a line; past every cascade the sun reaches it whole. The point is
// moved along its normal first so a lit surface never shadows itself, up to twice as far where the light grazes it.
fn sun_shadow(position: vec3<f32>, normal: vec3<f32>, facing: f32) -> f32 {
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

    let view_lit: f32 = cascade_lit(view, clip, uv);

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

@fragment
fn fs_sun(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    return vec4<f32>(0.0);
  }

  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let slice: f32 = textureLoad(material_target, texel, 0).z;
  // `plight_infinity`: L towards the light, V towards the eye, H halfway.
  let to_light: vec3<f32> = lighting.to_sun.xyz;
  let half_way: vec3<f32> = normalize(to_light - normalize(position));
  let lit: vec4<f32> = textureSampleLevel(
    material_lut,
    lut_sampler,
    vec3<f32>(dot(to_light, normal), dot(half_way, normal), slice),
    0.0
  );
  let world: vec3<f32> = (transpose(camera.view) * vec4<f32>(position, 0.0)).xyz + camera.position.xyz;
  let world_normal: vec3<f32> = normalize((transpose(camera.view) * vec4<f32>(normal, 0.0)).xyz);
  let shadow: f32 = sun_shadow(world, world_normal, dot(normal, to_light));

  return vec4<f32>(lighting.sun.rgb * lit.x, lighting.sun.w * lit.y) * shadow;
}
