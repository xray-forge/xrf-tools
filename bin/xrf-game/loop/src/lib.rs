#![doc = include_str!("../README.md")]

pub(crate) mod clock;
pub(crate) mod feed;
pub(crate) mod loop_stepper;
pub(crate) mod record;
pub(crate) mod state;
pub(crate) mod thread;

#[cfg(test)]
mod tests;

pub use crate::clock::loop_clock::LoopClock;
pub use crate::clock::loop_time::LoopTime;
pub use crate::feed::loop_event::LoopEvent;
pub use crate::feed::loop_events::LoopEvents;
pub use crate::feed::loop_feed::LoopFeed;
pub use crate::feed::loop_frame::LoopFrame;
pub use crate::feed::loop_read::LoopRead;
pub use crate::feed::loop_status::LoopStatus;
pub use crate::loop_stepper::LoopStepper;
pub use crate::record::loop_command_log::LoopCommandLog;
pub use crate::record::loop_logged_command::LoopLoggedCommand;
pub use crate::record::loop_session::LoopSession;
pub use crate::record::loop_witness::LoopWitness;
pub use crate::state::loop_blend::LoopBlend;
pub use crate::state::loop_state::LoopState;
pub use crate::state::loop_system::LoopSystem;
pub use crate::thread::game_loop::GameLoop;
