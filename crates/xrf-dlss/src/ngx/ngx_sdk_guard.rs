use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::ngx::ngx_lock::lock_ngx;
use crate::ngx::ngx_sdk::NgxSdk;

/// A device's NGX held for a call into it: the SDK's own mutex, which keeps its parameters, then NGX's lock across the
/// process, always in that order.
pub(crate) struct NgxSdkGuard<'a> {
  sdk: MutexGuard<'a, NgxSdk>,
  _ngx: MutexGuard<'static, ()>,
}

impl<'a> NgxSdkGuard<'a> {
  pub(crate) fn lock(sdk: &'a Mutex<NgxSdk>) -> Self {
    let sdk: MutexGuard<'a, NgxSdk> = sdk.lock().unwrap_or_else(PoisonError::into_inner);

    Self { sdk, _ngx: lock_ngx() }
  }
}

impl Deref for NgxSdkGuard<'_> {
  type Target = NgxSdk;

  fn deref(&self) -> &NgxSdk {
    &self.sdk
  }
}

impl DerefMut for NgxSdkGuard<'_> {
  fn deref_mut(&mut self) -> &mut NgxSdk {
    &mut self.sdk
  }
}
