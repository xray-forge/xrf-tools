#import "common/camera"
#import "common/scene_light"
#import "common/octahedral"
#import "common/lighting"
#import "common/hmodel"
#import "common/sky"
#import "common/fullscreen"
#import "common/occlusion"
#import "common/reflection_trace"

// `hmodel` and `combine_1` over the light the frame accumulated, then the fog and tonemap of `combine_2`, faded into the
// weather's sky, into the viewport's scene; and `combine_1`'s and the sky's high part beside it, which the bloom is
// built from. Unlit, a surface is its raw albedo.

// The G-buffer and its light, the material table, the lighting and exposure, the ambient occlusion (visibility, then
// distance along the view, at half the frame's size), the sky behind the fog (at half the frame's size), the indirect
// light (colour, then distance along the view, at half the frame's size) and the
// reflections (what each surface reflects, then one, at the size they are traced at).
#import "generated/frame/combine"

// The fog's square below which the sky behind changes no pixel by half an eight-bit step: 1 / 512.
const UNSEEN_SKY: f32 = 0.001953125;

// What shows where nothing was drawn and neither the sky nor the fog is: the level viewer's backdrop, #202428.
const BACKDROP: vec3<f32> = vec3<f32>(32.0, 36.0, 40.0) / 255.0;

// The view space direction through a pixel of the viewport.
fn pixel_toward(pixel: vec2<f32>) -> vec3<f32> {
  return normalize(camera_view_position(pixel, 0.5));
}

// The same direction in the world.
fn world_direction(toward: vec3<f32>) -> vec3<f32> {
  return normalize((transpose(camera.view) * vec4<f32>(toward, 0.0)).xyz);
}

// A view space point's offset from the camera in the world's frame, metres.
fn world_offset(position: vec3<f32>) -> vec3<f32> {
  return (transpose(camera.view) * vec4<f32>(position, 0.0)).xyz;
}

// `hmodel` over the G-buffer's view space normal and point, its reflection of the environment apart.
fn shaded_terms(albedo: vec4<f32>, light: vec4<f32>, normal: vec3<f32>, position: vec3<f32>, slice: f32,
  occlusion: f32, visible: vec3<f32>) -> HmodelTerms {
  return hmodel_terms(lighting, material_lut, lut_sampler, sky_environment_0, sky_environment_1, sky_clamp, albedo,
    light, world_direction(normal), world_direction(normalize(position)), slice, occlusion, visible);
}

// What shows at a pixel where nothing was drawn: the backdrop, or its checkerboard with a second colour.
fn backdrop_at(pixel: vec2<f32>) -> vec3<f32> {
  let backdrop: vec3<f32> = select(BACKDROP, camera.backdrop.rgb, camera.backdrop.w > 0.5);
  let squares: vec4<f32> = camera.backdrop_squares;

  if (squares.w <= 0.0) {
    return backdrop;
  }

  let cell: vec2<i32> = vec2<i32>(floor(pixel / squares.w));

  return select(backdrop, squares.rgb, ((cell.x + cell.y) & 1) == 1);
}

// The scene, tonemapped, and its high part (`tonemap`'s `low` and `high`).
struct CombineOutput {
  @location(0) low: vec4<f32>,
  @location(1) high: vec4<f32>,
};

// What a pixel comes to, tonemapped and past the tonemap.
fn combined(low: vec3<f32>, high: vec3<f32>) -> CombineOutput {
  return CombineOutput(vec4<f32>(low, 1.0), vec4<f32>(high, 1.0));
}

// How far the ground seen through a puddle's water shifts, metres a unit of its ripples' tilt, and the puddle over which
// the shift comes in.
const REFRACTION_SHIFT: f32 = 0.25;
const REFRACTION_FROM: vec2<f32> = vec2<f32>(0.6, 0.8);

