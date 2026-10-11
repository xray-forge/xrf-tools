#import "common/fullscreen"
#import "frame/fsr/common"

// FSR's debug views (after `ffx_fsr3upscaler_debug_view.h`, one target a view rather than its grid), at the display's
// size: each pixel shows what FSR holds at the drawn texel under it, or in its own history.

// The reactive, disocclusion, shading change and accumulation masks.
@group(0) @binding(1) var masks: texture_2d<f32>;
@group(0) @binding(2) var new_locks: texture_2d<f32>;
@group(0) @binding(3) var luma_instability: texture_2d<f32>;
// This frame's history: colour, and each pixel's lock.
@group(0) @binding(4) var history: texture_2d<f32>;

// The drawn texel under a display pixel.
fn drawn_texel(clip: vec2<f32>) -> vec2<i32> {
  return clamped_texel(floor((floor(clip) + 0.5) * fsr.downscale), fsr.render_size);
}

@fragment
fn fs_masks(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let reactive_masks: vec4<f32> = textureLoad(masks, drawn_texel(in.clip.xy), 0);

  return vec4<f32>(reactive_masks[MASK_REACTIVE], reactive_masks[MASK_DISOCCLUSION],
    reactive_masks[MASK_SHADING_CHANGE], 1.0);
}

@fragment
fn fs_history_trust(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let trust: f32 = textureLoad(masks, drawn_texel(in.clip.xy), 0)[MASK_ACCUMULATION];

  return select(vec4<f32>(trust, trust, trust, 1.0), vec4<f32>(1.0, 0.0, 0.0, 1.0), trust == 0.0);
}

@fragment
fn fs_locks(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let lock: f32 = textureLoad(history, vec2<i32>(floor(in.clip.xy)), 0).a;
  let found: f32 = textureLoad(new_locks, drawn_texel(in.clip.xy), 0).x;

  return vec4<f32>(found, lock / LOCK_MAX, 0.0, 1.0);
}

@fragment
fn fs_luma_instability(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let instability: f32 = textureLoad(luma_instability, drawn_texel(in.clip.xy), 0).x;

  return vec4<f32>(instability, instability, instability, 1.0);
}
