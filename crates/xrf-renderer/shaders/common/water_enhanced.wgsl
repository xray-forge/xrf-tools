#import "common/camera"
#import "common/sky_box"
#import "generated/structs"

// The sky turned as its box is, raised to fill a reflection's perspective, both keyframes' cubes tinted by the sky's
// colour.
fn enhanced_sky(
  direction: vec3<f32>,
  lighting: Lighting,
  sky_cube_0: texture_cube<f32>,
  sky_cube_1: texture_cube<f32>,
  sky_clamp: sampler,
) -> vec3<f32> {
  var turned: vec3<f32> = sky_box_direction(direction, lighting.sky_params.x);

  turned.y = (turned.y - max(cos(turned.x) * 0.65, cos(turned.z) * 0.65)) * 2.1 + 0.35;

  let cubes: vec3<f32> = mix(
    textureSampleLevel(sky_cube_0, sky_clamp, turned, 0.0).rgb,
    textureSampleLevel(sky_cube_1, sky_clamp, turned, 0.0).rgb,
    lighting.sky.w,
  );

  return saturate(lighting.sky.rgb) * cubes;
}

// How far along the view the G-buffer's surface lies at a place on the screen; nothing where only the sky is. The
// perspective's depth terms alone give it, as its `w` is the distance: `depth * d = P[3][2] - P[2][2] * d`.
fn scene_distance(uv: vec2<f32>, depth_target: texture_depth_2d) -> f32 {
  let size: vec2<f32> = camera.viewport.xy;
  let pixel: vec2<f32> = clamp(floor(uv * size), vec2<f32>(0.0), size - 1.0);
  let stored: f32 = textureLoad(depth_target, vec2<i32>(pixel), 0);

  return select(camera.projection[3][2] / (stored + camera.projection[2][2]), 0.0, stored <= 0.0);
}
