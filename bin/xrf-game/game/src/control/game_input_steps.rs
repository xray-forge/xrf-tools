use std::time::{Duration, Instant};

use xrf_error::{XrfError, XrfResult};
use xrf_game_mcp::GameInput;
use xrf_game_platform::{PlatformEventKind, PlatformMouseButton, PlatformScancode};

use crate::control::scheduled_input::ScheduledInput;

/// An agent's input steps read into platform events, each due when its waits have passed.
pub(crate) struct GameInputSteps;

impl GameInputSteps {
  /// The longest a request's waits may run: under the bridge's wait for its answer.
  pub(crate) const LONGEST: Duration = Duration::from_secs(20);

  /// The events `inputs` play from `due`, which their waits move on to where the last of them ends, a wait after the
  /// last event included.
  ///
  /// # Errors
  ///
  /// Returns why a step cannot be played: a key or button with no such name, or waits longer than `LONGEST`.
  pub(crate) fn read(inputs: Vec<GameInput>, due: &mut Instant) -> XrfResult<Vec<ScheduledInput>> {
    let start: Instant = *due;
    let mut scheduled: Vec<ScheduledInput> = Vec::new();
    let mut push = |due: Instant, kind: PlatformEventKind| scheduled.push(ScheduledInput { due, kind });

    for input in inputs {
      match input {
        GameInput::Press { key } => {
          let scancode: PlatformScancode = Self::read_key(&key)?;

          push(*due, Self::create_key(scancode, true));
          push(*due, Self::create_key(scancode, false));
        }
        GameInput::Down { key } => push(*due, Self::create_key(Self::read_key(&key)?, true)),
        GameInput::Up { key } => push(*due, Self::create_key(Self::read_key(&key)?, false)),
        GameInput::Text { text } => push(*due, PlatformEventKind::Text { text }),
        GameInput::Move { dx, dy, x, y } => push(*due, PlatformEventKind::MouseMoved { x, y, dx, dy }),
        GameInput::Click { x, y, button } => {
          let button: PlatformMouseButton = Self::read_button(button.as_deref())?;

          push(
            *due,
            PlatformEventKind::MouseButton {
              button,
              is_down: true,
              x,
              y,
            },
          );
          push(
            *due,
            PlatformEventKind::MouseButton {
              button,
              is_down: false,
              x,
              y,
            },
          );
        }
        GameInput::Wheel { dy } => push(*due, PlatformEventKind::MouseWheel { dx: 0.0, dy }),
        GameInput::Wait { ms } => {
          *due += Duration::from_millis(ms);

          if due.duration_since(start) > Self::LONGEST {
            return Err(XrfError::new_invalid_error(format!(
              "the waits run past {} s",
              Self::LONGEST.as_secs()
            )));
          }
        }
      }
    }

    Ok(scheduled)
  }

  fn read_key(name: &str) -> XrfResult<PlatformScancode> {
    PlatformScancode::from_name(name).ok_or_else(|| {
      XrfError::new_invalid_error(format!(
        "SDL names no key '{name}'; try `W`, `Grave`, `Return`, `Escape`, `Space`, `Left Shift`, `Up`"
      ))
    })
  }

  fn read_button(name: Option<&str>) -> XrfResult<PlatformMouseButton> {
    match name.unwrap_or("left") {
      "left" => Ok(PlatformMouseButton::Left),
      "right" => Ok(PlatformMouseButton::Right),
      "middle" => Ok(PlatformMouseButton::Middle),
      other => Err(XrfError::new_invalid_error(format!(
        "no mouse button '{other}'; left, right or middle"
      ))),
    }
  }

  fn create_key(scancode: PlatformScancode, is_down: bool) -> PlatformEventKind {
    PlatformEventKind::Key {
      scancode,
      is_down,
      is_repeat: false,
    }
  }
}
