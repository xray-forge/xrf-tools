//! A domain's openings, one a handle, and how a tool's are opened, restored and released.

mod session_opening;
mod session_release;
mod sessions;
mod sessions_state;

#[cfg(test)]
mod tests;

pub use session_opening::SessionOpening;
pub(crate) use session_release::SessionRelease;
pub use sessions::Sessions;
