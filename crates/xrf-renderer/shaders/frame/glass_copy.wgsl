// The frame copied for the enhanced glass to refract, where the glass draws at all (`frame/glass_copy_dispatch`).

// The frame before the glass, and its copy.
#import "generated/frame/glass_copy"

@compute @workgroup_size(8, 8)
fn cs_glass_copy(@builtin(global_invocation_id) id: vec3<u32>) {
  if (any(id.xy >= textureDimensions(scene))) {
    return;
  }

  textureStore(copied, id.xy, textureLoad(scene, id.xy, 0));
}
