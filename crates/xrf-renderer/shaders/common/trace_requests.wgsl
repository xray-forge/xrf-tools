#import "common/reflection_trace"
#import "generated/common/trace_requests"

// The list a pass leaves the rays it cannot answer in for the scene pass (`TraceRequestParameters`, group 3), the
// surfaces' trace and the water's reflection alike.

// What the screen answered for a request's ray, packed as it carries it: `answer`'s colour already weighed by its
// `share`, then the share.
fn trace_request_screen(answer: vec3<f32>, share: f32) -> vec2<u32> {
  return vec2<u32>(pack2x16float(answer.rg * share), pack2x16float(vec2<f32>(answer.b * share, share)));
}

// The turn of `turns` a pixel traces on: its place in an 8 × 8 ordered dither, so each turn takes
// exactly one in `turns` of them, every one within a 4 × 2 block. Blue noise read from the compressed map bunched them,
// a turn taking up to a quarter, past the list's room, and the pixels drawn last never traced.
fn trace_turn_of(cell: vec2<u32>, turns: u32) -> u32 {
  let x: u32 = cell.x & 7u;
  let mixed: u32 = x ^ (cell.y & 7u);
  let place: u32 = ((mixed & 1u) << 5u) | ((x & 1u) << 4u) | ((mixed & 2u) << 2u) | ((x & 2u) << 1u) |
    ((mixed & 4u) >> 1u) | ((x & 4u) >> 2u);

  return place * turns / 64u;
}

// What a list's pixels stretch their turns by this frame, so the rays they ask fit its room (`keep_trace_wait`).
fn trace_request_wait() -> u32 {
  return clamp(atomicLoad(&trace_request_head[TRACE_REQUEST_WAIT]), 1u, TRACE_REQUEST_WAIT_MOST);
}

// Lists a request, counting the scene pass's dispatch a workgroup at a time; whether it was listed. A request past the
// list's room is not, and its pass holds what it reflected the frame before: the list fills in the order pixels are
// drawn, so a pixel dropped to the sky drew a straight edge along a triangle.
fn list_trace_request(request: TraceRequest) -> bool {
  let index: u32 = atomicAdd(&trace_request_head[0], 1u);

  if (index >= arrayLength(&trace_requests)) {
    return false;
  }

  if (index % TRACE_REQUEST_WORKGROUP == 0u) {
    atomicAdd(&trace_request_head[1], 1u);
  }

  trace_requests[index] = request;

  return true;
}
