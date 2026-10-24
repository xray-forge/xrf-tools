//! What plugins know of the application's windows: the one a command was invoked from, their native handles, and which
//! of them a close was approved for.

mod invoking_window;
mod window_close_approvals;
mod window_handles;

#[cfg(test)]
mod tests;

pub use invoking_window::InvokingWindow;
pub use window_close_approvals::WindowCloseApprovals;
pub use window_handles::WindowHandles;
