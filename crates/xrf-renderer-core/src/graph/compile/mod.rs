//! Compiling a declared frame graph: culling, ordering the lifetimes, pooling the transients, merging render passes and
//! cutting the encode groups, and then executing what that made.

mod compiled_graph;
mod compiled_pass;
mod encode_group;
mod graph_compile_options;
mod graph_compiler;
mod transient_slot;

pub use compiled_graph::CompiledGraph;
pub use encode_group::EncodeGroup;
pub use graph_compile_options::GraphCompileOptions;
pub(crate) use graph_compiler::GraphCompiler;
pub use transient_slot::TransientSlot;
