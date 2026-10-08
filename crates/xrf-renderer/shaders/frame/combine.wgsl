#import "common/camera"
#import "common/octahedral"
#import "common/lighting"
#import "common/hmodel"
#import "common/sky"
#import "common/fullscreen"
#import "common/occlusion"

// `hmodel` and `combine_1` over the light the frame accumulated, then the fog and tonemap of `combine_2`, faded into the
// weather's sky, into the viewport's scene; and `combine_1`'s and the sky's high part beside it, which the bloom is
// built from. Unlit, a surface is its raw albedo.

// The G-buffer and its light, the material table, the lighting and exposure, the ambient occlusion (visibility, then
// distance along the view, at half the frame's size), the haze map (the sky as drawn, clouds and all, blurred by
// bearing and height) and the indirect light (colour, then distance along the view, at half the frame's size).
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

// `hmodel` over the G-buffer's view space normal and point.
fn shaded_color(albedo: vec4<f32>, light: vec4<f32>, normal: vec3<f32>, position: vec3<f32>, slice: f32, occlusion: f32,
  visible: vec3<f32>) -> vec3<f32> {
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));

  return hmodel(lighting, material_lut, lut_sampler, sky_environment_0, sky_environment_1, sky_clamp, albedo, light,
    normalize(rotation * normal), normalize(rotation * normalize(position)), slice, occlusion, visible);
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
  var shaded: vec3<f32> = shaded_color(albedo, light, normal, position, material.z, occlusion, bounced);

  // The light the frame's surfaces bounce onto this one, lighting its albedo as the hemisphere does.
  if (lighting.indirect.x > 0.5) {
    shaded += albedo.rgb * upsampled_light(indirect_light, floor(in.clip.xy), -position.z);
  }

  // The engine fogs towards `fog_color` before the tonemap, then fades into the sky itself by the fog squared, both
  // parts alike (`skyblend` in either's alpha).
  let fogged: vec3<f32> = mix(shaded, lighting.fog_color.rgb, fog);
  let finished: vec3<f32> = tonemap(fogged, scale);
  let high: vec3<f32> = tonemap_high(fogged, scale);

  if (!is_sky_drawn) {
    return combined(finished, high);
  }

  let sky: vec3<f32> = sky_behind(direction, toward, scale, is_hazed);

  return combined(mix(finished, sky, fog * fog), mix(high, sky / DEF_HDR, fog * fog));
}
