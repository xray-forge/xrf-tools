use std::collections::VecDeque;

use xrf_error::{XrfError, XrfResult};

use crate::record::loop_logged_command::LoopLoggedCommand;

/// A session's commands, each with the tick that took it, in the order they were taken: all a session's outcome
/// depends on beside the state it started from, since a step reads no clock. Stepping the same state through it again
/// takes every command at its tick and reaches the same state. It ends at a tick, the first it says nothing of: a
/// session may run on long after its last command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoopCommandLog<C> {
  commands: VecDeque<LoopLoggedCommand<C>>,
  end: u64,
}

impl<C> Default for LoopCommandLog<C> {
  fn default() -> Self {
    Self {
      commands: VecDeque::new(),
      end: 0,
    }
  }
}

impl<C> LoopCommandLog<C> {
  /// Notes that `tick` took `command`, after every command noted before; the log then ends past that tick at least.
  ///
  /// # Errors
  ///
  /// Returns an invalid error for a tick before the last noted, which no session takes commands in.
  pub fn push(&mut self, tick: u64, command: C) -> XrfResult<()> {
    if let Some(last) = self.commands.back()
      && tick < last.tick
    {
      return Err(XrfError::new_invalid_error(format!(
        "a command log notes tick {tick} after tick {}",
        last.tick
      )));
    }

    self.commands.push_back(LoopLoggedCommand { tick, command });
    self.end_at(tick + 1);

    Ok(())
  }

  /// Ends the log at `tick`, the tick its session stopped at, unless it already holds a command of that tick or later.
  pub fn end_at(&mut self, tick: u64) {
    self.end = self.end.max(tick);
  }

  /// The first tick the log says nothing of.
  pub fn get_end(&self) -> u64 {
    self.end
  }

  /// How many commands wait to be taken.
  pub fn len(&self) -> usize {
    self.commands.len()
  }

  pub fn is_empty(&self) -> bool {
    self.commands.is_empty()
  }

  /// Takes the commands `tick` took, in order, with any of ticks before it not taken yet.
  pub fn take_at(&mut self, tick: u64) -> impl Iterator<Item = C> + '_ {
    std::iter::from_fn(move || {
      if self.commands.front()?.tick > tick {
        return None;
      }

      self.commands.pop_front().map(|logged| logged.command)
    })
  }
}
