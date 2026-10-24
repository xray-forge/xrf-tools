use xrf_app_core::session::Sessions;

use crate::OpenedDialogProject;

/// The dialogs projects open, each held by the tool that opened it.
pub type DialogProjects = Sessions<OpenedDialogProject>;
