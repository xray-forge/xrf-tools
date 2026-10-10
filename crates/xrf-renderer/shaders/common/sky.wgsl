#import "common/lighting"
#import "common/sky_box"
#import "common/sky_clouds"

// The weather's sky as the engine draws it: both keyframes' cubes through the half box `RenderSky` draws, their
// irradiance cubes `hmodel` samples, and the clouds' dome over them, each read with the frame's lighting.

// Both keyframes' cubes, irradiance cubes and clouds; linear and clamped, as a script binds a sky, and linear,
// repeating and trilinear, as the clouds tile; the environment cubes; and the sun's sprite the lens flare draws in the
// sky (`sun_texture`).
#import "generated/common/sky"

const SKY_PI: f32 = 3.14159265;

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
    atan2(direction.x, direction.z) / (SKY_PI * 2.0) + 0.5,
    asin(clamp(direction.y, -1.0, 1.0)) / SKY_PI + 0.5,
  );
}
