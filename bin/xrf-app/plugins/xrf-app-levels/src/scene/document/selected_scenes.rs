use xrf_app_core::session::Sessions;

use crate::scene::SelectedScene;

/// The open Level Editor scenes, each held by the tool that opened it and read again on the pool once a commit moves
/// one of its files.
pub type SelectedScenes = Sessions<SelectedScene>;
