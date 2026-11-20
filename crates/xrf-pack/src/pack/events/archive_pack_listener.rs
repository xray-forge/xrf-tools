use std::sync::Arc;

use crate::pack::events::ArchivePackEvent;

/// What hears a packing run's events: called on the run's own thread, once per event, in the order they happen.
///
/// Shared rather than boxed, so the options carrying it stay cloneable like every run's.
pub type ArchivePackListener = Arc<dyn Fn(&ArchivePackEvent<'_>) + Send + Sync>;
