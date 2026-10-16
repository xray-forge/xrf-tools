use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::ngx::bindings::NVSDK_NGX_Handle;

/// A feature dropped while the device may still run what was submitted with it, released once that work is done.
pub(crate) struct NgxRetiredFeature {
  pub(crate) handle: *mut NVSDK_NGX_Handle,
  /// Set once the device has finished the work submitted before the feature was dropped.
  is_done: Arc<AtomicBool>,
}

impl NgxRetiredFeature {
  /// Retires `handle` behind what `queue` was given so far.
  pub(crate) fn new(handle: *mut NVSDK_NGX_Handle, queue: &wgpu::Queue) -> Self {
    let is_done: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let signal: Arc<AtomicBool> = Arc::clone(&is_done);

    queue.on_submitted_work_done(move || signal.store(true, Ordering::Release));

    Self { handle, is_done }
  }

  /// Whether the device has finished what was submitted with it, as the device's last poll found.
  pub(crate) fn is_done(&self) -> bool {
    self.is_done.load(Ordering::Acquire)
  }
}
