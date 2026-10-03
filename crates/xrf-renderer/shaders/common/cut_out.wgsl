// Cut-out alpha as every pass drawing a cut-out surface into the G-buffer cuts it.

// Whether a cut-out texel is cut: its alpha raised where minification thins it out, cut along a ramp one texel wide so
// the edge does not shimmer. `size` is the texture's in texels, `dx` and `dy` its coordinate's derivatives.
fn is_alpha_cut(alpha: f32, size: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>, reference: f32) -> bool {
  let extent: f32 = max(dot(dx * size, dx * size), dot(dy * size, dy * size));
  let raised: f32 = alpha * (1.0 + 0.25 * max(0.0, 0.5 * log2(max(extent, 1e-8))));
  let width: f32 = max(abs(dpdx(raised)) + abs(dpdy(raised)), 1.0 / 255.0);

  return saturate((raised - reference) / width + 0.5) <= 0.5;
}
