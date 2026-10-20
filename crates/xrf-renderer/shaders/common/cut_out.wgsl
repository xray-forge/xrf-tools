// Cut-out alpha as every pass drawing a cut-out surface into the G-buffer cuts it.

// The mip a texture of `size` texels is read at where its coordinate's derivatives are `dx` and `dy`: its footprint's
// longer side.
fn footprint_mip(size: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> f32 {
  return 0.5 * log2(max(max(dot(dx * size, dx * size), dot(dy * size, dy * size)), 1e-8));
}

// Whether a cut-out texel read at `mip` is cut: its alpha raised where minification thins it out.
fn is_alpha_cut_at(alpha: f32, mip: f32, reference: f32) -> bool {
  return alpha * (1.0 + 0.25 * max(0.0, mip)) <= reference;
}

// Whether a cut-out texel is cut, its mip taken from the texture's `size` in texels and its coordinate's derivatives.
fn is_alpha_cut(alpha: f32, size: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>, reference: f32) -> bool {
  return is_alpha_cut_at(alpha, footprint_mip(size, dx, dy), reference);
}
