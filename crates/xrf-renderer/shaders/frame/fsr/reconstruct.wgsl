#import "frame/fsr/nearest"

// `ffx_fsr3upscaler_prepare_inputs.h`'s scatter, `ReconstructPrevDepth`: each drawn texel's nearest depth pushed to
// where its surface stood the frame before, to every texel its bilinear footprint covers there, the nearest kept by an
// atomic maximum of the depth's bits. The buffer is cleared to the far plane, zero reversed, before.

@group(0) @binding(3) var<storage, read_write> reconstructed: array<atomic<u32>>;

@compute @workgroup_size(8, 8)
fn cs_reconstruct(@builtin(global_invocation_id) id: vec3<u32>) {
  let width: u32 = u32(fsr.render_size.x);
  let position: vec2<f32> = vec2<f32>(id.xy);

  if (!is_on_screen(position, fsr.render_size)) {
    return;
  }

  let extents: DepthExtents = find_depth_extents(position);
  let motion: vec2<f32> = load_fsr_motion(extents.at);
  let depth_in_meters: f32 = min(to_view_depth(extents.nearest), FSR_FP16_MAX);
  // Motion too small to tell from still is taken for none.
  let moved: vec2<f32> = motion * f32(to_4k_velocity(motion) > still_threshold(depth_in_meters));
  let footprint: BilinearFootprint = bilinear_footprint((position + 0.5) / fsr.render_size + moved, fsr.render_size);
  let bits: u32 = bitcast<u32>(extents.nearest);

  for (var corner: i32 = 0; corner < 4; corner++) {
    let at: vec2<f32> = footprint.base + footprint_corner(corner);

    if (footprint.weights[corner] > RECONSTRUCTED_DEPTH_WEIGHT_THRESHOLD && is_on_screen(at, fsr.render_size)) {
      atomicMax(&reconstructed[u32(at.y) * width + u32(at.x)], bits);
    }
  }
}
