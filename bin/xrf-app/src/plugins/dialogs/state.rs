use xrf_dialog::DialogProject;

use crate::core::session::Session;

/// The committed dialogs document and its pending replacement.
pub type DialogProjectState = Session<DialogProject>;
