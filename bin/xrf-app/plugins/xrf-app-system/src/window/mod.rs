//! The window as its page sees it: a close the page asked about first and agreed to, and what the page decided about
//! leaving, which only the page knows.

pub(crate) mod commands;
mod leave_decision;

pub use leave_decision::LeaveDecision;
