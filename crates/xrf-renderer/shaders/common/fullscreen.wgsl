// One triangle covering the target, which a full-screen pass draws its fragments over.

struct FullscreenVarying {
  @builtin(position) clip: vec4<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) index: u32) -> FullscreenVarying {
  let uv: vec2<f32> = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
  var out: FullscreenVarying;

  out.clip = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);

  return out;
}
