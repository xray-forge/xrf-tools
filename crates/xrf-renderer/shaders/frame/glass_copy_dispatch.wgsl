// Whether the enhanced glass draws anything this frame, read from the cull's glass lists, early and late: the glass
// copy's dispatch over the whole frame where it does, none where it does not, so glass in the frustum that every nearer
// surface hides costs no copy.

// The cull's early and late draw arguments, the frame whose size the copy covers, and the dispatch written.
#import "generated/frame/glass_copy_dispatch"

// Each layout's glass draw list among the cull's arguments (`StaticBatch::get_pass_list`).
override GLASS_LIST_BAKED: u32 = 0u;
override GLASS_LIST_TREE: u32 = 0u;
override GLASS_LIST_MODEL: u32 = 0u;

// The words a draw's indirect arguments take, and where its instance count stands among them.
const DRAW_WORDS: u32 = 4u;
const DRAW_INSTANCES: u32 = 1u;

// The copy's workgroup side, as `frame/glass_copy` declares it.
const COPY_GROUP: u32 = 8u;

// The glass a list draws, early and late together.
fn listed(list: u32) -> u32 {
  return early[list * DRAW_WORDS + DRAW_INSTANCES] + late[list * DRAW_WORDS + DRAW_INSTANCES];
}

@compute @workgroup_size(1)
fn cs_glass_copy_dispatch() {
  let drawn: u32 = listed(GLASS_LIST_BAKED) + listed(GLASS_LIST_TREE) + listed(GLASS_LIST_MODEL);
  let size: vec2<u32> = textureDimensions(scene);
  let groups: vec2<u32> = select(vec2<u32>(0u), (size + COPY_GROUP - 1u) / COPY_GROUP, drawn > 0u);

  dispatch[0] = groups.x;
  dispatch[1] = groups.y;
  dispatch[2] = 1u;
}