// The ground seen through a puddle's water, its colour read where the ripples bend the view to: the ripples' tilt off
// the level up, `REFRACTION_SHIFT` metres a unit of it on the ground, faded in as the puddle deepens into water as
// Screen Space Shaders' refraction does; the pixel's own where the bent texel is no water.
fn refracted_albedo(texel: vec2<i32>, albedo: vec4<f32>, normal: vec3<f32>, distance: f32) -> vec3<f32> {
  let puddle: f32 = textureLoad(wet_surface, texel, 0).g;
  let strength: f32 = smoothstep(REFRACTION_FROM.x, REFRACTION_FROM.y, puddle);

  if (strength <= 0.0) {
    return albedo.rgb;
  }

  let tilt: vec3<f32> = normal - (camera.view * vec4<f32>(0.0, 1.0, 0.0, 0.0)).xyz;
  let pixels: f32 = REFRACTION_SHIFT * camera.projection[1][1] * camera.viewport.y / (2.0 * max(distance, 0.1));
  let size: vec2<i32> = vec2<i32>(textureDimensions(albedo_target));
  let at: vec2<i32> = clamp(texel + vec2<i32>(round(vec2<f32>(tilt.x, -tilt.y) * pixels * strength)), vec2<i32>(0),
    size - 1);

  if (textureLoad(wet_surface, at, 0).g < REFRACTION_FROM.x) {
    return albedo.rgb;
  }

  return textureLoad(albedo_target, at, 0).rgb;
}

