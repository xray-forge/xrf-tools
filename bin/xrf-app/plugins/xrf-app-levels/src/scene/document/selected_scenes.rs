use xrf_app_core::session::Sessions;

use crate::scene::SelectedScene;

/// The Level Editor scenes open, each held by the tool that opened it, read again on the pool once a commit moved one of
/// the files it was read from.
pub type SelectedScenes = Sessions<SelectedScene>;
