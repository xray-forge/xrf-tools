//! What a mounted world actually read, when a caller asked to be told.

#[cfg(test)]
mod tests;
mod xray_read_trace;
mod xray_read_trace_summary;

pub use xray_read_trace::{XrayReadTrace, XrayReadTraceEntry};
pub use xray_read_trace_summary::{XrayReadTraceHotPath, XrayReadTraceSummary};
