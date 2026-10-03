#import "common/fullscreen"
#import "frame/fsr/common"

// `ffx_fsr2_compute_luminance_pyramid.h`, its first step: SPD's box reduction of the frame's log luma, each texel the
// mean of the eight by eight drawn texels under it, read where they stand unjittered; off the screen they count as
// nothing, as SPD counts them. No exposure is taken from it: the frame is in the display's range.

@group(0) @binding(1) var color: texture_2d<f32>;
@group(0) @binding(2) var linear_sampler: sampler;

// Drawn texels a side this step reduces.
const STEP: f32 = 8.0;

@fragment
fn fs_luma_first(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let base: vec2<f32> = floor(in.clip.xy) * STEP;
  var sum: f32 = 0.0;

  for (var y: f32 = 0.0; y < STEP; y += 1.0) {
    for (var x: f32 = 0.0; x < STEP; x += 1.0) {
      let at: vec2<f32> = base + vec2<f32>(x, y);
      let uv: vec2<f32> = clamped_uv((at + 0.5 + fsr.jitter) / fsr.render_size, fsr.render_size);
      let luma: f32 = to_log_luma(textureSampleLevel(color, linear_sampler, uv, 0.0).rgb);

      sum += select(0.0, luma, all(at < fsr.render_size));
    }
  }

  return vec4<f32>(sum / (STEP * STEP), 0.0, 0.0, 1.0);
}
