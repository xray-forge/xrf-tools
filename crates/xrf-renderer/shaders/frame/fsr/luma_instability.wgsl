#import "common/fullscreen"
#import "frame/fsr/common"

// `ffx_fsr3upscaler_luma_instability.h`, at the drawn size: each texel's four last lumas, and whether this frame's
// returns to an earlier one rather than the last, a flicker the accumulation then lets the history hold through. Only
// where the history is trusted; elsewhere the lumas start again.

@group(0) @binding(1) var dilated_motion: texture_2d<f32>;
// The reactive, disocclusion, shading change and accumulation masks.
@group(0) @binding(2) var masks: texture_2d<f32>;
@group(0) @binding(3) var luma: texture_2d<f32>;
@group(0) @binding(4) var previous_luma_history: texture_2d<f32>;
@group(0) @binding(5) var linear_sampler: sampler;

// The trust in the history past which the lumas are watched.
const WATCHED_ACCUMULATION: f32 = 0.9;

struct LumaInstabilityOutput {
  @location(0) luma_history: vec4<f32>,
  @location(1) instability: vec4<f32>,
};

// `ComputeLumaInstabilityFactor`: whether this frame's luma is more like an earlier one, changed the same way, than the
// last; once four lumas stand, the oldest kept being the history's third.
fn instability_factor(current: f32, history: vec4<f32>) -> f32 {
  let first_diff: f32 = current - history.x;
  let first_similarity: f32 = min_over_max_or(current, history.x, 1.0);
  var most_similar: f32 = first_similarity;

  if (first_similarity >= 1.0) {
    return 0.0;
  }

  for (var index: i32 = 1; index < 4; index++) {
    if (sign(first_diff) == sign(current - history[index])) {
      most_similar = max(most_similar, min_over_max(current, history[index]));
    }
  }

  return f32(most_similar > first_similarity) * f32(history.z != 0.0);
}

@fragment
fn fs_luma_instability(in: FullscreenVarying) -> LumaInstabilityOutput {
  let position: vec2<f32> = floor(in.clip.xy);
  let motion: vec2<f32> = textureLoad(dilated_motion, clamped_texel(position, fsr.render_size), 0).xy;
  let uv: vec2<f32> = (position + 0.5) / fsr.render_size;
  let drawn_uv: vec2<f32> = clamped_uv(uv + fsr.jitter / fsr.render_size, fsr.render_size);
  let reprojected: vec2<f32> = uv + fsr.previous_jitter / fsr.render_size + motion;
  var history: vec4<f32> = vec4<f32>(0.0);
  var factor: f32 = 0.0;

  if (is_uv_inside(reprojected)) {
    let reactive_masks: vec4<f32> = saturate(textureSampleLevel(masks, linear_sampler, drawn_uv, 0.0));

    if (reactive_masks[MASK_ACCUMULATION] > WATCHED_ACCUMULATION) {
      let current: f32 = textureSampleLevel(luma, linear_sampler, drawn_uv, 0.0).x;
      let previous: vec4<f32> = textureSampleLevel(previous_luma_history, linear_sampler,
        clamped_uv(reprojected, fsr.render_size), 0.0);
      // A flicker is held less the faster the surface moves.
      let velocity_weight: f32 = 1.0 - saturate(to_4k_velocity(motion) / 20.0);

      factor =instability_factor(current, previous) * velocity_weight
        * (1.0 - reactive_masks[MASK_DISOCCLUSION]) * (1.0 - reactive_masks[MASK_REACTIVE])
        * (1.0 - reactive_masks[MASK_SHADING_CHANGE]);
      history = vec4<f32>(current, previous.x, previous.y, previous.z);
    }
  }

  var out: LumaInstabilityOutput;

  out.luma_history = history;
  out.instability = vec4<f32>(factor, 0.0, 0.0, 1.0);

  return out;
}
