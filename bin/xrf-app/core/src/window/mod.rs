//! What plugins know of the application's windows: the one a command was invoked from, their native handles, which of
//! them a close was approved for, and which still wait hidden for their page.

mod held_window;
mod invoking_window;
mod window_close_approvals;
mod window_handles;
mod window_reveals;

#[cfg(test)]
mod tests;

pub use invoking_window::InvokingWindow;
pub use window_close_approvals::WindowCloseApprovals;
pub use window_handles::WindowHandles;
pub use window_reveals::WindowReveals;
