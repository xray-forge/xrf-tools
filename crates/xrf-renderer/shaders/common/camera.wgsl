// What every pass drawing a viewport knows of its camera and the frame's switches.
struct Camera {
  view_projection: mat4x4<f32>,
  inverse_view_projection: mat4x4<f32>,
  view: mat4x4<f32>,
  projection: mat4x4<f32>,
  inverse_projection: mat4x4<f32>,
  // xyz: the eye in renderer space.
  position: vec4<f32>,
  // xy: the viewport's size in device pixels; zw: its top left corner in the window.
  viewport: vec4<f32>,
  // The frustum's six planes in renderer space, pointing inward.
  planes: array<vec4<f32>, 6>,
  // x: textured, y: bumped, z: the baked hemisphere's strength; w unused.
  switches: vec4<f32>,
  // x: one where every static surface draws as its triangles' edges; y: times a uv checker repeats in place of every
  // surface's textures, zero for none; z: one where surfaces draw solid, their alpha ignored; w: one where an
  // untextured surface is clay rather than its shader's tint.
  modes: vec4<f32>,
  // World to clip without the jitter, this frame and the last, which a surface's motion is measured by.
  motion_current: mat4x4<f32>,
  motion_previous: mat4x4<f32>,
  // What shows where nothing was drawn and neither the sky nor the fog is; `w` one where it is set.
  backdrop: vec4<f32>,
  // What a surface naming no base texture is drawn; `w` one where it is set, white where it is not.
  plain: vec4<f32>,
  // The backdrop's second colour, and in `w` the side of a square in render pixels; zero for a plain backdrop.
  backdrop_squares: vec4<f32>,
  // x: one while something is selected; y: the place its clusters are drawn in, or any at `0xffffffff`; z: how many runs
  // of clusters follow.
  selection: vec4<u32>,
  // Runs of clusters in order, two a vector: each its first and the one past its last.
  selection_runs: array<vec4<u32>, 32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;

// The marks the G-buffer's material target holds in its alpha, a bit each: what is selected, which the present outlines,
// what is self-lit, whose light the sun pass fills, what is lit as foliage, which passes light through from behind,
// what is terrain, which reflects the sky where a reflection meets nothing, and what is a plant (grass and leaves,
// however lit), which rain does not wet and which reflects little.
const MARK_SELECTED: u32 = 128u;
const MARK_EMISSIVE: u32 = 64u;
const MARK_FLORA: u32 = 32u;
const MARK_TERRAIN: u32 = 16u;
const MARK_PLANT: u32 = 8u;

fn encode_marks(is_selected: bool, is_emissive: bool, is_flora: bool, is_terrain: bool, is_plant: bool) -> f32 {
  return f32(select(0u, MARK_SELECTED, is_selected) | select(0u, MARK_EMISSIVE, is_emissive) |
    select(0u, MARK_FLORA, is_flora) | select(0u, MARK_TERRAIN, is_terrain) | select(0u, MARK_PLANT, is_plant)) /
    255.0;
}

fn has_mark(marks: f32, mark: u32) -> bool {
  return (u32(round(marks * 255.0)) & mark) != 0u;
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

// The view space point under a pixel of the viewport's own targets at a depth, reversed: one is the near plane.
fn camera_view_position(pixel: vec2<f32>, depth: f32) -> vec3<f32> {
  let size: vec2<f32> = camera.viewport.xy;
  let ndc: vec2<f32> = vec2<f32>(pixel.x / size.x * 2.0 - 1.0, 1.0 - pixel.y / size.y * 2.0);
  let point: vec4<f32> = camera.inverse_projection * vec4<f32>(ndc, depth, 1.0);

  return point.xyz / point.w;
}
