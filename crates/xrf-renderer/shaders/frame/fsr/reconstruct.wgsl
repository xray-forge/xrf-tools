#import "frame/fsr/nearest"

// `ffx_fsr2_reconstruct_dilated_velocity_and_previous_depth.h`'s scatter, `ReconstructPrevDepth`: each drawn texel's
// nearest depth pushed to where its surface stood the frame before, to every texel its bilinear footprint covers
// there, the nearest kept by an atomic maximum of the depth's bits. The buffer is cleared to the far plane, zero
// reversed, before.

@group(0) @binding(3) var<storage, read_write> reconstructed: array<atomic<u32>>;

@compute @workgroup_size(8, 8)
fn cs_reconstruct(@builtin(global_invocation_id) id: vec3<u32>) {
  let width: u32 = u32(fsr.render_size.x);
  let position: vec2<f32> = vec2<f32>(id.xy);

  if (!is_on_screen(position, fsr.render_size)) {
    return;
  }

  let nearest: Nearest = find_nearest(position);
  let motion: vec2<f32> = load_fsr_motion(nearest.at);
  // Motion under a tenth of a display pixel is taken for none.
  let moved: vec2<f32> = motion * f32(length(motion * fsr.display_size) > 0.1);
  let footprint: BilinearFootprint = bilinear_footprint((position + 0.5) / fsr.render_size + moved, fsr.render_size);
  let bits: u32 = bitcast<u32>(nearest.depth);

  for (var corner: i32 = 0; corner < 4; corner++) {
    let at: vec2<f32> = footprint.base + footprint_corner(corner);

    if (footprint.weights[corner] > RECONSTRUCTED_DEPTH_WEIGHT_THRESHOLD && is_on_screen(at, fsr.render_size)) {
      atomicMax(&reconstructed[u32(at.y) * width + u32(at.x)], bits);
    }
  }
}
