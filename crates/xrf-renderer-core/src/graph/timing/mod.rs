//! The GPU time of each pass, measured by the graph itself.

mod gpu_zones;
mod graph_pass_time;
mod graph_timer;
mod timer_slot;

pub use graph_pass_time::GraphPassTime;
pub use graph_timer::GraphTimer;
