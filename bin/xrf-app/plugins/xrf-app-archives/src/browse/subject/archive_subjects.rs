use xrf_app_core::session::Sessions;

use crate::browse::ArchiveSubject;

/// What each explorer has open, held by the tool that opened it.
pub type ArchiveSubjects = Sessions<ArchiveSubject>;
