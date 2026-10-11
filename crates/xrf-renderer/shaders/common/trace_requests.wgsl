#import "common/reflection_trace"
#import "generated/common/trace_requests"

// The list a pass leaves the rays it cannot answer in for the scene pass (`TraceRequestParameters`, group 3), the
// surfaces' trace and the water's reflection alike.

// What the screen answered for a request's ray, packed as it carries it: `answer`'s colour already weighed by its
// `share`, then the share.
fn trace_request_screen(answer: vec3<f32>, share: f32) -> vec2<u32> {
  return vec2<u32>(pack2x16float(answer.rg * share), pack2x16float(vec2<f32>(answer.b * share, share)));
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
