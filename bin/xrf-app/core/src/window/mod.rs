//! What plugins know of the application's windows: their native handles, and which of them a close was approved for.

mod window_close_approvals;
mod window_handles;

#[cfg(test)]
mod tests;

pub use window_close_approvals::WindowCloseApprovals;
pub use window_handles::WindowHandles;
