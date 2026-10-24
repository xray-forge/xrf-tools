//! What this instance of the application is, and what it currently costs.
//!
//! Readings with different lifetimes, which is why they are separate commands rather than one: the build is fixed
//! when the binary is compiled, the machine is fixed while the application runs, and the usage is true only for the
//! instant it was read. A surface polling the last would otherwise re-read the first two forever.
//!
//! - [`HostInfo`] - the machine and the runtime the application found when it started.
//! - [`RuntimeSnapshot`] - what it costs the machine right now, and how long it has been running.
//! - [`DescendantUsage`] - the processes below this one, which is where the webview's memory actually is.
//! - [`memory`] - what the backend and each named webview process hold, cheap enough to poll from a status bar.
//! - [`StartupLog`] - the log's first lines: the build, the machine, the process and the variables set.

pub(crate) mod commands;
mod descendant_usage;
mod host_info;
mod machine_probe_state;
pub mod memory;
mod runtime_snapshot;
mod startup_log;

pub use descendant_usage::DescendantUsage;
pub use host_info::HostInfo;
pub use machine_probe_state::MachineProbeState;
pub use runtime_snapshot::{MachineUsage, ProcessUsage, RuntimeSnapshot};
pub use startup_log::StartupLog;
