#import "common/fullscreen"
#import "frame/fsr/nearest"

// `ffx_fsr3upscaler_prepare_inputs.h` less the scatter: the nearest depth of the nine and the motion found there, the
// farthest depth in metres, and the frame's luma.

@group(0) @binding(3) var color: texture_2d<f32>;

struct PrepareInputsOutput {
  @location(0) dilated_depth: vec4<f32>,
  @location(1) dilated_motion: vec4<f32>,
  @location(2) farthest_depth: vec4<f32>,
  @location(3) luma: vec4<f32>,
};

@fragment
fn fs_prepare_inputs(in: FullscreenVarying) -> PrepareInputsOutput {
  let position: vec2<f32> = floor(in.clip.xy);
  let extents: DepthExtents = find_depth_extents(position);
  // `GetCurrentFrameLuma`: linear colour, kept off negative.
  let rgb: vec3<f32> = max(textureLoad(color, clamped_texel(position, fsr.render_size), 0).rgb, vec3<f32>(0.0));
  var out: PrepareInputsOutput;

  out.dilated_depth = vec4<f32>(extents.nearest, 0.0, 0.0, 1.0);
  out.dilated_motion = vec4<f32>(load_fsr_motion(extents.at), 0.0, 1.0);
  out.farthest_depth = vec4<f32>(min(to_view_depth(extents.farthest), FSR_FP16_MAX), 0.0, 0.0, 1.0);
  out.luma = vec4<f32>(to_luma(rgb), 0.0, 0.0, 1.0);

  return out;
}
