#import "common/camera"
#import "common/octahedral"

// The G-buffer lit by the sun and the hemisphere, as the engine's base lighting combines them, then tonemapped into
// the viewport's rectangle of the window.

struct Lighting {
  // xyz: towards the sun, in view space.
  to_sun: vec4<f32>,
  // rgb: the sun's colour; w: its specular weight.
  sun: vec4<f32>,
  ambient: vec4<f32>,
  environment: vec4<f32>,
  sky_irradiance: vec4<f32>,
  sky_zenith: vec4<f32>,
  sky_horizon: vec4<f32>,
  // x: the tonemap's scale.
  params: vec4<f32>,
};

@group(1) @binding(0) var albedo_target: texture_2d<f32>;
@group(1) @binding(1) var normal_target: texture_2d<f32>;
@group(1) @binding(2) var material_target: texture_2d<f32>;
@group(1) @binding(3) var depth_target: texture_depth_2d;
@group(1) @binding(4) var material_lut: texture_3d<f32>;
@group(1) @binding(5) var lut_sampler: sampler;
@group(1) @binding(6) var<uniform> lighting: Lighting;

struct CombineVarying {
  @builtin(position) clip: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> CombineVarying {
  let uv: vec2<f32> = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
  var out: CombineVarying;

  out.clip = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);

  return out;
}

// The engine's filmic curve, `x * (1 + x / 1.7²) / (1 + x)`.
fn tonemap(color: vec3<f32>) -> vec3<f32> {
  let x: vec3<f32> = color * lighting.params.x;

  return x * (1.0 + x / (1.7 * 1.7)) / (1.0 + x);
}

fn lookup(n_dot_l: f32, n_dot_h: f32, slice: f32) -> vec4<f32> {
  return textureSampleLevel(material_lut, lut_sampler, vec3<f32>(n_dot_l, n_dot_h, slice), 0.0);
}

@fragment
fn fs_main(in: CombineVarying) -> @location(0) vec4<f32> {
  let size: vec2<f32> = camera.viewport.xy;
  let pixel: vec2<f32> = in.clip.xy - camera.viewport.zw;
  let texel: vec2<i32> = vec2<i32>(clamp(pixel, vec2<f32>(0.0), size - 1.0));
  let depth: f32 = textureLoad(depth_target, texel, 0);
  let ndc: vec2<f32> = vec2<f32>(pixel.x / size.x * 2.0 - 1.0, 1.0 - pixel.y / size.y * 2.0);

  // Nothing drawn: the sky, plain until the weather draws one.
  if (depth <= 0.0) {
    let direction: vec3<f32> = normalize(camera_unproject(ndc, 0.5) - camera.position.xyz);
    let sky: vec3<f32> = mix(lighting.sky_horizon.rgb, lighting.sky_zenith.rgb, sqrt(saturate(direction.y)));

    return vec4<f32>(sky, 1.0);
  }

  let albedo: vec4<f32> = textureLoad(albedo_target, texel, 0);
  let material: vec4<f32> = textureLoad(material_target, texel, 0);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let projected: vec4<f32> = camera.inverse_projection * vec4<f32>(ndc, depth, 1.0);
  let to_eye: vec3<f32> = -normalize(projected.xyz / projected.w);
  let to_sun: vec3<f32> = lighting.to_sun.xyz;
  let half_way: vec3<f32> = normalize(to_sun + to_eye);
  let slice: f32 = material.z;
  let sun: vec4<f32> = lookup(dot(to_sun, normal), dot(half_way, normal), slice);
  let sun_light: vec3<f32> = lighting.sun.rgb * sun.x;
  let sun_specular: f32 = lighting.sun.w * sun.y;
  let occlusion: f32 = mix(1.0, material.x, camera.switches.z);
  let n_dot_v: f32 = dot(normal, to_eye);
  // `dot(reflect(V, N), V)`, which is the same whichever way V points.
  let hemi: vec4<f32> = lookup(occlusion, 0.5 + 0.5 * (1.0 - 2.0 * n_dot_v * n_dot_v), slice);
  // todo: Sample the sky's cube for the environment once the weather draws a sky.
  let environment: vec3<f32> = pow(lighting.environment.rgb * lighting.sky_irradiance.rgb, vec3<f32>(2.0));
  let hemi_diffuse: vec3<f32> = environment * hemi.x + lighting.ambient.rgb;
  let gloss: f32 = albedo.a;
  let color: vec3<f32> = albedo.rgb * (sun_light + hemi_diffuse) + gloss * sun_specular
    + environment * hemi.y * gloss;

  return vec4<f32>(tonemap(color), 1.0);
}
