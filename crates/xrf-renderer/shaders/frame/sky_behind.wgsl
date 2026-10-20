#import "common/camera"
#import "common/lighting"
#import "common/scene_light"
#import "common/sky"
#import "common/fullscreen"

// The sky the distance fog fades into, along each pixel's view at half the frame's size, which combine mixes in by the
// fog's square: the sky as drawn, and the haze above the fold where the haze is drawn. Only where one of the frame's
// pixels a texel covers shows it; the rest are left clear, and combine weighs the texels it reads by their alpha, so
// none bleeds into a fogged pixel beside it.

// The lighting and exposure, the haze map (the sky as drawn, clouds and all, blurred by bearing and height) and the
// frame's depth.
#import "generated/frame/sky_behind"

// The fog's square below which the sky behind changes no pixel by half an eight-bit step, as combine leaves it out.
const UNSEEN_SKY: f32 = 0.001953125;

// Whether the sky behind shows at a pixel of the frame: something is drawn there, short of where the fog is total, and
// the fog is thick enough for its square to show the sky.
fn is_shown_behind(texel: vec2<i32>) -> bool {
  let size: vec2<i32> = vec2<i32>(textureDimensions(depth_target));
  let depth: f32 = textureLoad(depth_target, min(texel, size - 1), 0);
  let fog: f32 = fog_amount(lighting, camera_view_position(vec2<f32>(texel) + 0.5, max(depth, 1e-7)));

  return depth > 0.0 && fog < 1.0 && fog * fog >= UNSEEN_SKY;
}

@fragment
fn fs_sky_behind(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let first: vec2<i32> = vec2<i32>(in.clip.xy) * 2;

  if (!(is_shown_behind(first) || is_shown_behind(first + vec2<i32>(1, 0)) || is_shown_behind(first + vec2<i32>(0, 1)) ||
    is_shown_behind(first + vec2<i32>(1, 1)))) {
    return vec4<f32>(0.0);
  }

  // Along the view through the frame's pixel this one's centre stands over.
  let toward: vec3<f32> = normalize(camera_view_position(in.clip.xy * 2.0, 0.5));
  let direction: vec3<f32> = normalize((transpose(camera.view) * vec4<f32>(toward, 0.0)).xyz);
  let scale: f32 = picture_exposure(lighting, exposure);
  let is_hazed: bool = lighting.fog.z > 0.5;

  return vec4<f32>(sky_behind(lighting, haze_map, direction, toward, scale, is_hazed), 1.0);
}
