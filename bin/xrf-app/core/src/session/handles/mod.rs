//! A domain's openings, one a handle, and how a tool's are restored and released.

mod session_release;
mod session_restore;
mod sessions;
mod sessions_state;

pub(crate) use session_release::SessionRelease;
pub use session_restore::SessionRestore;
pub use sessions::Sessions;
