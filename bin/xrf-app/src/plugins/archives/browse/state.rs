use crate::core::session::Session;
use crate::plugins::archives::browse::archive_subject::ArchiveSubject;

/// What the explorer has open, and its pending replacement.
pub type ArchiveBrowseState = Session<ArchiveSubject>;
