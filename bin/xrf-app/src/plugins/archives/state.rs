use xrf_archive::ArchiveProject;

use crate::core::session::DocumentSession;

/// The committed archive document and its pending replacement.
pub type ArchiveProjectState = DocumentSession<ArchiveProject>;
