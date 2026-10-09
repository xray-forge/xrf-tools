#import "common/camera"
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
// distance along the view, at half the frame's size), the haze map (the sky as drawn, clouds and all, blurred by
// bearing and height), the indirect light (colour, then distance along the view, at half the frame's size) and the
// reflections (what each surface reflects, then one, at the size they are traced at).
#import "generated/frame/combine"

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

// The sky as the frame draws it behind a pixel: the sky, the sun's sprite on it, and the clouds over both.
fn sky_drawn(direction: vec3<f32>, toward: vec3<f32>, scale: f32) -> vec3<f32> {
  let below: vec3<f32> = sky_color(lighting, direction, scale) + sky_sun_sprite(lighting, toward);

  return sky_with_clouds(lighting, direction, below, scale);
}

// The sky's haze along a world direction: the sky as drawn there, clouds and all, blurred past any shape; below the
// horizon, the horizon's own, since nothing stands behind the ground.
fn sky_haze(direction: vec3<f32>) -> vec3<f32> {
  let lifted: vec3<f32> = normalize(vec3<f32>(direction.x, max(direction.y, 0.0), direction.z));
  let coordinates: vec2<f32> = sky_haze_coordinates(lifted);
  // Around the compass the map wraps; up it does not, so the zenith's row never blends with the nadir's.
  let rows: f32 = f32(textureDimensions(haze_map).y);

  return textureSampleLevel(
    haze_map,
    sky_repeat,
    vec2<f32>(coordinates.x, clamp(coordinates.y, 0.5 / rows, 1.0 - 0.5 / rows)),
    0.0,
  ).rgb;
}

// What the distance fades into: the sky as drawn behind it; with the haze, its haze where the cubes stand above the
// fold, so no cloud's shape lies over a far hill, and the rim as drawn under it, so the fade meets the sky exactly there.
fn sky_behind(direction: vec3<f32>, toward: vec3<f32>, scale: f32, is_hazed: bool) -> vec3<f32> {
  let shown: vec3<f32> = sky_drawn(direction, toward, scale);

  if (!is_hazed) {
    return shown;
  }

  return mix(shown, sky_haze(direction), sky_above_fold(lighting, direction));
}

// `hmodel` over the G-buffer's view space normal and point, its reflection of the environment apart.
fn shaded_terms(albedo: vec4<f32>, light: vec4<f32>, normal: vec3<f32>, position: vec3<f32>, slice: f32,
  occlusion: f32, visible: vec3<f32>) -> HmodelTerms {
  return hmodel_terms(lighting, material_lut, lut_sampler, sky_environment_0, sky_environment_1, sky_clamp, albedo,
    light, world_direction(normal), world_direction(normalize(position)), slice, occlusion, visible);
}

// The most a surface's reflection of the environment outshines the light it shows of its own, with the enhanced
// reflections: Anomaly's rain sheen rides on the lit albedo by thirty times the rain, and wet metal under a lamp at
// night turns into one.
const SHEEN_LIMIT: f32 = 4.0;
const SHEEN_LUMINANCE: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

// Metres the rain's ripples shift what the wet ground reflects, as the game's water shifts its own; the most pixels
// they shift it, and the metres over which the shift fades out, so a far puddle's reflection does not crawl.
const RIPPLE_SHIFT: f32 = 0.12;
const RIPPLE_SHIFT_MOST: f32 = 3.0;
const RIPPLE_SHIFT_REACH: vec2<f32> = vec2<f32>(10.0, 60.0);

