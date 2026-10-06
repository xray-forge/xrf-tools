#import "common/lighting"
#import "common/sky_box"

// The engine's `hmodel` and `combine_1`, for every pass that lights a surface as the deferred frame does. The irradiance
// cubes and the material table come in as arguments, so each pass binds them where it likes.

// The material table's lighting at its three coordinates.
fn material_lookup(table: texture_3d<f32>, table_sampler: sampler, x: f32, y: f32, slice: f32) -> vec4<f32> {
  return textureSampleLevel(table, table_sampler, vec3<f32>(x, y, slice), 0.0);
}

// `env_color * lerp(env_s0, env_s1, w)` along a world direction, squared as `hmodel` squares it: the lighting's
// stand-in until both cubes are up. The cubes take the engine's coordinate, whose `z` is the renderer's negated.
fn hemisphere_environment(state: Lighting, cube_0: texture_cube<f32>, cube_1: texture_cube<f32>, cube_sampler: sampler,
  direction: vec3<f32>) -> vec3<f32> {
  let lookup: vec3<f32> = cube_lookup(direction);
  let cubes: vec3<f32> = mix(
    textureSampleLevel(cube_0, cube_sampler, lookup, 0.0).rgb,
    textureSampleLevel(cube_1, cube_sampler, lookup, 0.0).rgb,
    state.sky.w,
  );
  let irradiance: vec3<f32> = mix(state.sky_irradiance.rgb, cubes, state.sky_irradiance.w);
  let environment: vec3<f32> = state.environment.rgb * irradiance;

  return environment * environment;
}

// `plight_infinity` for the sun: its colour by the diffuse term, its specular weight by the specular, at a view space
// normal and point.
fn sun_light(state: Lighting, table: texture_3d<f32>, table_sampler: sampler, normal: vec3<f32>, position: vec3<f32>,
  slice: f32) -> vec4<f32> {
  let to_light: vec3<f32> = state.to_sun.xyz;
  let half: vec3<f32> = normalize(to_light - normalize(position));
  let terms: vec4<f32> = material_lookup(table, table_sampler, dot(to_light, normal), dot(half, normal), slice);

  return vec4<f32>(state.sun.rgb * terms.x, state.sun.w * terms.y);
}

// How much of the environment along the reflection a surface of a gloss reflects, by the hemisphere's lookup: the
// gloss on vanilla; on Anomaly none while dry, a sheen as it pours, brightest where the hemisphere lights least.
fn hmodel_gloss_weight(state: Lighting, hemisphere: vec4<f32>, gloss: f32) -> f32 {
  let rain: f32 = state.engine.y;

  return select(gloss, (gloss + rain * 0.25) * (1.0 - hemisphere.x) * (rain * 30.0), state.engine.x > 0.5);
}

// The hemisphere's lookup for a view, by the occlusion and by how far the reflection turns from it: its diffuse
// term, then the share of the environment along the reflection it passes. Directions in any one space.
fn hmodel_hemisphere(table: texture_3d<f32>, table_sampler: sampler, normal: vec3<f32>, to_point: vec3<f32>,
  slice: f32, occlusion: f32) -> vec4<f32> {
  return material_lookup(table, table_sampler, occlusion, 0.5 + 0.5 * dot(reflect(to_point, normal), to_point), slice);
}

// `hmodel` and `combine_1`: the hemisphere and ambient, times the screen's occlusion as `combine_1` multiplies
// `hdiffuse` and `hspecular` by `occ`, added to what the lights accumulated. Vanilla adds every reflection white;
// Anomaly's tints the lights' by their colour and multiplies the lit albedo by the hemisphere's. Directions are in world
// space; `albedo` carries the gloss in alpha.
fn hmodel(state: Lighting, table: texture_3d<f32>, table_sampler: sampler, cube_0: texture_cube<f32>,
  cube_1: texture_cube<f32>, cube_sampler: sampler, albedo: vec4<f32>, light: vec4<f32>, normal_world: vec3<f32>,
  to_point_world: vec3<f32>, slice: f32, occlusion: f32, visible: vec3<f32>) -> vec3<f32> {
  return hmodel_traced(state, table, table_sampler, cube_0, cube_1, cube_sampler, albedo, light, normal_world,
    to_point_world, slice, occlusion, visible, vec4<f32>(0.0));
}

// `hmodel` with what a traced reflection found standing in for the cube along the reflection: `traced` is the
// radiance met times how far it is trusted, then that trust, which takes as much of the cube away; nothing traced
// leaves the engine's own.
fn hmodel_traced(state: Lighting, table: texture_3d<f32>, table_sampler: sampler, cube_0: texture_cube<f32>,
  cube_1: texture_cube<f32>, cube_sampler: sampler, albedo: vec4<f32>, light: vec4<f32>, normal_world: vec3<f32>,
  to_point_world: vec3<f32>, slice: f32, occlusion: f32, visible: vec3<f32>, traced: vec4<f32>) -> vec3<f32> {
  let reflected: vec3<f32> = reflect(to_point_world, normal_world);
  let gloss: f32 = albedo.a;
  let hemisphere: vec4<f32> = hmodel_hemisphere(table, table_sampler, normal_world, to_point_world, slice, occlusion);
  let hemisphere_diffuse: vec3<f32> =
    (hemisphere_environment(state, cube_0, cube_1, cube_sampler, normal_world) * hemisphere.x + state.ambient.rgb)
    * visible;
  let is_extended: bool = state.engine.x > 0.5;
  // Anomaly's `hmodel` reads the reflection on the cube's faces, remapped short of the top one.
  let on_faces: vec3<f32> = reflected / max(max(abs(reflected.x), abs(reflected.y)), abs(reflected.z));
  let reflection: vec3<f32> = select(
    vec3<f32>(reflected.x, reflected.y * 2.0 - 1.0, reflected.z),
    vec3<f32>(on_faces.x, select(on_faces.y, on_faces.y * 2.0 - 1.0, on_faces.y < 0.999), on_faces.z),
    is_extended,
  );
  let weight: f32 = hmodel_gloss_weight(state, hemisphere, gloss);
  let reflected_environment: vec3<f32> =
    hemisphere_environment(state, cube_0, cube_1, cube_sampler, reflection) * (1.0 - saturate(traced.a)) + traced.rgb;
  let hemisphere_gloss: vec3<f32> = reflected_environment * hemisphere.y * weight * visible;
  // `C = D * light`: the lit albedo, and the gloss times what the lights reflect.
  let lit: vec3<f32> = albedo.rgb * (light.rgb + hemisphere_diffuse);
  let glossed: f32 = gloss * light.a;

  if (is_extended) {
    return lit + light.rgb * glossed + hemisphere_gloss * lit;
  }

  return lit + glossed + hemisphere_gloss;
}
