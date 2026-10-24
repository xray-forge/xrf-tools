use xrf_dialog::DialogProject;

use crate::core::session::Sessions;

/// The dialogs projects open, each held by the tool that opened it.
pub type DialogProjectState = Sessions<DialogProject>;
