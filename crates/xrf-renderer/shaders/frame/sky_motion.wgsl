#import "common/camera"
#import "common/fullscreen"

// Where nothing was drawn, the motion a point of the sky makes as the camera turns, into the motion target the G-buffer
// cleared there: the resolves reading the target whole, DLSS's above all, would otherwise hold the sky still on the
// screen and drag the last frames' sky along as the view turns.

// The frame's depth.
#import "generated/frame/sky_motion"

// Metres along its ray a pixel's sky point stands: far enough that the camera's own travel moves it by no visible share.
const SKY_DISTANCE: f32 = 1e5;

@fragment
fn fs_sky_motion(in: FullscreenVarying) -> @location(0) vec2<f32> {
  if (textureLoad(depth_target, vec2<i32>(in.clip.xy), 0) > 0.0) {
    discard;
  }

  let toward: vec3<f32> = normalize(camera_view_position(in.clip.xy, 1.0));
  let sky: vec3<f32> = camera.position.xyz + (transpose(camera.view) * vec4<f32>(toward, 0.0)).xyz * SKY_DISTANCE;

  return camera_motion(sky, sky);
}
