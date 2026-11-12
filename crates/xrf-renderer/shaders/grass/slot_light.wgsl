#import "common/camera"
#import "common/fullscreen"
#import "grass/records"

// A debug view of the light the level's detail slots store (`xrDO_Light`'s output): each drawn pixel's slot, found by
// where the pixel stands as `CDetailManager::QueryDB` finds it and kept only where the pixel stands within the slot's
// heights, and the value the view shows as a grey, or the lamps' three as a colour; its sixteen steps as they are
// stored, the slot's edges darkened so neighbouring slots tell apart. Written over the light target with one in alpha
// where a slot is shown, nothing where none is.

#import "generated/grass/slot_light"

// `GrassSlotValue`.
const SLOT_VALUE_SUN: u32 = 0u;
const SLOT_VALUE_HEMI: u32 = 1u;
const SLOT_VALUE_LIGHT: u32 = 2u;

// How much of a slot's value its edge keeps, over a band one drawn pixel wide; edges are drawn only where a pixel spans
// at most this share of a slot, so a tuft's or a wall's silhouette, where the slots jump between pixels, draws none.
const SLOT_EDGE_KEPT: f32 = 0.6;
const SLOT_EDGE_SPAN: f32 = 0.25;

// Metres below a slot's stored base and above its top a surface still shows the slot: its ground, a little off its
// box, and the tufts planted on it. What stands higher, a tree or a roof over the slot, shows none.
const SLOT_BELOW: f32 = 0.5;
const SLOT_ABOVE: f32 = 1.5;

@fragment
fn fs_slot_light(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);
  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let world: vec3<f32> = (transpose(camera.view) * vec4<f32>(position, 0.0)).xyz + camera.position.xyz;
  // The engine's ground plane in slots, its `z` the renderer's negated.
  let slots: vec2<f32> = vec2<f32>(world.x, -world.z) / GRASS_SLOT_METERS;
  // Taken before any branch, where every pixel of the quad still runs.
  let pixel_slots: vec2<f32> = max(fwidth(slots), vec2<f32>(1e-6));

  if (depth <= 0.0) {
    return vec4<f32>(0.0);
  }

  let cell: vec2<i32> = vec2<i32>(floor(slots)) + slot_light.grid.zw;

  if (any(cell < vec2<i32>(0)) || any(cell >= slot_light.grid.xy)) {
    return vec4<f32>(0.0);
  }

  let record: u32 = grid[u32(cell.y * slot_light.grid.x + cell.x)];

  if (record == 0u) {
    return vec4<f32>(0.0);
  }

  let at: u32 = (record - 1u) * GRASS_SLOT_WORDS;
  let heights: vec2<f32> = grass_slot_heights(slot_records[at]);

  if (world.y < heights.x - SLOT_BELOW || world.y > heights.y + SLOT_ABOVE) {
    return vec4<f32>(0.0);
  }

  let word: u32 = slot_records[at + 1u];
  var shown: vec3<f32>;

  switch (slot_light.value) {
    case SLOT_VALUE_SUN: {
      shown = vec3<f32>(grass_slot_light(word, GRASS_LIGHT_SUN));
    }
    case SLOT_VALUE_HEMI: {
      shown = vec3<f32>(grass_slot_light(word, GRASS_LIGHT_HEMI));
    }
    case SLOT_VALUE_LIGHT, default: {
      shown = vec3<f32>(grass_slot_light(word, GRASS_LIGHT_RED), grass_slot_light(word, GRASS_LIGHT_GREEN),
        grass_slot_light(word, GRASS_LIGHT_BLUE));
    }
  }

  let inside: vec2<f32> = fract(slots);
  let from_edge: vec2<f32> = min(inside, 1.0 - inside) / pixel_slots;

  if (max(pixel_slots.x, pixel_slots.y) < SLOT_EDGE_SPAN && min(from_edge.x, from_edge.y) < 1.0) {
    shown *= SLOT_EDGE_KEPT;
  }

  return vec4<f32>(shown, 1.0);
}
