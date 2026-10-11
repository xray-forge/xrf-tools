#import "frame/fsr/common"

// What the reconstruction and the input preparation read of the frame as drawn: its depth and its motion.

@group(0) @binding(1) var depth_target: texture_depth_2d;
@group(0) @binding(2) var motion_target: texture_2d<f32>;

// `LoadInputMotionVector`: FSR's motion is the renderer's turned, from now to the frame before.
fn load_fsr_motion(position: vec2<f32>) -> vec2<f32> {
  return -textureLoad(motion_target, clamped_texel(position, fsr.render_size), 0).xy;
}

fn load_depth(position: vec2<f32>) -> f32 {
  return textureLoad(depth_target, clamped_texel(position, fsr.render_size), 0);
}

// The nearest depth about a texel, the texel it stands at, and the farthest of those nearer than the centre.
struct DepthExtents {
  nearest: f32,
  at: vec2<f32>,
  farthest: f32,
};

// `FindDepthExtents`, reversed: the nearest of the nine on the screen is the greatest. The farthest only moves while a
// nearer one is found, as AMD's does.
fn find_depth_extents(position: vec2<f32>) -> DepthExtents {
  let centre: f32 = load_depth(position);
  var extents: DepthExtents = DepthExtents(centre, position, centre);

  for (var index: i32 = 1; index < 9; index++) {
    let sample: vec2<f32> = position + NEAREST_ORDER[index];
    let sampled: f32 = load_depth(sample);

    if (is_on_screen(sample, fsr.render_size) && sampled > extents.nearest) {
      extents = DepthExtents(sampled, sample, min(extents.farthest, sampled));
    }
  }

  return extents;
}
