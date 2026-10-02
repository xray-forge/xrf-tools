#import "common/camera"
#import "common/octahedral"
#import "common/lighting"
#import "common/fullscreen"

// `hmodel` and `combine_1` over the light the frame accumulated, then the tonemap of `combine_2`, into the viewport's
// scene. Unlit, a surface is its raw albedo.

struct Exposure {
  // The scale the tonemap multiplies by, adapted on the GPU frame by frame.
  adapted: f32,
};

@group(1) @binding(0) var albedo_target: texture_2d<f32>;
@group(1) @binding(1) var normal_target: texture_2d<f32>;
@group(1) @binding(2) var material_target: texture_2d<f32>;
@group(1) @binding(3) var depth_target: texture_depth_2d;
@group(1) @binding(4) var light_target: texture_2d<f32>;
@group(1) @binding(5) var material_lut: texture_3d<f32>;
@group(1) @binding(6) var lut_sampler: sampler;
@group(1) @binding(7) var<uniform> lighting: Lighting;
@group(1) @binding(8) var<storage, read> exposure: Exposure;
// Visibility, then distance along the view, at half the frame's size.
@group(1) @binding(9) var occlusion_target: texture_2d<f32>;

// The occlusion searched at half resolution brought up to the frame's pixel: the four searched pixels around it by how
// near each lies, and each by how near its distance lies to the pixel's, so an edge never takes the other side's.
fn upsampled_occlusion(pixel: vec2<f32>, distance: f32) -> f32 {
  let last: vec2<f32> = vec2<f32>(textureDimensions(occlusion_target)) - 1.0;
  let base: vec2<f32> = floor(pixel / 2.0);
  let fraction: vec2<f32> = (pixel - base * 2.0) * 0.5;
  var sum: f32 = 0.0;
  var weights: f32 = 0.0;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let offset: vec2<f32> = vec2<f32>(f32(corner & 1u), f32(corner >> 1u));
    let texel: vec4<f32> = textureLoad(occlusion_target, vec2<i32>(clamp(base + offset, vec2<f32>(0.0), last)), 0);
    let bilinear: f32 = mix(1.0 - fraction.x, fraction.x, offset.x) * mix(1.0 - fraction.y, fraction.y, offset.y);
    let difference: f32 = abs(texel.y - distance) / max(distance, 1e-3);

    sum += texel.x * bilinear / (difference * difference + 1e-3);
    weights += bilinear / (difference * difference + 1e-3);
  }

  return sum / max(weights, 1e-6);
}

fn lookup(x: f32, y: f32, slice: f32) -> vec4<f32> {
  return textureSampleLevel(material_lut, lut_sampler, vec3<f32>(x, y, slice), 0.0);
}

// `env_color * env_s0` squared, as `hmodel` squares it: the sky's irradiance until the weather draws its cubes.
fn hemisphere_environment() -> vec3<f32> {
  let environment: vec3<f32> = lighting.environment.rgb * lighting.sky_irradiance.rgb;

  return environment * environment;
}

@fragment
fn fs_combine(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  // Nothing drawn: the sky, plain until the weather draws one.
  if (depth <= 0.0) {
    let toward: vec3<f32> = camera_view_position(in.clip.xy, 0.5);
    let direction: vec3<f32> = normalize((transpose(camera.view) * vec4<f32>(toward, 0.0)).xyz);
    let sky: vec3<f32> = mix(lighting.sky_horizon.rgb, lighting.sky_zenith.rgb, sqrt(saturate(direction.y)));

    return vec4<f32>(sky, 1.0);
  }

  let albedo: vec4<f32> = textureLoad(albedo_target, texel, 0);

  if (lighting.params.y < 0.5) {
    return vec4<f32>(albedo.rgb, 1.0);
  }

  let material: vec4<f32> = textureLoad(material_target, texel, 0);
  let light: vec4<f32> = textureLoad(light_target, texel, 0);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let to_point: vec3<f32> = normalize(position);
  // How much of the hemisphere and ambient light reaches the point, as `combine_1` multiplies them by `occ`.
  let visible: f32 = select(1.0, upsampled_occlusion(floor(in.clip.xy), -position.z), lighting.params.w > 0.5);
  let slice: f32 = material.z;
  let gloss: f32 = albedo.a;
  let occlusion: f32 = mix(1.0, material.x, camera.switches.z);
  // `hmodel`: the hemisphere looked up by occlusion and by how far the reflection turns from the view.
  let hemisphere: vec4<f32> = lookup(occlusion, 0.5 + 0.5 * dot(reflect(to_point, normal), to_point), slice);
  let environment: vec3<f32> = hemisphere_environment();
  let hemisphere_diffuse: vec3<f32> = (environment * hemisphere.x + lighting.ambient.rgb) * visible;
  let hemisphere_gloss: vec3<f32> = environment * hemisphere.y * gloss * visible;
  // `C = D * light`: the lit albedo, the gloss times what the lights reflect, and the hemisphere's reflection.
  let color: vec3<f32> = albedo.rgb * (light.rgb + hemisphere_diffuse) + gloss * light.a + hemisphere_gloss;
  let scale: f32 = lighting.params.x * select(1.0, exposure.adapted, lighting.params.z > 0.5);

  return vec4<f32>(tonemap(color, scale), 1.0);
}
