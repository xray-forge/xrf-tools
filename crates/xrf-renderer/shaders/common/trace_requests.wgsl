#import "common/reflection_trace"
#import "common/trace_turns"
#import "generated/common/trace_requests"

// The list a pass leaves the rays it cannot answer in for the scene pass (`TraceRequestParameters`, group 3), the
// surfaces' trace and the water's reflection alike.

// What the screen answered for a request's ray, packed as it carries it: `answer`'s colour already weighed by its
// `share`, then the share.
fn trace_request_screen(answer: vec3<f32>, share: f32) -> vec2<u32> {
  return vec2<u32>(pack2x16float(answer.rg * share), pack2x16float(vec2<f32>(answer.b * share, share)));
}

// What a list's pixels stretch their turns by this frame, so the rays they ask fit its room (`keep_trace_wait`), as the
// head held it when the frame started.
fn trace_request_wait() -> u32 {
  return clamp(trace_request_waits[0], 1u, TRACE_REQUEST_WAIT_MOST);
}

// Lists a request, counting the scene pass's dispatch a workgroup at a time; whether it was listed. A request past the
// list's room is not, and its pass holds what it reflected the frame before: the list fills in the order pixels are
// drawn, so a pixel dropped to the sky drew a straight edge along a triangle.
fn list_trace_request(request: TraceRequest) -> bool {
  let index: u32 = atomicAdd(&trace_request_head[0], 1u);

  if (index >= arrayLength(&trace_requests)) {
    return false;
  }

  // A new workgroup's first request widens the scene pass's dispatch to it and the head's spare one, laid in rows a
  // dimension can hold, as `compute_index` reads them back.
  if (index % TRACE_REQUEST_WORKGROUP == 0u) {
    let groups: u32 = index / TRACE_REQUEST_WORKGROUP + 2u;

    atomicMax(&trace_request_head[1], min(groups, TRACE_REQUEST_ROW));
    atomicMax(&trace_request_head[2], (groups + TRACE_REQUEST_ROW - 1u) / TRACE_REQUEST_ROW);
  }

  trace_requests[index] = request;

  return true;
}
