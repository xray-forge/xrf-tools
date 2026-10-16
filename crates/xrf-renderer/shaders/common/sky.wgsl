#import "common/lighting"
#import "common/sky_box"
#import "common/sky_clouds"

// The weather's sky as the engine draws it: both keyframes' cubes through the half box `RenderSky` draws, their
// irradiance cubes `hmodel` samples, and the clouds' dome over them, each read with the frame's lighting.

// Both keyframes' cubes, irradiance cubes and clouds; linear and clamped, as a script binds a sky, and linear,
// repeating and trilinear, as the clouds tile; the environment cubes; and the sun's sprite the lens flare draws in the
// sky (`sun_texture`).
#import "generated/common/sky"
#import "common/math"

// The same along a world direction.
fn sky_above_fold(state: Lighting, direction: vec3<f32>) -> f32 {
  return sky_box_above_fold(sky_box_direction(direction, state.sky_params.x));
}

// The sky as `RenderSky` draws it along a world direction: the cubes through the half box, times `sky_color`, on the
// exposure's scale, written past the tonemap as vanilla's `sky2.ps` writes it, or through it as Anomaly's does.
fn sky_color(state: Lighting, direction: vec3<f32>, scale: f32) -> vec3<f32> {
  return sky_box_shown(direction, state, sky_cube_0, sky_cube_1, sky_clamp, scale);
}

// The clouds `RenderClouds` lays over the sky along a world direction (`sky_clouds_layer`), laid over the sky by their
// cover.
fn sky_with_clouds(state: Lighting, direction: vec3<f32>, below: vec3<f32>, scale: f32) -> vec3<f32> {
  return sky_clouds_over(state, direction, below, sky_clouds_0, sky_clouds_1, sky_repeat, scale);
}

// The sun's sprite along a view space direction, as `dxLensFlareRender` adds it to the sky before the clouds: a quad
// facing the view about where the sun stands on a plane ahead of the camera, its first texel to the right and up,
// times the sun's colour and alpha, `srcalpha, one`.
fn sky_sun_sprite(state: Lighting, toward: vec3<f32>) -> vec3<f32> {
  let sprite: vec4<f32> = state.sun_sprite;
  let sun_ahead: f32 = -state.to_sun.z;
  let ahead: f32 = -toward.z;

  if (sprite.w <= 0.0 || sun_ahead <= 0.01 || ahead <= 0.0) {
    return vec3<f32>(0.0);
  }

  let offset: vec2<f32> = (toward.xy / ahead - state.to_sun.xy / sun_ahead) / sprite.w;

  if (any(abs(offset) >= vec2<f32>(1.0))) {
    return vec3<f32>(0.0);
  }

  let texel: vec4<f32> = textureSampleLevel(sky_sun, sky_clamp, (1.0 - offset) * 0.5, 0.0);

  return texel.rgb * texel.a * sprite.rgb;
}

// The sky with its clouds along a world direction, as the frame shows it.
fn sky_shown(state: Lighting, direction: vec3<f32>, scale: f32) -> vec3<f32> {
  return sky_with_clouds(state, direction, sky_color(state, direction, scale), scale);
}

// Where the haze map holds a unit world direction: the bearing across, the height from the nadir up.
fn sky_haze_coordinates(direction: vec3<f32>) -> vec2<f32> {
  return vec2<f32>(
    atan2(direction.x, direction.z) / (PI * 2.0) + 0.5,
    asin(clamp(direction.y, -1.0, 1.0)) / PI + 0.5,
  );
}

// The sky as the frame draws it behind a pixel: the sky, the sun's sprite on it, and the clouds over both.
fn sky_drawn(state: Lighting, direction: vec3<f32>, toward: vec3<f32>, scale: f32) -> vec3<f32> {
  let below: vec3<f32> = sky_color(state, direction, scale) + sky_sun_sprite(state, toward);

  return sky_with_clouds(state, direction, below, scale);
}

// The sky's haze along a world direction, from the haze map `haze`: the sky as drawn there, clouds and all, blurred past
// any shape; below the horizon, the horizon's own, since nothing stands behind the ground.
fn sky_haze(haze: texture_2d<f32>, direction: vec3<f32>) -> vec3<f32> {
  let lifted: vec3<f32> = normalize(vec3<f32>(direction.x, max(direction.y, 0.0), direction.z));
  let coordinates: vec2<f32> = sky_haze_coordinates(lifted);
  // Around the compass the map wraps; up it does not, so the zenith's row never blends with the nadir's.
  let rows: f32 = f32(textureDimensions(haze).y);

  return textureSampleLevel(
    haze,
    sky_repeat,
    vec2<f32>(coordinates.x, clamp(coordinates.y, 0.5 / rows, 1.0 - 0.5 / rows)),
    0.0,
  ).rgb;
}

// The sky the far ground stands against along a world direction where the enhanced fog fades it only past a share:
// as drawn, but for the horizon over the fold in place of the rim under it (`sky_box_horizon_lit`), the clouds fading
// out at the horizon as they do; with the haze (`is_hazed`), the horizon's haze under the fold.
fn sky_over_ground(state: Lighting, haze: texture_2d<f32>, direction: vec3<f32>, toward: vec3<f32>, scale: f32,
  is_hazed: bool) -> vec3<f32> {
  let lit: vec3<f32> = sky_box_horizon_lit(direction, state, sky_cube_0, sky_cube_1, sky_clamp);
  let below: vec3<f32> = sky_box_scaled(lit, state, scale) + sky_sun_sprite(state, toward);
  let shown: vec3<f32> = sky_with_clouds(state, direction, below, scale);

  if (!is_hazed) {
    return shown;
  }

  return mix(sky_haze(haze, direction), shown, sky_above_fold(state, direction));
}

// What the distance fades into: the sky as drawn behind it; with the haze (`is_hazed`), its haze where the cubes stand
// above the fold, so no cloud's shape lies over a far hill, and the rim as drawn under it, so the fade meets the sky
// exactly there. Where the enhanced fog fades only past a share, the sky the far ground stands against, or the haze all
// round with the haze.
fn sky_behind(state: Lighting, haze: texture_2d<f32>, direction: vec3<f32>, toward: vec3<f32>, scale: f32,
  is_hazed: bool) -> vec3<f32> {
  if (state.sky_fade.w > 0.5) {
    if (is_hazed) {
      return sky_haze(haze, direction);
    }

    return sky_over_ground(state, haze, direction, toward, scale, false);
  }

  let shown: vec3<f32> = sky_drawn(state, direction, toward, scale);

  if (!is_hazed) {
    return shown;
  }

  return mix(shown, sky_haze(haze, direction), sky_above_fold(state, direction));
}
