#import "common/fullscreen"
#import "common/reactive"

// The mask DLSS super resolution upscales from this frame rather than the history by, at the drawn size: how far what
// drew over the surfaces (the rain, the particles, the water, the composited surfaces) changed each texel against the
// frame before them, relative to its brightness, as the temporal resolve's reactive weight reads it, at most
// `REACTIVE_BIAS`. What drew over a surface moves with the surface's motion behind it, and the history smeared a rain
// streak along a swaying blade.

// The frame as DLSS takes it, and as it stood before the water, the composited surfaces and the particles drew.
#import "generated/frame/dlss_reactive"

const REACTIVE_BIAS: f32 = 0.8;

@fragment
fn fs_dlss_reactive(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let over: vec3<f32> = max(textureLoad(frame, texel, 0).rgb, vec3<f32>(0.0));
  let under: vec3<f32> = max(textureLoad(opaque, texel, 0).rgb, vec3<f32>(0.0));
  let reactive: f32 = reactive_change(over, under);

  return vec4<f32>(reactive * REACTIVE_BIAS, 0.0, 0.0, 0.0);
}
