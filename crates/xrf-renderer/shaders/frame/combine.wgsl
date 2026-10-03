#import "common/camera"
#import "common/octahedral"
#import "common/lighting"
#import "common/sky"
#import "common/fullscreen"

// `hmodel` and `combine_1` over the light the frame accumulated, then the fog and tonemap of `combine_2`, faded into the
// weather's sky, into the viewport's scene. Unlit, a surface is its raw albedo.

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
// The sky as drawn, clouds and all, blurred by bearing and height.
@group(1) @binding(10) var haze_map: texture_2d<f32>;

// What shows where nothing was drawn and neither the sky nor the fog is: the backdrop's colour overhead and around.
const BACKDROP_ZENITH: vec3<f32> = vec3<f32>(0.13, 0.15, 0.19);
const BACKDROP_HORIZON: vec3<f32> = vec3<f32>(0.32, 0.34, 0.37);

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

// `env_color * lerp(env_s0, env_s1, w)` along a world direction, squared as `hmodel` squares it: the lighting's
// stand-in until both cubes are up.
fn hemisphere_environment(direction: vec3<f32>) -> vec3<f32> {
  let irradiance: vec3<f32> = mix(lighting.sky_irradiance.rgb, sky_environment(lighting, direction), lighting.sky_irradiance.w);
  let environment: vec3<f32> = lighting.environment.rgb * irradiance;

  return environment * environment;
}

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

// `hmodel` and `combine_1`: the hemisphere and ambient, times the screen's occlusion as `combine_1` multiplies
// `hdiffuse` and `hspecular` by `occ`, added to what the lights accumulated. Vanilla adds every reflection white;
// Anomaly's tints the lights' by their colour and multiplies the lit albedo by the hemisphere's.
fn shaded_color(albedo: vec4<f32>, light: vec4<f32>, normal: vec3<f32>, position: vec3<f32>, slice: f32, occlusion: f32,
  visible: f32) -> vec3<f32> {
  let to_point: vec3<f32> = normalize(position);
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
  let normal_world: vec3<f32> = normalize(rotation * normal);
  let to_point_world: vec3<f32> = normalize(rotation * to_point);
  let reflected: vec3<f32> = reflect(to_point_world, normal_world);
  let gloss: f32 = albedo.a;
  // The hemisphere looked up by occlusion and by how far the reflection turns from the view.
  let hemisphere: vec4<f32> = lookup(occlusion, 0.5 + 0.5 * dot(reflected, to_point_world), slice);
  let hemisphere_diffuse: vec3<f32> = (hemisphere_environment(normal_world) * hemisphere.x + lighting.ambient.rgb) * visible;
  let is_extended: bool = lighting.engine.x > 0.5;
  // Anomaly's `hmodel` reads the reflection on the cube's faces, remapped short of the top one, and weighs it by the
  // rain: none while dry, a sheen as it pours, brightest where the hemisphere lights least.
  let on_faces: vec3<f32> = reflected / max(max(abs(reflected.x), abs(reflected.y)), abs(reflected.z));
  let reflection: vec3<f32> = select(
    vec3<f32>(reflected.x, reflected.y * 2.0 - 1.0, reflected.z),
    vec3<f32>(on_faces.x, select(on_faces.y, on_faces.y * 2.0 - 1.0, on_faces.y < 0.999), on_faces.z),
    is_extended,
  );
  let rain: f32 = lighting.engine.y;
  let weight: f32 = select(gloss, (gloss + rain * 0.25) * (1.0 - hemisphere.x) * (rain * 30.0), is_extended);
  let hemisphere_gloss: vec3<f32> = hemisphere_environment(reflection) * hemisphere.y * weight * visible;
  // `C = D * light`: the lit albedo, and the gloss times what the lights reflect.
  let lit: vec3<f32> = albedo.rgb * (light.rgb + hemisphere_diffuse);
  let glossed: f32 = gloss * light.a;

  if (is_extended) {
    return lit + light.rgb * glossed + hemisphere_gloss * lit;
  }

  return lit + glossed + hemisphere_gloss;
}

@fragment
fn fs_combine(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);
  let is_lit: bool = lighting.params.y > 0.5;
  let scale: f32 = lighting.params.x * select(1.0, exposure.adapted, lighting.params.z > 0.5);
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
  let visible: f32 = select(1.0, upsampled_occlusion(floor(in.clip.xy), -position.z), lighting.params.w > 0.5);
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
