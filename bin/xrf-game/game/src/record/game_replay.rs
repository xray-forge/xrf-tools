use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_game_loop::{LoopCommandLog, LoopSession};

use crate::record::game_recording_header::GameRecordingHeader;
use crate::record::game_recording_line::GameRecordingLine;
use crate::record::game_replay_check::GameReplayCheck;
use crate::state::game_command::GameCommand;
use crate::state::game_state::GameState;

/// A recording read back to be stepped again: what its session started from, its commands by the tick that took
/// each, and the digests its state had on the way.
pub struct GameReplay {
  header: GameRecordingHeader,
  log: LoopCommandLog<GameCommand>,
  digests: BTreeMap<u64, String>,
}

impl GameReplay {
  /// Reads the header of the recording in `file`, which says what a replay of it opens.
  ///
  /// # Errors
  ///
  /// Returns why it has none this build replays: a file that does not read, one that does not begin with a header, or
  /// a header of another format or tick rate.
  pub fn read_header(file: &Path) -> XrfResult<GameRecordingHeader> {
    let mut lines = Self::read_lines(file)?;

    Self::take_header(file, &mut lines)
  }

  /// Reads the recording in `file` whole.
  ///
  /// # Errors
  ///
  /// Returns why it does not replay: no header this build replays, a line that does not read, or lines out of tick
  /// order.
  pub fn read(file: &Path) -> XrfResult<Self> {
    let mut lines = Self::read_lines(file)?;
    let header: GameRecordingHeader = Self::take_header(file, &mut lines)?;
    let mut log: LoopCommandLog<GameCommand> = LoopCommandLog::default();
    let mut digests: BTreeMap<u64, String> = BTreeMap::new();

    for line in lines {
      match line? {
        GameRecordingLine::Header(_) => {
          return Err(XrfError::new_invalid_error(format!(
            "the recording '{}' holds a second header",
            file.display()
          )));
        }
        GameRecordingLine::Command { tick, command } => log.push(tick, command.into())?,
        GameRecordingLine::Digest { tick, digest } => {
          digests.insert(tick, digest);
          // A recording cut short has no end line: it ends at its last digest then.
          log.end_at(tick);
        }
        GameRecordingLine::End { tick } => log.end_at(tick),
      }
    }

    log::info!(
      "Recording '{}' read: {} commands and {} state digests to tick {}",
      file.display(),
      log.len(),
      digests.len(),
      log.get_end()
    );

    Ok(Self { header, log, digests })
  }

  pub fn get_header(&self) -> &GameRecordingHeader {
    &self.header
  }

  /// The tick the recording ends at.
  pub fn get_end(&self) -> u64 {
    self.log.get_end()
  }

  /// The loop session that steps the recording again, each of its digests checked on the way.
  pub fn into_session(self) -> LoopSession<GameState> {
    let end: u64 = self.log.get_end();

    LoopSession {
      replay: Some(self.log),
      witness: Some(Box::new(GameReplayCheck::new(self.digests, end))),
    }
  }

  /// The lines of the recording in `file`, each read as it is reached.
  fn read_lines(file: &Path) -> XrfResult<impl Iterator<Item = XrfResult<GameRecordingLine>> + use<>> {
    let name: String = file.display().to_string();
    let reader: BufReader<File> = BufReader::new(File::open(file).map_err(|error| {
      XrfError::new_io_error(format!("the recording '{name}' cannot be read: {error}"), error.kind())
    })?);

    Ok(
      reader
        .lines()
        .enumerate()
        .filter(|(_, line)| line.as_ref().map_or(true, |line| !line.trim().is_empty()))
        .map(move |(index, line)| {
          let line: String =
            line.map_err(|error| XrfError::new_read_error(format!("the recording '{name}' does not read: {error}")))?;

          serde_json::from_str::<GameRecordingLine>(&line).map_err(|error| {
            XrfError::new_parsing_error(format!(
              "line {} of the recording '{name}' is not a recording's: {error}",
              index + 1
            ))
          })
        }),
    )
  }

  /// The header a recording's lines begin with, checked against what this build replays.
  fn take_header(
    file: &Path,
    lines: &mut impl Iterator<Item = XrfResult<GameRecordingLine>>,
  ) -> XrfResult<GameRecordingHeader> {
    let Some(GameRecordingLine::Header(header)) = lines.next().transpose()? else {
      return Err(XrfError::new_invalid_error(format!(
        "the recording '{}' does not begin with a header",
        file.display()
      )));
    };

    header.check()?;

    Ok(header)
  }
}
