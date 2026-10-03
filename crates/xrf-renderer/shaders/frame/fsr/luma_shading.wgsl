#import "common/fullscreen"
#import "frame/fsr/common"

// The luminance pyramid's second step, on to the mip the locks watch for a change of shading
// (`FFX_FSR2_SHADING_CHANGE_MIP_LEVEL`): each texel the mean of the four by four first-step texels under it.

@group(0) @binding(1) var first_step: texture_2d<f32>;

// First-step texels a side this step reduces.
const STEP: f32 = 4.0;

@fragment
fn fs_luma_shading(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let size: vec2<f32> = vec2<f32>(textureDimensions(first_step));
  let base: vec2<f32> = floor(in.clip.xy) * STEP;
  var sum: f32 = 0.0;

  for (var y: f32 = 0.0; y < STEP; y += 1.0) {
    for (var x: f32 = 0.0; x < STEP; x += 1.0) {
      let at: vec2<f32> = base + vec2<f32>(x, y);

      sum += select(0.0, textureLoad(first_step, clamped_texel(at, size), 0).x, all(at < size));
    }
  }

  return vec4<f32>(sum / (STEP * STEP), 0.0, 0.0, 1.0);
}
