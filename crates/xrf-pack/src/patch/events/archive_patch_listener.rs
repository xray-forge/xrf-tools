use crate::patch::events::ArchivePatchEvent;

/// What hears a patch run's events: called on the run's own thread, once per event, in the order they happen.
pub type ArchivePatchListener = Box<dyn Fn(&ArchivePatchEvent<'_>) + Send + Sync>;
