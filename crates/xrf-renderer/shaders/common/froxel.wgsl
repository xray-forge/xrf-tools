#import "generated/structs"

// The enhanced volumetric lighting's froxels, as `pass/froxel_uniform.rs` describes their grid: froxels across and down
// the screen and along the view, their slices thin near the eye and thickening away from it, a share `t` of the grid
// standing at `distance × t²` along the view.

// The view depth a share of the grid along the view stands at.
fn froxel_depth(froxels: Froxels, t: f32) -> f32 {
  return froxels.size.w * t * t;
}

// The share of the grid along the view a view depth stands at, its far end past the volume's reach.
fn froxel_share(froxels: Froxels, depth: f32) -> f32 {
  return sqrt(saturate(depth / froxels.size.w));
}

// Henyey-Greenstein's phase for a lean `g` and the cosine between where light went and where it goes, scaled so light
// scattered every way alike weighs one.
fn froxel_phase(g: f32, cosine: f32) -> f32 {
  let g2: f32 = g * g;

  return (1.0 - g2) / pow(max(1.0 + g2 - 2.0 * g * cosine, 1e-4), 1.5);
}
