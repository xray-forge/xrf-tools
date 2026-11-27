use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use xrf_error::{XrfError, XrfResult};
use xrf_game_loop::{LoopStepper, LoopWitness};

use crate::record::game_recorded_command::GameRecordedCommand;
use crate::record::game_recording_header::GameRecordingHeader;
use crate::record::game_recording_line::GameRecordingLine;
use crate::record::game_state_digest::GameStateDigest;
use crate::state::game_command::GameCommand;
use crate::state::game_state::GameState;

/// Records a session to a file as the loop steps it: each command with the tick that took it, a digest of the state
/// at an interval, and the tick it stopped at as it is dropped. What is written reaches the disk at each digest, so a
/// crash loses a second at most. A file that stops taking lines ends the recording, logged once; the game plays on.
pub struct GameRecorder {
  file: PathBuf,
  /// None once the recording ended.
  writer: Option<BufWriter<File>>,
  /// The tick the session stands at.
  tick: u64,
  commands: usize,
  digests: usize,
  /// The longest a digest took, which the loop's thread spent between two ticks.
  slowest_digest: Duration,
}

impl GameRecorder {
  /// Starts a recording in `file`, made anew with its folder, of a session starting as `header` says.
  ///
  /// # Errors
  ///
  /// Returns why the file cannot be made or written.
  pub fn create(file: &Path, header: &GameRecordingHeader) -> XrfResult<Self> {
    let describe = |error: std::io::Error| {
      XrfError::new_io_error(
        format!("the recording '{}' cannot be written: {error}", file.display()),
        error.kind(),
      )
    };

    if let Some(directory) = file.parent().filter(|directory| !directory.as_os_str().is_empty()) {
      std::fs::create_dir_all(directory).map_err(describe)?;
    }

    let mut recorder: Self = Self {
      file: file.to_path_buf(),
      writer: Some(BufWriter::new(File::create(file).map_err(describe)?)),
      tick: 0,
      commands: 0,
      digests: 0,
      slowest_digest: Duration::ZERO,
    };

    recorder
      .write(&GameRecordingLine::Header(header.clone()))
      .map_err(describe)?;
    log::info!("Recording the session to '{}'", file.display());

    Ok(recorder)
  }

  /// Writes `line`, and with any line but a command sends what is written to the disk.
  fn write(&mut self, line: &GameRecordingLine) -> std::io::Result<()> {
    let Some(writer) = &mut self.writer else {
      return Ok(());
    };

    serde_json::to_writer(&mut *writer, line)?;
    writer.write_all(b"\n")?;

    if matches!(line, GameRecordingLine::Command { .. }) {
      return Ok(());
    }

    writer.flush()
  }

  /// Writes `line`, the recording ended and the reason logged where the file does not take it.
  fn record(&mut self, line: &GameRecordingLine) {
    if let Err(error) = self.write(line) {
      log::error!(
        "The recording '{}' ended at tick {}, its file not written: {error}",
        self.file.display(),
        self.tick
      );
      self.writer = None;
    }
  }

  /// Ends the recording at the tick the session stands at, nothing recorded after.
  fn end(&mut self) {
    if self.writer.is_none() {
      return;
    }

    self.record(&GameRecordingLine::End { tick: self.tick });

    if self.writer.take().is_none() {
      return;
    }

    log::info!(
      "Recorded {} commands and {} state digests over {} ticks to '{}', the slowest digest in {:.1?}",
      self.commands,
      self.digests,
      self.tick,
      self.file.display(),
      self.slowest_digest
    );
  }
}

impl LoopWitness<GameState> for GameRecorder {
  fn see_commands(&mut self, tick: u64, commands: &[GameCommand]) {
    // A tick is kept whole or not at all: a replay stepping half of what a tick took would part from the session.
    let Some(kept) = commands
      .iter()
      .map(GameRecordedCommand::of)
      .collect::<Option<Vec<GameRecordedCommand>>>()
    else {
      log::warn!(
        "The recording '{}' ends at tick {tick}, which took a command it cannot keep",
        self.file.display()
      );
      self.end();

      return;
    };

    for command in kept {
      self.record(&GameRecordingLine::Command { tick, command });
      self.commands += 1;
    }
  }

  fn see_step(&mut self, stepper: &LoopStepper<GameState>) {
    if self.writer.is_none() {
      return;
    }

    self.tick = stepper.get_time().tick;

    if !self.tick.is_multiple_of(GameStateDigest::INTERVAL) {
      return;
    }

    let started: Instant = Instant::now();
    let digest: GameStateDigest = GameStateDigest::of(self.tick, stepper.get_state());

    self.slowest_digest = self.slowest_digest.max(started.elapsed());
    self.digests += 1;
    self.record(&GameRecordingLine::Digest {
      tick: self.tick,
      digest: digest.to_string(),
    });
  }
}

impl Drop for GameRecorder {
  fn drop(&mut self) {
    self.end();
  }
}
