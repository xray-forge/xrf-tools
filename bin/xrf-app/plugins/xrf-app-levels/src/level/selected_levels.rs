use xrf_app_core::session::Sessions;
use xrf_level_source::SelectedLevel;

/// The levels open, each held by the tool that opened it, read again on the pool once a commit moved one of their own
/// files.
pub type SelectedLevels = Sessions<SelectedLevel>;
