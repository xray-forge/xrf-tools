#import "common/camera"

// The bolt striking, as `dxThunderboltRender` draws it over the finished scene and tested against its depth: its model
// placed by the strike, its coordinates shifted down by it, and its two glows facing the view, each composited as its
// shader says.

// The strike, the draw's texture, and the bolt's model in renderer space (its position, then its coordinate, a vertex).
#import "generated/frame/thunder"

// A glow's quad: its corners as two triangles, each from minus one to one across and up.
const GLOW_CORNERS: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
  vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(-1.0, 1.0),
  vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
);

struct ThunderVarying {
  @builtin(position) clip: vec4<f32>,
  @location(0) uv: vec2<f32>,
  // What the texel's colour and alpha are both scaled by.
  @location(1) opacity: f32,
};

@vertex
fn vs_model(@builtin(vertex_index) vertex_index: u32) -> ThunderVarying {
  let index: u32 = model_indices[vertex_index];
  let local: vec3<f32> = model_vertices[index * 2u].xyz;
  let world: vec3<f32> = thunder.position.xyz + thunder.axes[0].xyz * local.x + thunder.axes[1].xyz * local.y +
    thunder.axes[2].xyz * local.z;
  var out: ThunderVarying;

  out.clip = camera.view_projection * vec4<f32>(world, 1.0);
  out.uv = model_vertices[index * 2u + 1u].xy + vec2<f32>(0.0, thunder.shift.x);
  out.opacity = 1.0;

  return out;
}

// A glow (`SFlare`): a quad about its point, facing the view along its right and its top; the corner to its right and
// top has the first texel, as the engine lays the quad out.
fn glow(vertex_index: u32, position: vec3<f32>, extent: vec4<f32>) -> ThunderVarying {
  var corners: array<vec2<f32>, 6> = GLOW_CORNERS;
  let corner: vec2<f32> = corners[vertex_index % 6u];
  let right: vec3<f32> = vec3<f32>(camera.view[0].x, camera.view[1].x, camera.view[2].x);
  let up: vec3<f32> = vec3<f32>(camera.view[0].y, camera.view[1].y, camera.view[2].y);
  let world: vec3<f32> = position + right * corner.x * extent.x + up * corner.y * extent.y;
  var out: ThunderVarying;

  out.clip = camera.view_projection * vec4<f32>(world, 1.0);
  out.uv = (1.0 - corner) * 0.5;
  out.opacity = extent.z;

  return out;
}

@vertex
fn vs_glow_top(@builtin(vertex_index) vertex_index: u32) -> ThunderVarying {
  return glow(vertex_index, thunder.top.xyz, thunder.top_extent);
}

@vertex
fn vs_glow_center(@builtin(vertex_index) vertex_index: u32) -> ThunderVarying {
  return glow(vertex_index, thunder.center.xyz, thunder.center_extent);
}

@fragment
fn fs_thunder(in: ThunderVarying) -> @location(0) vec4<f32> {
  return textureSample(thunder_texture, thunder_sampler, in.uv) * in.opacity;
}
