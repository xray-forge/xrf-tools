use std::sync::Arc;

use crate::project::verify::ltx_verify_event::LtxVerifyEvent;

/// What hears a verification's events: called on the run's own thread, once per event, in the order they happen.
///
/// Shared rather than boxed, so the options carrying it stay cloneable like every run's.
pub type LtxVerifyListener = Arc<dyn Fn(&LtxVerifyEvent<'_>) + Send + Sync>;
