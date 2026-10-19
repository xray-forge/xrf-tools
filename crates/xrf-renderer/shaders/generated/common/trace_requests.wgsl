// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(3) @binding(0) var<storage, read_write> trace_requests: array<TraceRequest>;
@group(3) @binding(1) var<storage, read_write> trace_request_head: array<atomic<u32>>;
@group(3) @binding(2) var<storage, read> trace_request_waits: array<u32>;
