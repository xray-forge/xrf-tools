//! The game binary's library: how this program is put together (its arguments, directories and logging), kept apart
//! from `main` so it is tested. What the game draws with is `xrf-game-settings`'.

pub(crate) mod arguments;
pub(crate) mod directories;
pub(crate) mod logging;
#[cfg(test)]
mod tests;

pub use crate::arguments::game_arguments::GameArguments;
pub use crate::directories::game_directories::GameDirectories;
pub use crate::logging::game_log_setup::GameLogSetup;
