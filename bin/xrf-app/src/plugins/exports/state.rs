use xrf_export::ExportsProject;

use crate::core::session::Sessions;

/// The exports projects open, each held by the tool that opened it.
pub type ExportsProjectState = Sessions<ExportsProject>;
