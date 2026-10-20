// The marks the G-buffer's material target holds in its alpha, a bit each: what is selected, which the present
// outlines, what is self-lit, whose light the sun pass fills, what is lit as foliage, which passes light through from
// behind, what is terrain, which reflects the sky where a reflection meets nothing, and what is a plant (grass and
// leaves, however lit), which rain does not wet and which reflects little, what moved by itself since the last frame
// (swayed by the wind), whose shading its motion does not fully carry, and what is grass, which casts contact shadows
// only as far as the grass's shadows reach.
const MARK_SELECTED: u32 = 128u;
const MARK_EMISSIVE: u32 = 64u;
const MARK_FLORA: u32 = 32u;
const MARK_TERRAIN: u32 = 16u;
const MARK_PLANT: u32 = 8u;
// A steep surface the enhanced parallax marched, whose sun term is its relief's shadow, which the sun pass takes.
const MARK_RELIEF: u32 = 4u;
const MARK_MOVED: u32 = 2u;
const MARK_GRASS: u32 = 1u;
// The least a surface's own movement since the last frame, in metres, that counts as moved.
const MOVED_THRESHOLD: f32 = 1e-4;

fn encode_marks(
  is_selected: bool,
  is_emissive: bool,
  is_flora: bool,
  is_terrain: bool,
  is_relief: bool,
  is_plant: bool,
  has_moved: bool,
) -> f32 {
  return f32(select(0u, MARK_SELECTED, is_selected) | select(0u, MARK_EMISSIVE, is_emissive) |
    select(0u, MARK_FLORA, is_flora) | select(0u, MARK_TERRAIN, is_terrain) | select(0u, MARK_RELIEF, is_relief) |
    select(0u, MARK_PLANT, is_plant) | select(0u, MARK_MOVED, has_moved)) /
    255.0;
}

// Whether a surface moved by itself as far as `moved` since the last frame counts as moved.
fn is_moved(moved: vec3<f32>) -> bool {
  return dot(moved, moved) > MOVED_THRESHOLD * MOVED_THRESHOLD;
}

fn has_mark(marks: f32, mark: u32) -> bool {
  return (u32(round(marks * 255.0)) & mark) != 0u;
}
