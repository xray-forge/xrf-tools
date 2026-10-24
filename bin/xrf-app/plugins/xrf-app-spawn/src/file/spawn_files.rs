use xrf_app_core::session::Sessions;

use crate::SpawnSession;

/// The spawns open in the editors, each held by the tool that opened it.
pub type SpawnFiles = Sessions<SpawnSession>;
