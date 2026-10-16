#import "common/fullscreen"
#import "frame/fsr/common"

// `ffx_fsr3upscaler_shading_change.h`: at half the drawn size, how much the shading changed since the frame before, the
// largest across the shading change pyramid's three levels of the mean change times the mean of its sign, so that a
// change only counts where it agrees in direction.

@group(0) @binding(1) var level0: texture_2d<f32>;
@group(0) @binding(2) var level1: texture_2d<f32>;
@group(0) @binding(3) var level2: texture_2d<f32>;
@group(0) @binding(4) var linear_sampler: sampler;

// `ComputeShadingChangeLuma`'s sample of a level.
fn level_change(level: texture_2d<f32>, uv: vec2<f32>) -> f32 {
  let sample: vec2<f32> = textureSampleLevel(level, linear_sampler, uv, 0.0).xy;

  return abs(sample.x * sample.y);
}

@fragment
fn fs_shading_change(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let uv: vec2<f32> = (floor(in.clip.xy) + 0.5) / fsr.shading_size + fsr.jitter / fsr.render_size;
  let level_uv: vec2<f32> = clamped_uv(uv, fsr.shading_size);
  let changes: vec3<f32> = vec3<f32>(level_change(level0, level_uv), level_change(level1, level_uv),
    level_change(level2, level_uv));

  return vec4<f32>(saturate(max(changes.x, max(changes.y, changes.z))), 0.0, 0.0, 1.0);
}
