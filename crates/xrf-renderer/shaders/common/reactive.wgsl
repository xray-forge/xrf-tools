#import "common/luminance"

// How strongly a change over the opaque frame, relative to its brightness, marks a texel reactive.
const REACTIVE_GAIN: f32 = 2.0;

// How far what drew over the opaque frame (the rain, the particles, the water, the composited surfaces) changed a
// texel, `over` against `under`, relative to their brightness, from none to one: what the temporal resolve weighs its
// history by, and the mask DLSS upscales by.
fn reactive_change(over: vec3<f32>, under: vec3<f32>) -> f32 {
  return saturate(length(over - under) / (luminance(over) + luminance(under) + 0.05) * REACTIVE_GAIN);
}
