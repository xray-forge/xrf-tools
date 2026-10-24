use crate::core::session::Sessions;
use crate::plugins::archives::browse::archive_subject::ArchiveSubject;

/// What each explorer has open, held by the tool that opened it.
pub type ArchiveBrowseState = Sessions<ArchiveSubject>;
