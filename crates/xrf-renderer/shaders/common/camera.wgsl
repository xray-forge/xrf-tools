#import "common/marks"

// What every pass drawing a viewport knows of its camera and the frame's switches (`CameraUniform`).
#import "generated/structs"

@group(0) @binding(0) var<uniform> camera: Camera;

// How much a surface's texture footprint is scaled to read its textures at the camera's mip bias: its derivatives times
// this pick the mip `output.y` away.
fn texture_footprint_scale() -> f32 {
  return exp2(camera.output.y);
}

// Whether a cluster drawn in a place is what the selection names.
fn is_selected(entry: vec2<u32>) -> bool {
  let selection: vec4<u32> = camera.selection;

  if (selection.x == 0u || (selection.y != 0xffffffffu && entry.y != selection.y)) {
    return false;
  }

  // The last run starting at or before the cluster, found by halves, holds it if any does.
  var low: u32 = 0u;
  var high: u32 = selection.z;

  while (low < high) {
    let middle: u32 = (low + high) / 2u;

    if (selection_run(middle).x <= entry.x) {
      low = middle + 1u;
    } else {
      high = middle;
    }
  }

  return low > 0u && entry.x < selection_run(low - 1u).y;
}

// One of the selection's runs, by its index.
fn selection_run(index: u32) -> vec2<u32> {
  let row: vec4<u32> = camera.selection_runs[index / 2u];

  return select(row.xy, row.zw, index % 2u == 1u);
}

// The surfaces' bits in `switches.w`: the engines' steep parallax drawn; the parallax, the bumps or the heights drawn
// in place of the albedo for their debug views; the enhanced parallax marched; and the sampled mip drawn in place of
// the albedo for its debug view.
const SURFACES_STEEP: u32 = 1u;
const SURFACES_PARALLAX_SHOWN: u32 = 2u;
const SURFACES_BUMPS_SHOWN: u32 = 4u;
const SURFACES_HEIGHTS_SHOWN: u32 = 8u;
const SURFACES_ENHANCED: u32 = 16u;
const SURFACES_MIP_SHOWN: u32 = 32u;

fn has_surfaces_bit(bit: u32) -> bool {
  return (u32(camera.switches.w) & bit) != 0u;
}

// What the sun and the lamps draw into the light target (`output.z`, `LightView`): their light; for the cascades view
// the sun the cascade it reads, from one, and how much of it reaches, the lamps nothing; for the lamps view the lamps
// their cluster's count, the sun nothing; for the overdraw view neither anything.
const LIGHT_VIEW_LIGHT: u32 = 0u;
const LIGHT_VIEW_CASCADES: u32 = 1u;
const LIGHT_VIEW_LAMPS: u32 = 2u;
const LIGHT_VIEW_OVERDRAW: u32 = 3u;

fn light_view() -> u32 {
  return u32(camera.output.z);
}

// The albedo every surface shares as clay: a mid grey, light enough to read the shading on.
const CLAY_ALBEDO: f32 = 0.5;

// What an untextured surface is drawn: clay, or the tint it is given.
fn untextured_color(tint: vec3<f32>) -> vec3<f32> {
  return select(tint, vec3<f32>(CLAY_ALBEDO), camera.modes.w > 0.5);
}

// How far a surface's point moved on the screen since the last frame, from where it stood then to where it stands
// now, each through its own frame's unjittered camera: in texture coordinates, now less then, `y` down.
fn camera_motion(world: vec3<f32>, previous_world: vec3<f32>) -> vec2<f32> {
  let current: vec4<f32> = camera.motion_current * vec4<f32>(world, 1.0);
  let previous: vec4<f32> = camera.motion_previous * vec4<f32>(previous_world, 1.0);

  return (current.xy / current.w - previous.xy / previous.w) * vec2<f32>(0.5, -0.5);
}

// The point of the view frustum under a viewport position at a depth, reversed: one is the near plane.
fn camera_unproject(ndc: vec2<f32>, depth: f32) -> vec3<f32> {
  let point: vec4<f32> = camera.inverse_view_projection * vec4<f32>(ndc, depth, 1.0);

  return point.xyz / point.w;
}

// Whether the camera's lens has parallel rays, as a level map's: a projection leaving `w` as it is.
fn camera_is_orthographic() -> bool {
  return camera.projection[3][3] == 1.0;
}

// The distance along the view of the point a depth shows, reversed, from the lens's depth terms: a perspective's
// hyperbola, or an orthographic lens's line.
fn camera_view_distance(depth: f32) -> f32 {
  if (camera_is_orthographic()) {
    return (camera.projection[3][2] - depth) / camera.projection[2][2];
  }

  return camera.projection[3][2] / (depth + camera.projection[2][2]);
}

// Where the camera looks through a view space point: from the eye through it, or the view's own forward for an
// orthographic lens, whose rays are parallel.
fn camera_view_ray(position: vec3<f32>) -> vec3<f32> {
  if (camera_is_orthographic()) {
    return vec3<f32>(0.0, 0.0, -1.0);
  }

  return normalize(position);
}

// Where the camera looks through a world point: from the eye through it, or the view's forward in the world for an
// orthographic lens.
fn camera_world_ray(world: vec3<f32>) -> vec3<f32> {
  if (camera_is_orthographic()) {
    return -normalize(vec3<f32>(camera.view[0][2], camera.view[1][2], camera.view[2][2]));
  }

  return normalize(world - camera.position.xyz);
}

// The ground one pixel of the viewport's height spans at a distance along the view: growing with it through a
// perspective, the same at every distance through an orthographic lens.
fn camera_pixel_footprint(distance: f32) -> f32 {
  let pixel: f32 = 2.0 / (camera.projection[1][1] * camera.viewport.y);

  return select(distance * pixel, pixel, camera_is_orthographic());
}

// The view space point under a pixel of the viewport's own targets at a depth, reversed: one is the near plane.
fn camera_view_position(pixel: vec2<f32>, depth: f32) -> vec3<f32> {
  let size: vec2<f32> = camera.viewport.xy;
  let ndc: vec2<f32> = vec2<f32>(pixel.x / size.x * 2.0 - 1.0, 1.0 - pixel.y / size.y * 2.0);
  let point: vec4<f32> = camera.inverse_projection * vec4<f32>(ndc, depth, 1.0);

  return point.xyz / point.w;
}
