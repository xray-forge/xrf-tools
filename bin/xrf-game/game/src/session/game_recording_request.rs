use std::path::PathBuf;

/// What a load is asked to do with a recording of its session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameRecordingRequest {
  /// Record the session opened to this file, made anew.
  Record(PathBuf),
  /// Step the session recorded in this file again, then play on from its end.
  Replay(PathBuf),
}
