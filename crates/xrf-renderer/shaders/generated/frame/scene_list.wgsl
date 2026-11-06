// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(5) @binding(0) var<storage, read> trace_requests: array<TraceRequest>;
@group(5) @binding(1) var<storage, read_write> trace_request_head: array<u32>;
