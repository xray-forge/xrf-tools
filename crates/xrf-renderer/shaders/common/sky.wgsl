#import "common/lighting"
#import "common/sky_box"

// The weather's sky as the engine draws it: both keyframes' cubes through the half box `RenderSky` draws, their
// irradiance cubes `hmodel` samples, and the clouds' dome over them, each read with the frame's lighting.

@group(2) @binding(0) var sky_cube_0: texture_cube<f32>;
@group(2) @binding(1) var sky_cube_1: texture_cube<f32>;
@group(2) @binding(2) var sky_environment_0: texture_cube<f32>;
@group(2) @binding(3) var sky_environment_1: texture_cube<f32>;
@group(2) @binding(4) var sky_clouds_0: texture_2d<f32>;
@group(2) @binding(5) var sky_clouds_1: texture_2d<f32>;
// Linear and clamped, as a script binds a sky; linear, repeating and trilinear, as the clouds tile.
@group(2) @binding(6) var sky_clamp: sampler;
@group(2) @binding(7) var sky_repeat: sampler;
// The sun's sprite the lens flare draws in the sky (`sun_texture`).
@group(2) @binding(9) var sky_sun: texture_2d<f32>;

const SKY_PI: f32 = 3.14159265;

// `sky2.vs` and `sky2.ps`: the colour doubled on the tonemap's scale, then the blended cubes a third of it.
const SKY_FACTOR: f32 = 2.0 * 0.33;

// Anomaly's `sky2.vs`: the colour 1.7 times the tonemap's scale, which its `sky2.ps` then takes through the curve.
const CURVED_SKY_FACTOR: f32 = 1.7;

// How high above the fold the rim is still averaged, fading back to single taps by then, as a height over the side.
const HAZE_TOP: f32 = 0.04;

// Taps the rim is averaged over, and the turn between two, two texels of a 512 face: a box over some twenty-four.
const RIM_TAPS: i32 = 13;
const RIM_TAP_ANGLE: f32 = 0.35 * SKY_PI / 180.0;

// `mScale.scale(10, 0.4f, 10)`: the dome `RenderClouds` draws, wide and flat around the camera.
const DOME_WIDTH: f32 = 10.0;
const DOME_HEIGHT: f32 = 0.4;

// `CLOUD_TILE0`, `CLOUD_SPEED0`, `CLOUD_TILE1` and `CLOUD_SPEED1` (`shared/cloudconfig.h`).
const CLOUD_TILE_0: f32 = 0.7;
const CLOUD_SPEED_0: f32 = 2.0 * 0.05;
const CLOUD_TILE_1: f32 = 2.8;
const CLOUD_SPEED_1: f32 = 2.0 * 0.025;

// How sharply the clouds fade to the horizon, `pow(v.p.y, 25)` in `clouds.vs`.
const CLOUD_FADE: f32 = 25.0;

// The two layers' drift, `setHP(PI_DIV_4, 0)` and `setHP(PI_DIV_4 + PI_DIV_8, 0)` in `x` and `z`, through the byte
// `RenderClouds` packs a direction into.
const CLOUD_WIND_0: vec2<f32> = vec2<f32>(-0.7098039, 0.7019608);
const CLOUD_WIND_1: vec2<f32> = vec2<f32>(-0.9294118, 0.3803922);

// Both skies at an engine cube coordinate, blended as `L_ambient.w` blends them, at their top level: the GPU picks a
// cube's faces as Direct3D does, so the engine's coordinate is sampled as it is.
fn sky_blended_cubes(state: Lighting, lookup: vec3<f32>) -> vec3<f32> {
  return mix(
    textureSampleLevel(sky_cube_0, sky_clamp, lookup, 0.0).rgb,
    textureSampleLevel(sky_cube_1, sky_clamp, lookup, 0.0).rgb,
    state.sky.w,
  );
}

