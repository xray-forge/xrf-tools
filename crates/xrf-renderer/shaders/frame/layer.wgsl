// A window's layer: a host's own interface (a developer tool's console and stats) drawn over everything else of the
// window, as clipped, textured, coloured triangles in the window's device pixels.

// The window's size in device pixels: a struct, as the HLSL backend takes immediates only as one.
struct LayerWindow {
  size: vec2<f32>,
}

var<immediate> window: LayerWindow;

@group(0) @binding(0) var layer_texture: texture_2d<f32>;
@group(0) @binding(1) var layer_sampler: sampler;

struct LayerVertex {
  @location(0) position: vec2<f32>,
  @location(1) uv: vec2<f32>,
  // Premultiplied, display encoded, eight bits a channel.
  @location(2) color: vec4<f32>,
}

struct LayerVarying {
  @builtin(position) clip: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
}

@vertex
fn vs_layer(vertex: LayerVertex) -> LayerVarying {
  // Device pixels from the top left to clip space, whose `y` grows up.
  let ndc: vec2<f32> = vertex.position / window.size * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0);

  return LayerVarying(vec4<f32>(ndc, 0.0, 1.0), vertex.uv, vertex.color);
}

@fragment
fn fs_layer(in: LayerVarying) -> @location(0) vec4<f32> {
  // The window's swapchain is not sRGB: colours and texels blend as written, display encoded, as the layer's producer
  // shades them.
  return in.color * textureSample(layer_texture, layer_sampler, in.uv);
}
