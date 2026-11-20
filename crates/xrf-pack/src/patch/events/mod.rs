//! What a patch run tells whoever is watching: the events it hands out as it decides, and the printer that says them.

mod archive_patch_emitter;
mod archive_patch_event;
mod archive_patch_listener;
mod archive_patch_printer;

pub(crate) use archive_patch_emitter::ArchivePatchEmitter;
pub use archive_patch_event::ArchivePatchEvent;
pub use archive_patch_listener::ArchivePatchListener;
pub use archive_patch_printer::ArchivePatchPrinter;
