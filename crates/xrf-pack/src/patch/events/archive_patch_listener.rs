use std::sync::Arc;

use crate::patch::events::ArchivePatchEvent;

/// What hears a patch run's events: called on the run's own thread, once per event, in the order they happen.
///
/// Shared rather than boxed, so the options carrying it stay cloneable like every run's.
pub type ArchivePatchListener = Arc<dyn Fn(&ArchivePatchEvent<'_>) + Send + Sync>;
