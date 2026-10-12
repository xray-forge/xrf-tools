#import "common/froxel"

// The enhanced volumetric lighting's froxels integrated along the view, front to back: what each froxel scatters, less
// what the air before it takes, added up to its far edge, and how much of what stands behind still shows.

#import "generated/frame/froxel_integrate"

@compute @workgroup_size(8, 8, 1)
fn froxel_integrate(@builtin(global_invocation_id) id: vec3<u32>) {
  let size: vec3<u32> = vec3<u32>(froxels.size.xyz);

  if (any(id.xy >= size.xy)) {
    return;
  }

  var gathered: vec3<f32> = vec3<f32>(0.0);
  var shown: f32 = 1.0;

  for (var slice: u32 = 0u; slice < size.z; slice++) {
    let near: f32 = froxel_depth(froxels, f32(slice) / f32(size.z));
    let far: f32 = froxel_depth(froxels, f32(slice + 1u) / f32(size.z));
    let froxel: vec4<f32> = textureLoad(froxel_scattered, vec3<u32>(id.xy, slice), 0);
    let thickness: f32 = max(froxel.a, 1e-5);
    let through: f32 = exp(-thickness * (far - near));

    // What the froxel scatters over its depth, as much as its own air lets out.
    gathered += shown * froxel.rgb * (1.0 - through) / thickness;
    shown *= through;
    textureStore(froxel_integrated, vec3<u32>(id.xy, slice), vec4<f32>(gathered, shown));
  }
}
