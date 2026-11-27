//! The game binary's library: how this program is put together (its arguments, directories and logging, the game's
//! systems composed into the loop's state, and that state fed to the world), kept apart from `main` so it is tested.
//! What the game draws with is `xrf-game-settings`'.

pub(crate) mod arguments;
pub(crate) mod console;
#[cfg(feature = "mcp")]
pub(crate) mod control;
pub(crate) mod developer;
pub(crate) mod directories;
pub(crate) mod host;
pub(crate) mod input;
pub(crate) mod level;
pub(crate) mod load;
pub(crate) mod logging;
pub(crate) mod record;
pub(crate) mod render;
pub(crate) mod run;
pub(crate) mod session;
pub(crate) mod sound;
pub(crate) mod state;
#[cfg(test)]
mod tests;
pub(crate) mod time;

pub use crate::arguments::game_arguments::GameArguments;
pub use crate::arguments::game_mode_argument::GameModeArgument;
pub use crate::directories::game_directories::GameDirectories;
pub use crate::host::game_host::GameHost;
pub use crate::host::game_tree::GameTree;
pub use crate::input::game_action::GameAction;
pub use crate::input::game_bindings::GameBindings;
pub use crate::input::game_controls::GameControls;
pub use crate::input::game_key_names::GameKeyNames;
pub use crate::level::game_level::GameLevel;
pub use crate::logging::game_log_setup::GameLogSetup;
pub use crate::record::game_recorded_command::GameRecordedCommand;
pub use crate::record::game_recorder::GameRecorder;
pub use crate::record::game_recording_header::GameRecordingHeader;
pub use crate::record::game_recording_line::GameRecordingLine;
pub use crate::record::game_replay::GameReplay;
pub use crate::record::game_replay_check::GameReplayCheck;
pub use crate::record::game_state_digest::GameStateDigest;
pub use crate::render::game_bundle::GameBundle;
pub use crate::render::game_view::GameView;
pub use crate::run::game_run::GameRun;
pub use crate::session::game_load_request::GameLoadRequest;
pub use crate::session::game_load_source::GameLoadSource;
pub use crate::session::game_recording_request::GameRecordingRequest;
pub use crate::session::game_session::GameSession;
pub use crate::sound::game_footsteps::GameFootsteps;
pub use crate::sound::game_sound::GameSound;
pub use crate::state::game_command::GameCommand;
pub use crate::state::game_frame::GameFrame;
pub use crate::state::game_level_entry::GameLevelEntry;
pub use crate::state::game_level_state::GameLevelState;
pub use crate::state::game_progress::GameProgress;
pub use crate::state::game_state::GameState;
pub use crate::state::game_view_feed::GameViewFeed;
pub use crate::time::game_clock::GameClock;
pub use crate::time::game_time::GameTime;
