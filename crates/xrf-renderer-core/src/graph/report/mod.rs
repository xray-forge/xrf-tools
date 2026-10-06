//! What a compile made of a frame graph, for a person or a tool to read.

mod graph_pass_kind;
mod graph_pass_report;
mod graph_report;
mod graph_transient_report;

pub use graph_pass_kind::GraphPassKind;
pub use graph_pass_report::GraphPassReport;
pub use graph_report::GraphReport;
pub use graph_transient_report::GraphTransientReport;
