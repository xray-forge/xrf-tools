#import "frame/fsr/common"

// What the reconstruction and the dilation read of the frame as drawn: its depth and its motion.

@group(0) @binding(1) var depth_target: texture_depth_2d;
@group(0) @binding(2) var motion_target: texture_2d<f32>;

// `LoadInputMotionVector`: FSR's motion is the renderer's turned, from now to the frame before.
fn load_fsr_motion(position: vec2<f32>) -> vec2<f32> {
  return -textureLoad(motion_target, clamped_texel(position, fsr.render_size), 0).xy;
}

fn load_depth(position: vec2<f32>) -> f32 {
  return textureLoad(depth_target, clamped_texel(position, fsr.render_size), 0);
}

// The nearest depth about a texel, and the texel it stands at.
struct Nearest {
  depth: f32,
  at: vec2<f32>,
};

// `FindNearestDepth`, reversed: the nearest of the nine on the screen is the greatest.
fn find_nearest(position: vec2<f32>) -> Nearest {
  var nearest: Nearest = Nearest(load_depth(position), position);

  for (var index: i32 = 1; index < 9; index++) {
    let sample: vec2<f32> = position + NEAREST_ORDER[index];
    let sampled: f32 = load_depth(sample);

    if (is_on_screen(sample, fsr.render_size) && sampled > nearest.depth) {
      nearest = Nearest(sampled, sample);
    }
  }

  return nearest;
}
