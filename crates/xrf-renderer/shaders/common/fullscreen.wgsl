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

// A frame target read at a texel of the frame, held to the target's own size: one that stands in at a single texel while
// its feature is off reads that texel, rather than whatever an out-of-bounds load returns.
fn load_frame_target(frame_target: texture_2d<f32>, texel: vec2<i32>) -> vec4<f32> {
  return textureLoad(frame_target, min(texel, vec2<i32>(textureDimensions(frame_target)) - 1), 0);
}
