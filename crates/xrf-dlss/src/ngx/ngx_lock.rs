use std::sync::{Mutex, MutexGuard, PoisonError};

/// NGX's one lock in the process: its methods are not thread-safe whatever device they name (`nvsdk_ngx.h`), so every
/// call into NGX is made holding it.
static NGX: Mutex<()> = Mutex::new(());

/// Holds NGX's lock until the guard drops. Never taken while it is held: a guard is taken only where NGX is entered.
pub(crate) fn lock_ngx() -> MutexGuard<'static, ()> {
  NGX.lock().unwrap_or_else(PoisonError::into_inner)
}
