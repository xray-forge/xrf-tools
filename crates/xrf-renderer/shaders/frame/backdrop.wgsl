#import "common/fullscreen"

// The page's backdrop where it is transparent around the viewports: each box painting there in its colour, outermost
// first, and over the washed ones the wash, as CSS `linear-gradient(angle, from, to)` paints it over a background colour.

// The wash's two stops premultiplied by their alpha, its box and its gradient line, then the planes
// (`BackdropUniform`).
#import "generated/structs"

// Planes `BackdropUniform::PLANES` holds room for.
const BACKDROP_PLANES: u32 = 4u;

@group(0) @binding(0) var<uniform> backdrop: Backdrop;

// How much of the pixel at `at` a box covers whose corners are rounded by `radii` (top left, top right, bottom right,
// bottom left), eased over one pixel at their curve.
fn get_coverage(at: vec2<f32>, area: vec4<f32>, radii: vec4<f32>) -> f32 {
  if (any(area.zw <= vec2<f32>(0.0))) {
    return 0.0;
  }

  let half: vec2<f32> = area.zw * 0.5;
  let offset: vec2<f32> = at - area.xy - half;
  // The window's `y` grows down, so a positive offset is the bottom half.
  let side: vec2<f32> = select(radii.xw, radii.yz, offset.x > 0.0);
  let corner: f32 = min(select(side.x, side.y, offset.y > 0.0), min(half.x, half.y));
  let outside: vec2<f32> = abs(offset) - half + corner;
  let distance: f32 = length(max(outside, vec2<f32>(0.0))) + min(max(outside.x, outside.y), 0.0) - corner;

  return clamp(0.5 - distance, 0.0, 1.0);
}

fn get_painted(index: u32, wash: vec4<f32>) -> vec3<f32> {
  let color: vec4<f32> = backdrop.colors[index];

  return select(color.rgb, color.rgb * (1.0 - wash.a) + wash.rgb, color.w > 0.0);
}

@fragment
fn fs_backdrop(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let at: vec2<f32> = in.clip.xy;
  let rect: vec4<f32> = backdrop.rect;
  let along: f32 = dot(at - (rect.xy + rect.zw * 0.5), backdrop.line.xy) / max(backdrop.line.z, 1.0);
  let wash: vec4<f32> = mix(backdrop.first, backdrop.last, clamp(along + 0.5, 0.0, 1.0));
  var color: vec3<f32> = get_painted(0u, wash);

  for (var index: u32 = 1u; index < BACKDROP_PLANES; index++) {
    color = mix(color, get_painted(index, wash), get_coverage(at, backdrop.boxes[index], backdrop.radii[index]));
  }

  // The window stores sRGB as given, as the page's colours are.
  return vec4<f32>(color, 1.0);
}
