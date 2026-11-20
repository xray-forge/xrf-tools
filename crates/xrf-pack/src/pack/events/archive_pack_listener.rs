use crate::pack::events::ArchivePackEvent;

/// What hears a packing run's events: called on the run's own thread, once per event, in the order they happen.
pub type ArchivePackListener = Box<dyn Fn(&ArchivePackEvent<'_>) + Send + Sync>;
