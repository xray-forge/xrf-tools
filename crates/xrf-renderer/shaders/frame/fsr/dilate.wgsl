#import "common/fullscreen"
#import "frame/fsr/nearest"

// `ReconstructAndDilate` less the scatter: the nearest depth of the nine, the motion found there, and the luma the locks
// read.

@group(0) @binding(3) var color: texture_2d<f32>;

struct DilateOutput {
  @location(0) depth: vec4<f32>,
  @location(1) motion: vec4<f32>,
  @location(2) lock_luma: vec4<f32>,
};

@fragment
fn fs_dilate(in: FullscreenVarying) -> DilateOutput {
  let position: vec2<f32> = floor(in.clip.xy);
  let nearest: Nearest = find_nearest(position);
  // `ComputeLockInputLuma`: an exposure of one, the colour already in the display's range.
  let rgb: vec3<f32> = max(textureLoad(color, clamped_texel(position, fsr.render_size), 0).rgb, vec3<f32>(0.0));
  var out: DilateOutput;

  out.depth = vec4<f32>(nearest.depth, 0.0, 0.0, 1.0);
  out.motion = vec4<f32>(load_fsr_motion(nearest.at), 0.0, 1.0);
  out.lock_luma = vec4<f32>(pow(max(to_perceived_luma(rgb), 0.0), 1.0 / 6.0), 0.0, 0.0, 1.0);

  return out;
}
