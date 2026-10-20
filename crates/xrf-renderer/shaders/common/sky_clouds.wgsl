#import "common/lighting"
#import "common/sky_box"

// The sky as the engine shows it along a world direction, with the clouds `RenderClouds` lays over it (`clouds.vs`,
// `clouds.ps`), and the same as light: what a reflection of the sky adds to a surface, in the units the surface's own
// light is in. Every pass drawing or reflecting the sky reads these through its own bindings, so the cube a reflection
// reads where it meets nothing shows the sky the frame shows, clouds and all: in an overcast rain the clouds are the
// bright of the sky.

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

// The sky as `RenderSky` draws it along a world direction: the cubes through the half box, times `sky_color`, on the
// exposure's scale, written past the tonemap as vanilla's `sky2.ps` writes it, or through it as Anomaly's does.
fn sky_box_shown(direction: vec3<f32>, state: Lighting, cube_0: texture_cube<f32>, cube_1: texture_cube<f32>,
  cube_sampler: sampler, scale: f32) -> vec3<f32> {
  return sky_box_scaled(sky_box_lit(direction, state, cube_0, cube_1, cube_sampler), state, scale);
}

// The sky lit, on the exposure's scale, written past the tonemap or through it as `sky_box_shown` writes it.
fn sky_box_scaled(lit: vec3<f32>, state: Lighting, scale: f32) -> vec3<f32> {
  if (state.engine.x > 0.5) {
    return tonemap(lit * CURVED_SKY_FACTOR, scale);
  }

  return lit * scale * SKY_FACTOR;
}

// The clouds along a world direction: the dome met along the view, two scrolling layers of both keyframes' textures
// tiled over it, times the clouds' colour, in `rgb`; their cover, faded to the horizon, in `a`.
fn sky_clouds_layer(state: Lighting, direction: vec3<f32>, clouds_0: texture_2d<f32>, clouds_1: texture_2d<f32>,
  repeat: sampler) -> vec4<f32> {
  let box: vec3<f32> = sky_box_direction(direction, state.sky_params.y);
  // The unit hemisphere point the dome's scale puts along the direction: `v.p`.
  let dome: vec3<f32> = normalize(vec3<f32>(box.x / DOME_WIDTH, box.y / DOME_HEIGHT, box.z / DOME_WIDTH));
  // `timers.z`, the time a tenth as fast.
  let drift: f32 = state.sky_params.z * 0.1;
  let first: vec2<f32> = dome.xz * CLOUD_TILE_0 + CLOUD_WIND_0 * drift * CLOUD_SPEED_0;
  let second: vec2<f32> = dome.xz * CLOUD_TILE_1 + CLOUD_WIND_1 * drift * CLOUD_SPEED_1;
  let texel: vec4<f32> = mix(
    textureSampleLevel(clouds_0, repeat, first, 0.0) + textureSampleLevel(clouds_0, repeat, second, 0.0),
    textureSampleLevel(clouds_1, repeat, first, 0.0) + textureSampleLevel(clouds_1, repeat, second, 0.0),
    state.sky.w,
  );
  let color: vec4<f32> = state.clouds * texel;

  return vec4<f32>(color.rgb, saturate(color.a * pow(max(dome.y, 0.0), CLOUD_FADE)));
}

// The sky with its clouds as the frame shows it, laid by their cover over the sky `below` them.
fn sky_clouds_over(state: Lighting, direction: vec3<f32>, below: vec3<f32>, clouds_0: texture_2d<f32>,
  clouds_1: texture_2d<f32>, repeat: sampler, scale: f32) -> vec3<f32> {
  let clouds: vec4<f32> = sky_clouds_layer(state, direction, clouds_0, clouds_1, repeat);

  return mix(below, clouds.rgb * scale, clouds.a);
}

// The sky's light along a renderer-space direction in the frame's own units: the sky as the frame shows it, clouds
// and all, taken back through the tonemap, so a reflection of it adds to a surface what the frame's own sky would.
fn sky_light(direction: vec3<f32>, state: Lighting, cube_0: texture_cube<f32>, cube_1: texture_cube<f32>,
  cube_sampler: sampler, clouds_0: texture_2d<f32>, clouds_1: texture_2d<f32>, repeat: sampler, scale: f32) -> vec3<f32> {
  let below: vec3<f32> = sky_box_shown(direction, state, cube_0, cube_1, cube_sampler, scale);

  return sky_as_light(sky_clouds_over(state, direction, below, clouds_0, clouds_1, repeat, scale), scale,
    state.reflections.w, state.engine.w > 0.5);
}
