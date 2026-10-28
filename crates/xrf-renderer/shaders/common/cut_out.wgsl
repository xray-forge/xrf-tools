// Cut-out alpha as every pass drawing a cut-out surface into the G-buffer cuts it.

// The mip a texture of `size` texels is read at where its coordinate's derivatives are `dx` and `dy`: its footprint's
// longer side.
fn footprint_mip(size: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> f32 {
  return 0.5 * log2(max(max(dot(dx * size, dx * size), dot(dy * size, dy * size)), 1e-8));
}

// How much a cut-out's alpha is raised a mip it is read below its top, so a blade minification averages with the gaps
// beside it still covers about what it covers up close: measured against a converged reference under DLAA with wind,
// 1 steadies the plants (error 5–8% lower, flicker 4–12%, still air too) where 0.25 left them thinning and 2 adds
// little (`plans/20261028043855-foliage-temporal-stability.md`).
const CUT_OUT_MIP_LIFT: f32 = 1.0;

// Whether a cut-out texel read at `mip` is cut: its alpha raised where minification thins it out.
fn is_alpha_cut_at(alpha: f32, mip: f32, reference: f32) -> bool {
  return alpha * (1.0 + CUT_OUT_MIP_LIFT * max(0.0, mip)) <= reference;
}

// Whether a cut-out texel is cut, its mip taken from the texture's `size` in texels and its coordinate's derivatives.
fn is_alpha_cut(alpha: f32, size: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>, reference: f32) -> bool {
  return is_alpha_cut_at(alpha, footprint_mip(size, dx, dy), reference);
}
