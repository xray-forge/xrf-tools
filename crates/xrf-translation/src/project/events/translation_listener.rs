use std::sync::Arc;

use crate::project::events::translation_event::TranslationEvent;

/// What hears a translation run's events: called on the run's own thread, once per event, in the order they happen.
///
/// Shared rather than boxed, so the options carrying it stay cloneable like every run's.
pub type TranslationListener = Arc<dyn Fn(&TranslationEvent<'_>) + Send + Sync>;
