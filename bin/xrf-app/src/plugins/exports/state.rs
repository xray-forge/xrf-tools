use xrf_export::ExportsProject;

use crate::core::session::Session;

/// The committed exports document and its pending replacement.
pub type ExportsProjectState = Session<ExportsProject>;
