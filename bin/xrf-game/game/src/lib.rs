//! The game binary's library: how this program is put together (its arguments, directories and logging, the game's
//! systems composed into the loop's state, and that state fed to the world), kept apart from `main` so it is tested.
//! What the game draws with is `xrf-game-settings`'.

pub(crate) mod arguments;
pub(crate) mod directories;
pub(crate) mod logging;
pub(crate) mod state;
#[cfg(test)]
mod tests;

pub use crate::arguments::game_arguments::GameArguments;
pub use crate::directories::game_directories::GameDirectories;
pub use crate::logging::game_log_setup::GameLogSetup;
pub use crate::state::game_command::GameCommand;
pub use crate::state::game_frame::GameFrame;
pub use crate::state::game_state::GameState;
pub use crate::state::game_view_feed::GameViewFeed;
