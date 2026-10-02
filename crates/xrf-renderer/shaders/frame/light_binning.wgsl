#import "common/light_clusters"

// Bins the lights standing in view into the clusters of the view, an invocation a cluster: each keeps the lights whose
// sphere reaches its box, up to its capacity. The two words after the clusters' counts count the clusters that ran out of
// room and the lights they left out, for the readouts.

@group(0) @binding(0) var<storage, read> records: array<LightRecord>;
@group(0) @binding(1) var<storage, read_write> counts: array<atomic<u32>>;
@group(0) @binding(2) var<storage, read_write> items: array<u32>;
@group(0) @binding(3) var<uniform> lights: Lights;

// A device coordinate across or up at a depth, in view space: `(ndc + offset) * depth / scale`.
fn to_view(ndc: f32, offset: f32, scale: f32, depth: f32) -> f32 {
  return (ndc + offset) * depth / scale;
}

@compute @workgroup_size(64)
fn bin(@builtin(global_invocation_id) id: vec3<u32>) {
  let cluster: u32 = id.x;

  if (cluster >= LIGHT_CLUSTERS) {
    return;
  }

  let x: f32 = f32(cluster % LIGHT_CLUSTERS_X);
  let y: f32 = f32((cluster / LIGHT_CLUSTERS_X) % LIGHT_CLUSTERS_Y);
  let z: f32 = f32(cluster / (LIGHT_CLUSTERS_X * LIGHT_CLUSTERS_Y));
  let near: f32 = light_slice_depth(lights, z);
  let far: f32 = light_slice_depth(lights, z + 1.0);
  // The tile's edges in device coordinates, `y` up from the bottom as clip space runs, the first row the top one.
  let left: f32 = x * (2.0 / f32(LIGHT_CLUSTERS_X)) - 1.0;
  let right: f32 = left + 2.0 / f32(LIGHT_CLUSTERS_X);
  let top: f32 = 1.0 - y * (2.0 / f32(LIGHT_CLUSTERS_Y));
  let bottom: f32 = top - 2.0 / f32(LIGHT_CLUSTERS_Y);
  let p: vec4<f32> = lights.projection;
  let xs: vec4<f32> = vec4<f32>(
    to_view(left, p.z, p.x, near),
    to_view(left, p.z, p.x, far),
    to_view(right, p.z, p.x, near),
    to_view(right, p.z, p.x, far)
  );
  let ys: vec4<f32> = vec4<f32>(
    to_view(bottom, p.w, p.y, near),
    to_view(bottom, p.w, p.y, far),
    to_view(top, p.w, p.y, near),
    to_view(top, p.w, p.y, far)
  );
  let least: vec3<f32> = vec3<f32>(min(min(xs.x, xs.y), min(xs.z, xs.w)), min(min(ys.x, ys.y), min(ys.z, ys.w)), -far);
  let most: vec3<f32> = vec3<f32>(max(max(xs.x, xs.y), max(xs.z, xs.w)), max(max(ys.x, ys.y), max(ys.z, ys.w)), -near);
  var kept: u32 = 0u;
  var reached: u32 = 0u;

  for (var light: u32 = 0u; light < lights.count; light++) {
    let sphere: vec4<f32> = records[light].sphere;
    let offset: vec3<f32> = sphere.xyz - clamp(sphere.xyz, least, most);

    if (dot(offset, offset) <= sphere.w * sphere.w) {
      reached += 1u;

      if (kept < LIGHT_CLUSTER_CAPACITY) {
        items[cluster * LIGHT_CLUSTER_CAPACITY + kept] = light;
        kept += 1u;
      }
    }
  }

  atomicStore(&counts[cluster], kept);

  if (reached > kept) {
    atomicAdd(&counts[LIGHT_CLUSTERS], 1u);
    atomicAdd(&counts[LIGHT_CLUSTERS + 1u], reached - kept);
  }
}
