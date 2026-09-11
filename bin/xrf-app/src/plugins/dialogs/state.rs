use xrf_dialog::DialogProject;

use crate::core::session::DocumentSession;

/// The committed dialogs document and its pending replacement.
pub type DialogProjectState = DocumentSession<DialogProject>;
