use xrf_app_core::session::Sessions;
use xrf_dialog::DialogProject;

/// The dialogs projects open, each held by the tool that opened it.
pub type DialogProjectState = Sessions<DialogProject>;
