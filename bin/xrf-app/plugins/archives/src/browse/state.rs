use xrf_app_core::session::Sessions;

use crate::browse::archive_subject::ArchiveSubject;

/// What each explorer has open, held by the tool that opened it.
pub type ArchiveBrowseState = Sessions<ArchiveSubject>;
