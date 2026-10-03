#import "common/fullscreen"

// The page's backdrop where it is transparent around the viewports: its colour, and over its box the wash, as CSS
// `linear-gradient(angle, from, to)` paints it over a background colour.

struct Backdrop {
  color: vec4<f32>,
  // Premultiplied by their alpha.
  first: vec4<f32>,
  last: vec4<f32>,
  // Left, top, width and height in window pixels.
  rect: vec4<f32>,
  // The gradient line's direction, `y` down, and its length.
  line: vec4<f32>,
};

@group(0) @binding(0) var<uniform> backdrop: Backdrop;

@fragment
fn fs_backdrop(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let at: vec2<f32> = in.clip.xy;
  let rect: vec4<f32> = backdrop.rect;
  let is_inside: bool = all(at >= rect.xy) && all(at < rect.xy + rect.zw);
  let along: f32 = dot(at - (rect.xy + rect.zw * 0.5), backdrop.line.xy) / max(backdrop.line.z, 1.0);
  let wash: vec4<f32> = mix(backdrop.first, backdrop.last, clamp(along + 0.5, 0.0, 1.0));
  let washed: vec3<f32> = backdrop.color.rgb * (1.0 - wash.a) + wash.rgb;

  // The window stores sRGB as given, as the page's colours are.
  return vec4<f32>(select(backdrop.color.rgb, washed, is_inside), 1.0);
}
