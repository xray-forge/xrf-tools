use crate::project::events::translation_event::TranslationEvent;

/// What hears a translation run's events: called on the run's own thread, once per event, in the order they happen.
pub type TranslationListener = Box<dyn Fn(&TranslationEvent<'_>) + Send + Sync>;
