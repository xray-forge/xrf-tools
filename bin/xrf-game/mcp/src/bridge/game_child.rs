use std::io::ErrorKind;
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use xrf_error::{XrfError, XrfResult};

use crate::bridge::game_channel::GameChannel;

/// A game the bridge started as its child, the line protocol over its standard input and output. Its standard error is
/// not piped: its log file holds every record. Dropped, it closes the game's input, which quits it, and kills it only
/// where it does not quit in time.
pub(crate) struct GameChild {
  child: Child,
  pub(crate) channel: GameChannel,
}

impl GameChild {
  /// How long a dropped or quitting game is given to exit before it is killed.
  const EXIT_WAIT: Duration = Duration::from_secs(15);

  /// How often a wait for the game's exit looks again.
  const EXIT_POLL: Duration = Duration::from_millis(50);

  /// Starts `executable` with `arguments` and its control endpoint on.
  ///
  /// # Errors
  ///
  /// Returns why the game cannot be started.
  pub(crate) fn start(executable: &Path, arguments: &[String]) -> XrfResult<Self> {
    let mut child: Child = Command::new(executable)
      .args(arguments)
      .arg("--mcp")
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::null())
      .spawn()
      .map_err(|error| {
        XrfError::new_io_error(format!("{} cannot start: {error}", executable.display()), error.kind())
      })?;
    let output: ChildStdout = child
      .stdout
      .take()
      .ok_or_else(|| XrfError::new_io_error("the game's output is not piped", ErrorKind::BrokenPipe))?;
    let input: ChildStdin = child
      .stdin
      .take()
      .ok_or_else(|| XrfError::new_io_error("the game's input is not piped", ErrorKind::BrokenPipe))?;

    Ok(Self {
      child,
      channel: GameChannel::new(Box::new(input), output, "xrf-game-mcp-output")?,
    })
  }

  /// Whether the game still runs.
  pub(crate) fn is_running(&mut self) -> bool {
    matches!(self.child.try_wait(), Ok(None))
  }

  /// Quits the game: closes its input, which it reads as a quit, and waits for it to exit, killing it where it does not.
  ///
  /// # Errors
  ///
  /// Returns why its exit could not be waited for.
  pub(crate) fn quit(mut self) -> XrfResult<ExitStatus> {
    self.close_and_wait()
  }

  fn close_and_wait(&mut self) -> XrfResult<ExitStatus> {
    self.channel.close_input();

    let deadline: Instant = Instant::now() + Self::EXIT_WAIT;

    loop {
      if let Some(status) = self.child.try_wait()? {
        return Ok(status);
      }

      if Instant::now() >= deadline {
        log::warn!("The game did not exit within {:?}; killing it", Self::EXIT_WAIT);
        self.child.kill()?;

        return Ok(self.child.wait()?);
      }

      thread::sleep(Self::EXIT_POLL);
    }
  }
}

impl Drop for GameChild {
  fn drop(&mut self) {
    if self.is_running()
      && let Err(error) = self.close_and_wait()
    {
      log::warn!("The game could not be stopped: {error}");
    }
  }
}
