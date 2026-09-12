use xrf_archive::ArchiveProject;

use crate::core::session::Session;

/// The committed archive document and its pending replacement.
pub type ArchiveProjectState = Session<ArchiveProject>;
