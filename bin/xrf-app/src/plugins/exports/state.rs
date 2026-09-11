use xrf_export::ExportsProject;

use crate::core::session::DocumentSession;

/// The committed exports document and its pending replacement.
pub type ExportsProjectState = DocumentSession<ExportsProject>;