@fragment
fn fs_combine(in: FullscreenVarying) -> CombineOutput {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);
  let is_lit: bool = lighting.params.y > 0.5;
  let scale: f32 = frame_scale(lighting, exposure);
  let is_fogged: bool = is_lit && lighting.fog_color.w > 0.5;
  // A level's frame draws its sky behind everything, so an empty pixel is the sky there instead.
  let is_sky_drawn: bool = is_lit && lighting.fog.w > 0.5;
  let toward: vec3<f32> = pixel_toward(in.clip.xy);
  let direction: vec3<f32> = world_direction(toward);
  let position: vec3<f32> = camera_view_position(in.clip.xy, max(depth, 1e-7));
  let fog: f32 = select(0.0, fog_amount(lighting, position), is_fogged);
  // The far plane ends where fog is total: past it is what anything there would have come to, the sky or the fog.
  let is_empty: bool = depth <= 0.0 || fog >= 1.0;

  if (is_empty) {
    // `sky2.ps`: the sky's high part is the sky as drawn, within `def_hdr`; where the enhanced fog fades only past a
    // share, the sky the far ground fades into, so where the fog ends it meets the faded ground.
    if (is_sky_drawn) {
      var sky: vec3<f32> = sky_drawn(lighting, direction, toward, scale);

      if (lighting.sky_fade.w > 0.5) {
        sky = sky_over_ground(lighting, haze_map, direction, toward, scale, lighting.fog.z > 0.5);
      }

      return combined(scene_from_shown(sky), sky / DEF_HDR);
    }

    if (is_fogged) {
      return combined(scene_from_light(lighting.fog_color.rgb, scale), tonemap_high(lighting.fog_color.rgb, scale));
    }

    return combined(scene_from_shown(backdrop_at(in.clip.xy)), vec3<f32>(0.0));
  }

  var albedo: vec4<f32> = textureLoad(albedo_target, texel, 0);

  if (!is_lit) {
    return combined(scene_from_shown(albedo.rgb), vec3<f32>(0.0));
  }

  let material: vec4<f32> = textureLoad(material_target, texel, 0);
  let light: vec4<f32> = textureLoad(light_target, texel, 0);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);

  albedo = vec4<f32>(refracted_albedo(texel, albedo, normal, -position.z), albedo.a);

  // How much of the hemisphere and ambient light reaches the point, as `combine_1` multiplies them by `occ`.
  let visible: f32 = select(1.0, upsampled_occlusion(occlusion_target, floor(in.clip.xy), -position.z),
    lighting.params.w > 0.5);
  let bounced: vec3<f32> = bounced_occlusion(visible, albedo.rgb, lighting.params.x);
  let occlusion: f32 = mix(1.0, material.x, camera.switches.z);
  // The lights' specular, the engine's model, sees the gloss the engine's own wetting gave, not what the reflections'
  // wet reach added for the trace.
  let reach: f32 = textureLoad(wet_reach, min(texel, vec2<i32>(textureDimensions(wet_reach)) - 1), 0).r;
  let terms: HmodelTerms = shaded_terms(vec4<f32>(albedo.rgb, max(albedo.a - reach, 0.0)), light, normal, position,
    material.z, occlusion, bounced);
  var shaded: vec3<f32> = terms.base;

  // The light the frame's surfaces bounce onto this one, lighting its albedo as the hemisphere does.
  if (lighting.indirect.x > 0.5) {
    shaded += albedo.rgb * upsampled_light(indirect_light, floor(in.clip.xy), -position.z);
  }

  // What the surface reflects, before the fog, which lies between the camera and it alike. Engine: the environment
  // `hmodel` reflects by the engine's own weight. Enhanced: a physically based specular in its place, the split-sum
  // environment BRDF over the surface's reflectance head on (a dielectric's, a metal's by its gloss, water's under a
  // puddle's coat) and roughness (its gloss's, widened by the ripples the wet patch filtered away), times the
  // intensity, over the traced reflection where there is one and the prefiltered sky, seen as the hemisphere lets it,
  // where there is none.
  let wet: vec4<f32> = textureLoad(wet_surface, texel, 0);

  if (lighting.reflections.x > 0.5) {
    let facing: f32 = saturate(dot(normal, -normalize(position)));
    let roughness: f32 = traced_roughness(albedo.a, wet.a + surface_variance(normal_target, depth_target, texel, wet.g));
    let head_on: vec3<f32> = reflectance_head_on(albedo.rgb, albedo.a, wet.r, material.z);
    let brdf: vec3<f32> = environment_brdf(head_on, roughness, facing) * lighting.reflections.z;
    // The full-size history the reflections were resolved into; a texel of nothing where none is traced.
    let traced: vec4<f32> = textureLoad(reflections, min(texel, vec2<i32>(textureDimensions(reflections)) - 1), 0);
    // The traced reflection where there is one, faded into the prefiltered sky towards the trace's roughness cut.
    let kept: f32 = select(0.0, smoothstep(TRACED_ROUGHNESS_FADE.y, TRACED_ROUGHNESS_FADE.x, roughness),
      traced.a >= 0.0);
    var radiance: vec3<f32> = max(traced.rgb, vec3<f32>(0.0));

    if (kept < 1.0) {
      let reflected: vec3<f32> = world_direction(reflect(normalize(position), normal));
      let aimed: vec3<f32> = surroundings_direction(sky_prefiltered, sky_clamp, world_offset(position), reflected);
      let sky: vec3<f32> = prefiltered_sky(sky_prefiltered, sky_clamp, aimed, roughness) *
        specular_occlusion(facing, occlusion * visible, roughness);

      radiance = mix(sky, radiance, kept);
    }

    shaded += radiance * brdf;

    // The sun on a wet surface's water film, which the engine's specular leaves out with the gloss the wetness added:
    // its disc in the film's lobe, as far as a puddle's coat or the wet reach covers the surface, shadowed as the sun's
    // pass found it.
    let film: f32 = saturate(wet.r + reach);

    if (film > 0.0) {
      let seen: f32 = textureLoad(sun_seen, texel, 0).x;

      shaded += lighting.sun.rgb * sun_glint(normal, -normalize(position), lighting.to_sun.xyz, roughness, head_on) *
        film * seen * lighting.reflections.z;
    }
  } else {
    shaded += terms.environment * terms.weight;
  }

  // The engine fogs towards `fog_color` before the tonemap, then fades into the sky itself by the fog squared, both
  // parts alike (`skyblend` in either's alpha). The enhanced fog thickens the first and tints it below its height;
  // the sky's fade stays the distance fog's, by its square or, where the enhanced fog asks, only past a share of it, so
  // the middle distance is veiled rather than shown through to the sky.
  let height: f32 = camera.position.y + (transpose(camera.view) * vec4<f32>(position, 0.0)).y;
  let thickened: f32 = select(0.0, fog_amount_at(lighting, position, height), is_fogged);
  let fogged: vec3<f32> = mix(shaded, fog_color_at(lighting, position, height), thickened);

  let finished: vec3<f32> = scene_from_light(fogged, scale);
  let high: vec3<f32> = tonemap_high(fogged, scale);

  let behind_share: f32 = select(fog * fog, smoothstep(lighting.sky_fade.x, 1.0, fog), lighting.sky_fade.w > 0.5);

  // Where the fog is too thin for the sky behind, mixed in by its share, to move a pixel by half an eight-bit step
  // whatever the two are, the sky is not built at all.
  if (!is_sky_drawn || behind_share < UNSEEN_SKY) {
    return combined(finished, high);
  }

  // At half the frame's size and read across it, its texels weighed by whether they were drawn: the sky behind is
  // soft, and only the fog's square shows it.
  let behind: vec4<f32> = textureSampleLevel(sky_behind_map, sky_clamp, in.clip.xy / camera.viewport.xy, 0.0);
  let sky: vec3<f32> = behind.rgb / max(behind.a, 1e-4);

  return combined(mix(finished, scene_from_shown(sky), behind_share), mix(high, sky / DEF_HDR, behind_share));
}
