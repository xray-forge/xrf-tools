//! What this instance of the application is, and what it currently costs.
//!
//! Three readings with three different lifetimes, which is why they are three commands rather than one: the build is
//! fixed when the binary is compiled, the machine is fixed while the application runs, and the usage is true only for
//! the instant it was read. A surface polling the third would otherwise re-read the first two forever.
//!
//! - [`host_info`] - the machine and the runtime the application found when it started.
//! - [`runtime_snapshot`] - what it costs the machine right now, and how long it has been running.
//! - [`process_tree`] - the processes below this one, which is where the webview's memory actually is.

pub mod commands;
pub mod host_info;
pub mod process_tree;
pub mod runtime_snapshot;
mod state;

pub use host_info::HostInfo;
pub use runtime_snapshot::RuntimeSnapshot;
pub use state::MachineProbeState;
