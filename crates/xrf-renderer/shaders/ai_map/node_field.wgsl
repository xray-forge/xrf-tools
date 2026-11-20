#import "common/camera"
#import "common/debug_colours"
#import "common/fullscreen"

// A debug view of a level's AI map (`level.ai`): each drawn pixel's node, found by the cell the pixel stands in as
// `CLevelGraph::vertex_id` packs one and, among that cell's nodes, the one whose height is nearest the pixel's within
// reach of it; and the field the view shows, its sixteen steps as greys or its missing links as a colour, the cells'
// edges darkened so neighbouring nodes tell apart. Written over the light target with one in alpha where a node is
// shown, nothing where none is.

#import "generated/ai_map/node_field"

// `AiMapField`.
const FIELD_HIGH_COVER: u32 = 0u;
const FIELD_LOW_COVER: u32 = 1u;
const FIELD_LINKS: u32 = 2u;

// Metres below a node's height and above it a surface still shows the node: its ground, a little off its plane, and
// what stands low on it. What stands higher, a roof or a floor over the node, shows none.
const NODE_BELOW: f32 = 0.5;
const NODE_ABOVE: f32 = 2.0;

// How much of a node's value its cell's edge keeps, over a band one drawn pixel wide; edges are drawn only where a pixel
// spans at most this share of a cell.
const CELL_EDGE_KEPT: f32 = 0.6;
const CELL_EDGE_SPAN: f32 = 0.25;

// The mean of four 4-bit covers, the first in the lowest bits, from 0 to 1.
fn cover_mean(covers: u32) -> f32 {
  let sum: u32 = (covers & 0xFu) + ((covers >> 4u) & 0xFu) + ((covers >> 8u) & 0xFu) + ((covers >> 12u) & 0xFu);

  return f32(sum) / 60.0;
}

@fragment
fn fs_node_field(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);
  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let world: vec3<f32> = (transpose(camera.view) * vec4<f32>(position, 0.0)).xyz + camera.position.xyz;
  // The engine's ground plane in cells from the grid's corner, its `z` the renderer's negated; a node stands at its
  // cell's centre, which the half moves to a whole number.
  let grid: vec2<f32> = (vec2<f32>(world.x, -world.z) - ai_map.origin.xy) / ai_map.origin.z + 0.5;
  // Taken before any branch, where every pixel of the quad still runs.
  let pixel_cells: vec2<f32> = max(fwidth(grid), vec2<f32>(1e-6));

  if (depth <= 0.0) {
    return vec4<f32>(0.0);
  }

  let cell: vec2<i32> = vec2<i32>(floor(grid));
  let row_length: i32 = i32(ai_map.cells.x);

  if (any(cell < vec2<i32>(0)) || cell.x >= i32(ai_map.cells.y) || cell.y >= row_length) {
    return vec4<f32>(0.0);
  }

  let entry: u32 = cells[u32(cell.x * row_length + cell.y)];
  let first: u32 = entry >> 8u;
  var found: u32 = 0xFFFFFFFFu;
  var nearest: f32 = NODE_ABOVE + NODE_BELOW;

  for (var index: u32 = first; index < first + (entry & 0xFFu); index++) {
    let height: f32 = f32(nodes[index * 2u] & 0xFFFFu) / 65535.0 * ai_map.height + ai_map.origin.w;
    let above: f32 = world.y - height;

    if (above >= -NODE_BELOW && above <= NODE_ABOVE && abs(above) < nearest) {
      nearest = abs(above);
      found = index;
    }
  }

  if (found == 0xFFFFFFFFu) {
    return vec4<f32>(0.0);
  }

  var shown: vec3<f32>;

  switch (ai_map.cells.z) {
    case FIELD_HIGH_COVER: {
      shown = vec3<f32>(cover_mean(nodes[found * 2u + 1u] & 0xFFFFu));
    }
    case FIELD_LOW_COVER: {
      shown = vec3<f32>(cover_mean(nodes[found * 2u + 1u] >> 16u));
    }
    case FIELD_LINKS, default: {
      shown = missing_links_colour(countOneBits((nodes[found * 2u] >> 16u) & 0xFu));
    }
  }

  let inside: vec2<f32> = fract(grid);
  let from_edge: vec2<f32> = min(inside, 1.0 - inside) / pixel_cells;

  if (max(pixel_cells.x, pixel_cells.y) < CELL_EDGE_SPAN && min(from_edge.x, from_edge.y) < 1.0) {
    shown *= CELL_EDGE_KEPT;
  }

  return vec4<f32>(shown, 1.0);
}