// The cubes around a box direction under the fold, averaged around the compass so the rim's texels do not show.
fn sky_rim(state: Lighting, box: vec3<f32>) -> vec3<f32> {
  var sum: vec3<f32> = vec3<f32>(0.0);

  for (var tap: i32 = 0; tap < RIM_TAPS; tap++) {
    let angle: f32 = (f32(tap) - f32(RIM_TAPS - 1) * 0.5) * RIM_TAP_ANGLE;
    let c: f32 = cos(angle);
    let s: f32 = sin(angle);
    let turned: vec3<f32> = vec3<f32>(box.x * c - box.z * s, box.y, box.x * s + box.z * c);

    sum += sky_blended_cubes(state, sky_box_lookup(turned));
  }

  return sum / f32(RIM_TAPS);
}

// How much of the sky along a box direction is the cubes above the fold rather than the rim averaged under it.
fn sky_box_above_fold(box: vec3<f32>) -> f32 {
  return smoothstep(BOX_HORIZON, HAZE_TOP, box.y / max(abs(box.x), abs(box.z)));
}

// The same along a world direction.
fn sky_above_fold(state: Lighting, direction: vec3<f32>) -> f32 {
  return sky_box_above_fold(sky_box_direction(direction, state.sky_params.x));
}

// The sky as `RenderSky` draws it along a world direction: the cubes through the half box, times `sky_color`, on the
// exposure's scale, written past the tonemap as vanilla's `sky2.ps` writes it, or through it as Anomaly's does.
fn sky_color(state: Lighting, direction: vec3<f32>, scale: f32) -> vec3<f32> {
  let box: vec3<f32> = sky_box_direction(direction, state.sky_params.x);
  let above: f32 = sky_box_above_fold(box);
  var color: vec3<f32> = sky_blended_cubes(state, sky_box_lookup(box));

  // Below the fold every pixel of a column reads one texel of the bottom rim, which draws its block noise as streaks
  // the height of the band; averaged around the compass there, it keeps the haze and loses the texels.
  if (above < 1.0) {
    color = mix(sky_rim(state, box), color, above);
  }

  let lit: vec3<f32> = color * state.sky.rgb;

  if (state.engine.x > 0.5) {
    return tonemap(lit * CURVED_SKY_FACTOR, scale);
  }

  return lit * scale * SKY_FACTOR;
}

// The clouds `RenderClouds` lays over the sky along a world direction (`clouds.vs`, `clouds.ps`): the dome met along
// the view, two scrolling layers of both keyframes' textures tiled over it, times the clouds' colour, faded to the
// horizon and laid over the sky by their cover.
fn sky_with_clouds(state: Lighting, direction: vec3<f32>, below: vec3<f32>, scale: f32) -> vec3<f32> {
  let box: vec3<f32> = sky_box_direction(direction, state.sky_params.y);
  // The unit hemisphere point the dome's scale puts along the direction: `v.p`.
  let dome: vec3<f32> = normalize(vec3<f32>(box.x / DOME_WIDTH, box.y / DOME_HEIGHT, box.z / DOME_WIDTH));
  // `timers.z`, the time a tenth as fast.
  let drift: f32 = state.sky_params.z * 0.1;
  let first: vec2<f32> = dome.xz * CLOUD_TILE_0 + CLOUD_WIND_0 * drift * CLOUD_SPEED_0;
  let second: vec2<f32> = dome.xz * CLOUD_TILE_1 + CLOUD_WIND_1 * drift * CLOUD_SPEED_1;
  let texel: vec4<f32> = mix(
    textureSampleLevel(sky_clouds_0, sky_repeat, first, 0.0) + textureSampleLevel(sky_clouds_0, sky_repeat, second, 0.0),
    textureSampleLevel(sky_clouds_1, sky_repeat, first, 0.0) + textureSampleLevel(sky_clouds_1, sky_repeat, second, 0.0),
    state.sky.w,
  );
  let color: vec4<f32> = state.clouds * texel;
  let cover: f32 = saturate(color.a * pow(max(dome.y, 0.0), CLOUD_FADE));

  return mix(below, color.rgb * scale, cover);
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
