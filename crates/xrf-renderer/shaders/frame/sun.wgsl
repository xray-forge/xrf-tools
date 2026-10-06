#import "common/camera"
#import "common/octahedral"
#import "common/lighting"
#import "common/fullscreen"
#import "common/sun_shadow"

// `accum_sun`: the sun at every drawn pixel, `Ldynamic_color * plight_infinity(m, P, N, L)`, times how much of it
// reaches the pixel through the cascades' maps. Diffuse in colour, specular in alpha, into the light the frame
// accumulates.

@group(1) @binding(0) var normal_target: texture_2d<f32>;
@group(1) @binding(1) var material_target: texture_2d<f32>;
@group(1) @binding(2) var depth_target: texture_depth_2d;
@group(1) @binding(3) var material_lut: texture_3d<f32>;
@group(1) @binding(4) var lut_sampler: sampler;
@group(1) @binding(5) var<uniform> lighting: Lighting;
@group(1) @binding(6) var shadow_maps: texture_depth_2d_array;
@group(1) @binding(7) var<uniform> shadows: Shadows;

// What `accum_emissive.ps` writes into a self-lit surface's light.
const EMISSIVE_LIGHT: f32 = 16.0;

@fragment
fn fs_sun(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    return vec4<f32>(0.0);
  }

  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let material: vec4<f32> = textureLoad(material_target, texel, 0);

  // `accum_emissive`: a self-lit surface's light is filled with sixteen, which the local lights then add to.
  if (has_mark(material.a, MARK_EMISSIVE)) {
    return vec4<f32>(EMISSIVE_LIGHT);
  }

  let slice: f32 = material.z;
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
  let shadow: f32 = sun_shadow(shadow_maps, shadows, world, world_normal, dot(normal, to_light));

  return vec4<f32>(lighting.sun.rgb * lit.x, lighting.sun.w * lit.y) * shadow;
}
