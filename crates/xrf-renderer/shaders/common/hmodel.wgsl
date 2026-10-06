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

// The direction `hmodel` reads the environment along for a reflected one: vanilla's lifted, Anomaly's on the cube's
// faces, remapped short of the top one.
fn hmodel_reflection_lookup(state: Lighting, reflected: vec3<f32>) -> vec3<f32> {
  let on_faces: vec3<f32> = reflected / max(max(abs(reflected.x), abs(reflected.y)), abs(reflected.z));

  return select(
    vec3<f32>(reflected.x, reflected.y * 2.0 - 1.0, reflected.z),
    vec3<f32>(on_faces.x, select(on_faces.y, on_faces.y * 2.0 - 1.0, on_faces.y < 0.999), on_faces.z),
    state.engine.x > 0.5,
  );
}

// `hmodel` and `combine_1` in two parts: what the surface shows without its reflection of the environment, the
// environment it reflects, and what that is weighed by; their sum is `hmodel`. The reflections pass sharpens the
// reflected part where it traces one.
struct HmodelTerms {
  base: vec3<f32>,
  environment: vec3<f32>,
  weight: vec3<f32>,
  // The lit albedo alone, what the surface shows of the light that reaches it.
  lit: vec3<f32>,
};

// How much of the environment a surface reflects, before the occlusion and the lit colour Anomaly tints it by: the
// hemisphere's reflection term by the gloss, or by Anomaly's rain. `reflected` and `to_point_world` in world space.
fn hmodel_environment_weight(state: Lighting, table: texture_3d<f32>, table_sampler: sampler, gloss: f32,
  occlusion: f32, reflected: vec3<f32>, to_point_world: vec3<f32>, slice: f32) -> f32 {
  let hemisphere: vec4<f32> = material_lookup(table, table_sampler, occlusion, 0.5 + 0.5 * dot(reflected, to_point_world),
    slice);

  return hemisphere.y * hmodel_gloss_weight(state, gloss, hemisphere.x);
}

// The gloss a reflection is weighed by: vanilla's own; Anomaly's none while dry, a sheen as it pours, brightest where
// the hemisphere lights least.
fn hmodel_gloss_weight(state: Lighting, gloss: f32, hemisphere: f32) -> f32 {
  let rain: f32 = state.engine.y;

  return select(gloss, (gloss + rain * 0.25) * (1.0 - hemisphere) * (rain * 30.0), state.engine.x > 0.5);
}

// The environment a surface reflects along a world direction, as `hmodel` looks it up.
fn hmodel_reflected_environment(state: Lighting, cube_0: texture_cube<f32>, cube_1: texture_cube<f32>,
  cube_sampler: sampler, reflected: vec3<f32>) -> vec3<f32> {
  return hemisphere_environment(state, cube_0, cube_1, cube_sampler, hmodel_reflection_lookup(state, reflected));
}

// The hemisphere and ambient, times the screen's occlusion as `combine_1` multiplies `hdiffuse` and `hspecular` by
// `occ`, added to what the lights accumulated; the environment's reflection apart. Vanilla adds every reflection white;
// Anomaly's tints the lights' by their colour and multiplies the lit albedo by the hemisphere's. Directions are in world
// space; `albedo` carries the gloss in alpha.
fn hmodel_terms(state: Lighting, table: texture_3d<f32>, table_sampler: sampler, cube_0: texture_cube<f32>,
  cube_1: texture_cube<f32>, cube_sampler: sampler, albedo: vec4<f32>, light: vec4<f32>, normal_world: vec3<f32>,
  to_point_world: vec3<f32>, slice: f32, occlusion: f32, visible: vec3<f32>) -> HmodelTerms {
  let reflected: vec3<f32> = reflect(to_point_world, normal_world);
  let gloss: f32 = albedo.a;
  // The hemisphere looked up by occlusion and by how far the reflection turns from the view.
  let hemisphere: vec4<f32> = material_lookup(table, table_sampler, occlusion, 0.5 + 0.5 * dot(reflected, to_point_world),
    slice);
  let hemisphere_diffuse: vec3<f32> =
    (hemisphere_environment(state, cube_0, cube_1, cube_sampler, normal_world) * hemisphere.x + state.ambient.rgb)
    * visible;
  let is_extended: bool = state.engine.x > 0.5;
  let weight: vec3<f32> = hemisphere.y * hmodel_gloss_weight(state, gloss, hemisphere.x) * visible;
  let environment: vec3<f32> = hmodel_reflected_environment(state, cube_0, cube_1, cube_sampler, reflected);
  // `C = D * light`: the lit albedo, and the gloss times what the lights reflect.
  let lit: vec3<f32> = albedo.rgb * (light.rgb + hemisphere_diffuse);
  let glossed: f32 = gloss * light.a;

  if (is_extended) {
    return HmodelTerms(lit + light.rgb * glossed, environment, weight * lit, lit);
  }

  return HmodelTerms(lit + glossed, environment, weight, lit);
}

// `hmodel` and `combine_1` whole.
fn hmodel(state: Lighting, table: texture_3d<f32>, table_sampler: sampler, cube_0: texture_cube<f32>,
  cube_1: texture_cube<f32>, cube_sampler: sampler, albedo: vec4<f32>, light: vec4<f32>, normal_world: vec3<f32>,
  to_point_world: vec3<f32>, slice: f32, occlusion: f32, visible: vec3<f32>) -> vec3<f32> {
  let terms: HmodelTerms = hmodel_terms(state, table, table_sampler, cube_0, cube_1, cube_sampler, albedo, light,
    normal_world, to_point_world, slice, occlusion, visible);

  return terms.base + terms.environment * terms.weight;
}
