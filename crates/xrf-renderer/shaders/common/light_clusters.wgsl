// The local lights' records and the clusters of the view they are binned into, as `pass/light_record.rs` and
// `pass/lights_uniform.rs` write them.

struct LightRecord {
  // Its position, then `1 / L_R²`.
  position: vec4<f32>,
  // Its colour, then its specular weight.
  color: vec4<f32>,
  // Where it points, then `cos` of half its cone, or less than minus one for none.
  axis: vec4<f32>,
  // Its right, then its projection's scale.
  right: vec4<f32>,
  // Its up, then its projector's texture slot, or less than zero for none.
  up: vec4<f32>,
  // The sphere it is binned by.
  sphere: vec4<f32>,
  // A shadowed light's near and far planes and face count.
  shadow: vec4<f32>,
  faces: array<vec4<f32>, 6>,
};

struct Lights {
  count: u32,
  near: f32,
  far: f32,
  // How the shadowed lights' maps are compared: zero `shadow_hw`, one `shadow_pcss`.
  shadow_filter: u32,
  // The projection's `x` and `y` scales and its offsets.
  projection: vec4<f32>,
};

// Tiles the view is cut into across and down, and slices into along it.
const LIGHT_CLUSTERS_X: u32 = 16u;
const LIGHT_CLUSTERS_Y: u32 = 9u;
const LIGHT_CLUSTERS_Z: u32 = 24u;
const LIGHT_CLUSTERS: u32 = 3456u;

// Lights one cluster holds at most; any more reaching it go unlit there.
const LIGHT_CLUSTER_CAPACITY: u32 = 64u;

// How far along the view a slice starts: exponentially from the near plane to the far one.
fn light_slice_depth(lights: Lights, slice: f32) -> f32 {
  return lights.near * pow(lights.far / lights.near, slice / f32(LIGHT_CLUSTERS_Z));
}

// The cluster a point of the frame falls in: its tile across the screen, the first row the top one, and its slice.
fn light_cluster(lights: Lights, screen: vec2<f32>, depth: f32) -> u32 {
  let tile: vec2<f32> = clamp(
    floor(screen * vec2<f32>(f32(LIGHT_CLUSTERS_X), f32(LIGHT_CLUSTERS_Y))),
    vec2<f32>(0.0),
    vec2<f32>(f32(LIGHT_CLUSTERS_X - 1u), f32(LIGHT_CLUSTERS_Y - 1u))
  );
  let slice: f32 = clamp(
    floor(log(depth / lights.near) / log(lights.far / lights.near) * f32(LIGHT_CLUSTERS_Z)),
    0.0,
    f32(LIGHT_CLUSTERS_Z - 1u)
  );

  return u32(tile.x) + u32(tile.y) * LIGHT_CLUSTERS_X + u32(slice) * LIGHT_CLUSTERS_X * LIGHT_CLUSTERS_Y;
}
