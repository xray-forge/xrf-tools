#import "common/camera"
#import "common/octahedral"
#import "common/lighting"
#import "common/hmodel"
#import "common/sky"
#import "common/fullscreen"
#import "common/occlusion"

// `hmodel` and `combine_1` over the light the frame accumulated, then the fog and tonemap of `combine_2`, faded into the
// weather's sky, into the viewport's scene. Unlit, a surface is its raw albedo.

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
// The sky as drawn, clouds and all, blurred by bearing and height.
@group(1) @binding(10) var haze_map: texture_2d<f32>;

// What shows where nothing was drawn and neither the sky nor the fog is: the backdrop's colour overhead and around.
const BACKDROP_ZENITH: vec3<f32> = vec3<f32>(0.13, 0.15, 0.19);
const BACKDROP_HORIZON: vec3<f32> = vec3<f32>(0.32, 0.34, 0.37);

// The world direction through a pixel of the viewport.
fn pixel_direction(pixel: vec2<f32>) -> vec3<f32> {
  let toward: vec3<f32> = camera_view_position(pixel, 0.5);

  return normalize((transpose(camera.view) * vec4<f32>(toward, 0.0)).xyz);
}

// The sky's haze along a world direction: the sky as drawn there, clouds and all, blurred past any shape; below the
// horizon, the horizon's own, since nothing stands behind the ground.
fn sky_haze(direction: vec3<f32>) -> vec3<f32> {
  let lifted: vec3<f32> = normalize(vec3<f32>(direction.x, max(direction.y, 0.0), direction.z));

  return textureSampleLevel(haze_map, sky_repeat, sky_haze_coordinates(lifted), 0.0).rgb;
}

// `hmodel` over the G-buffer's view space normal and point.
fn shaded_color(albedo: vec4<f32>, light: vec4<f32>, normal: vec3<f32>, position: vec3<f32>, slice: f32, occlusion: f32,
  visible: f32) -> vec3<f32> {
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));

  return hmodel(lighting, material_lut, lut_sampler, sky_environment_0, sky_environment_1, sky_clamp, albedo, light,
    normalize(rotation * normal), normalize(rotation * normalize(position)), slice, occlusion, visible);
}

@fragment
fn fs_combine(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);
  let is_lit: bool = lighting.params.y > 0.5;
  let scale: f32 = frame_scale(lighting, exposure);
  let is_fogged: bool = is_lit && lighting.fog_color.w > 0.5;
  // A level's frame draws its sky behind everything, so an empty pixel is the sky there instead.
  let is_sky_drawn: bool = is_lit && lighting.fog.w > 0.5;
  let direction: vec3<f32> = pixel_direction(in.clip.xy);
  let position: vec3<f32> = camera_view_position(in.clip.xy, max(depth, 1e-7));
  let fog: f32 = select(0.0, fog_amount(lighting, position), is_fogged);
  // The far plane ends where fog is total: past it is what anything there would have come to, the sky or the fog.
  let is_empty: bool = depth <= 0.0 || fog >= 1.0;

  if (is_empty) {
    if (is_sky_drawn) {
      return vec4<f32>(sky_shown(lighting, direction, scale), 1.0);
    }

    if (is_fogged) {
      return vec4<f32>(tonemap(lighting.fog_color.rgb, scale), 1.0);
    }

    return vec4<f32>(mix(BACKDROP_HORIZON, BACKDROP_ZENITH, sqrt(saturate(direction.y))), 1.0);
  }

  let albedo: vec4<f32> = textureLoad(albedo_target, texel, 0);

  if (!is_lit) {
    return vec4<f32>(albedo.rgb, 1.0);
  }

  let material: vec4<f32> = textureLoad(material_target, texel, 0);
  let light: vec4<f32> = textureLoad(light_target, texel, 0);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  // How much of the hemisphere and ambient light reaches the point, as `combine_1` multiplies them by `occ`.
  let visible: f32 = select(1.0, upsampled_occlusion(occlusion_target, floor(in.clip.xy), -position.z),
    lighting.params.w > 0.5);
  let occlusion: f32 = mix(1.0, material.x, camera.switches.z);
  let shaded: vec3<f32> = shaded_color(albedo, light, normal, position, material.z, occlusion, visible);
  // The engine fogs towards `fog_color` before the tonemap, then fades into the sky itself by the fog squared.
  let finished: vec3<f32> = tonemap(mix(shaded, lighting.fog_color.rgb, fog), scale);

  if (!is_sky_drawn) {
    return vec4<f32>(finished, 1.0);
  }

  // The haze takes the place of both, by the engine's own two blends: the distance takes the colour of the sky behind
  // it, no cloud's shape showing, the fog towards the haze's lit colour before the tonemap.
  if (lighting.fog.z > 0.5) {
    let haze: vec3<f32> = sky_haze(direction);
    let fogged: vec3<f32> = tonemap(mix(shaded, untonemap(haze) / max(scale, 1e-6), fog), scale);

    return vec4<f32>(mix(fogged, haze, fog * fog), 1.0);
  }

  return vec4<f32>(mix(finished, sky_shown(lighting, direction, scale), fog * fog), 1.0);
}
