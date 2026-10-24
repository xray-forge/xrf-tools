use xrf_app_core::session::Sessions;
use xrf_export::ExportsProject;

/// The exports projects open, each held by the tool that opened it.
pub type ExportsProjectState = Sessions<ExportsProject>;