// Where a pixel reads its reflection, shifted by the ripple across the ground at its view space point.
fn rippled_pixel(pixel: vec2<f32>, position: vec3<f32>, ripple: vec2<f32>) -> vec2<f32> {
  let across: vec3<f32> = (camera.view * vec4<f32>(ripple.x, 0.0, ripple.y, 0.0)).xyz * RIPPLE_SHIFT;
  let scale: vec2<f32> = vec2<f32>(camera.projection[0][0], -camera.projection[1][1]) * 0.5 * camera.viewport.xy;
  let shift: vec2<f32> = clamp(across.xy * scale / max(-position.z, 1.0), vec2<f32>(-RIPPLE_SHIFT_MOST),
    vec2<f32>(RIPPLE_SHIFT_MOST)) * smoothstep(RIPPLE_SHIFT_REACH.y, RIPPLE_SHIFT_REACH.x, -position.z);

  return clamp(pixel + shift, vec2<f32>(0.0), camera.viewport.xy - 1.0);
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
  let is_hazed: bool = is_sky_drawn && lighting.fog.z > 0.5;

  if (is_empty) {
    // `sky2.ps`: the sky's high part is the sky as drawn, within `def_hdr`.
    if (is_sky_drawn) {
      let sky: vec3<f32> = sky_drawn(direction, toward, scale);

      return combined(sky, sky / DEF_HDR);
    }

    if (is_fogged) {
      return combined(tonemap(lighting.fog_color.rgb, scale), tonemap_high(lighting.fog_color.rgb, scale));
    }

    return combined(backdrop_at(in.clip.xy), vec3<f32>(0.0));
  }

  let albedo: vec4<f32> = textureLoad(albedo_target, texel, 0);

  if (!is_lit) {
    return combined(albedo.rgb, vec3<f32>(0.0));
  }

  let material: vec4<f32> = textureLoad(material_target, texel, 0);
  let light: vec4<f32> = textureLoad(light_target, texel, 0);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  // How much of the hemisphere and ambient light reaches the point, as `combine_1` multiplies them by `occ`.
  let visible: f32 = select(1.0, upsampled_occlusion(occlusion_target, floor(in.clip.xy), -position.z),
    lighting.params.w > 0.5);
  let bounced: vec3<f32> = bounced_occlusion(visible, albedo.rgb, lighting.params.x);
  let occlusion: f32 = mix(1.0, material.x, camera.switches.z);
  let terms: HmodelTerms = shaded_terms(albedo, light, normal, position, material.z, occlusion, bounced);
  var shaded: vec3<f32> = terms.base;

  // The light the frame's surfaces bounce onto this one, lighting its albedo as the hemisphere does.
  if (lighting.indirect.x > 0.5) {
    shaded += albedo.rgb * upsampled_light(indirect_light, floor(in.clip.xy), -position.z);
  }

  // What the surface reflects: the traced reflection where there is one, shifted by the rain's ripples, the
  // environment `hmodel` reflects otherwise. The engine's reflection keeps its whole weight, the wet look as the engine
  // tuned it; the traced one only sharpens the slice of it a dielectric would reflect, never under a puddle, so where
  // nothing is traced the frame is the engine's. A puddle's clear coat of water reflects it over all of that by water's
  // Fresnel. All before the fog, which lies between the camera and both alike.
  let wet: vec4<f32> = textureLoad(wet_surface, texel, 0);
  var reflected: vec3<f32> = terms.environment;

  if (lighting.reflections.x > 0.5) {
    let traced: vec4<f32> = upsampled_reflection(reflections, depth_target, material_target, rippled_pixel(floor(in.clip.xy), position,
      wet.ba), -position.z, lighting.reflections.y);

    reflected = select(reflected, max(traced.rgb, vec3<f32>(0.0)), traced.a >= 0.0);
  }

  // The traced reflections' intensity scales the coat too; without them, water reflects as it is.
  let intensity: f32 = select(1.0, lighting.reflections.z, lighting.reflections.x > 0.5);
  // The puddle's ripple tilts what its water reflects across it a little, so it is never one even plate.
  let facing: f32 = saturate(dot(normal, -normalize(position)) +
    dot(wet.ba, world_direction(normalize(position)).xz) * 0.1);
  let puddle: f32 = saturate(wet.g);
  let roughness: f32 = reflection_roughness(albedo.a);
  let sheen: f32 = dot(terms.environment * terms.weight, SHEEN_LUMINANCE);
  let weight: vec3<f32> = select(terms.weight, terms.weight * min(1.0, SHEEN_LIMIT * dot(terms.lit, SHEEN_LUMINANCE) /
    max(sheen, 1e-4)), lighting.reflections.x > 0.5);
  let sharp: vec3<f32> = min(weight, vec3<f32>(dielectric_fresnel(facing, roughness, DIELECTRIC_F0) * intensity)) *
    (1.0 - puddle) * sharpened(roughness);
  let coat: f32 = saturate(wet.r * coat_fresnel(facing, coat_roughness(puddle)) * intensity);

  shaded += terms.environment * weight + (reflected - terms.environment) * sharp;
  shaded = mix(shaded, reflected, coat);

  // The engine fogs towards `fog_color` before the tonemap, then fades into the sky itself by the fog squared, both
  // parts alike (`skyblend` in either's alpha). The enhanced fog thickens the first and tints it below its height;
  // the sky's fade stays the distance fog's.
  let height: f32 = camera.position.y + (transpose(camera.view) * vec4<f32>(position, 0.0)).y;
  let thickened: f32 = select(0.0, fog_amount_at(lighting, position, height), is_fogged);
  let fogged: vec3<f32> = mix(shaded, fog_color_at(lighting, position, height), thickened);

  let finished: vec3<f32> = tonemap(fogged, scale);
  let high: vec3<f32> = tonemap_high(fogged, scale);

  if (!is_sky_drawn) {
    return combined(finished, high);
  }

  let sky: vec3<f32> = sky_behind(direction, toward, scale, is_hazed);

  return combined(mix(finished, sky, fog * fog), mix(high, sky / DEF_HDR, fog * fog));
}
