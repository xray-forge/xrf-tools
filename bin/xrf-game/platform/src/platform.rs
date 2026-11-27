use std::collections::HashMap;
use std::time::Duration;

use sdl3::event::Event;
use sdl3::gamepad::Gamepad;
use sdl3::video::Window;
use sdl3::{EventPump, GamepadSubsystem, Sdl, VideoSubsystem};

use crate::event::platform_event::PlatformEvent;
use crate::platform_library::PlatformLibrary;
use crate::window::platform_window::PlatformWindow;
use crate::window::platform_window_command::PlatformWindowCommand;
use crate::window::platform_window_mode::PlatformWindowMode;
use crate::window::platform_window_request::PlatformWindowRequest;

/// The game's platform: SDL started, its window opened and its events read, on the thread that started it, which
/// SDL takes for the main thread. It is the only door to SDL, so the library is loaded before anything calls into it.
pub struct Platform {
  context: Sdl,
  video: VideoSubsystem,
  gamepads: GamepadSubsystem,
  pump: EventPump,
  /// The gamepads connected, by the id their events carry: an unopened pad sends none.
  opened: HashMap<u32, Gamepad>,
}

impl Platform {
  /// Starts SDL with its video and gamepads.
  ///
  /// # Errors
  ///
  /// Returns why SDL cannot start: its library is not found or does not load, or a subsystem fails.
  pub fn start() -> Result<Self, String> {
    PlatformLibrary::load()?;

    let context: Sdl = sdl3::init().map_err(|error| format!("SDL cannot start: {error}"))?;
    let video: VideoSubsystem = context
      .video()
      .map_err(|error| format!("SDL's video cannot start: {error}"))?;
    let gamepads: GamepadSubsystem = context
      .gamepad()
      .map_err(|error| format!("SDL's gamepads cannot start: {error}"))?;
    let pump: EventPump = context
      .event_pump()
      .map_err(|error| format!("SDL's events cannot be read: {error}"))?;

    log::info!(
      "SDL {} started, video on {}",
      sdl3::version::version(),
      video.current_video_driver()
    );

    Ok(Self {
      context,
      video,
      gamepads,
      pump,
      opened: HashMap::new(),
    })
  }

  /// Opens the game's window, centred on the main display and hidden until `PlatformWindowCommand::Show`: the
  /// renderer takes seconds to draw its first frame, and a window shown before then shows whatever the system paints.
  ///
  /// # Errors
  ///
  /// Returns why SDL cannot open it, or cannot give it the mode asked.
  pub fn open_window(&self, request: &PlatformWindowRequest) -> Result<PlatformWindow, String> {
    let window: Window = self
      .video
      .window(&request.title, request.size.width, request.size.height)
      .resizable()
      .high_pixel_density()
      .position_centered()
      .hidden()
      .build()
      .map_err(|error| format!("The window cannot be opened: {error}"))?;
    let mut window: PlatformWindow = PlatformWindow::new(window, self.context.mouse(), self.video.text_input());

    if request.mode != PlatformWindowMode::Windowed {
      window.apply(PlatformWindowCommand::SetMode { mode: request.mode })?;
    }

    Ok(window)
  }

  /// The whole milliseconds SDL is asked to wait so that it waits `timeout` at least. SDL waits the whole milliseconds
  /// left of what it was asked once it has looked at its queue, so a millisecond short: asked one, it returns at once,
  /// and a caller waiting out the last of a millisecond would spin.
  pub(crate) fn get_wait_milliseconds(timeout: Duration) -> u32 {
    if timeout.is_zero() {
      return 0;
    }

    u32::try_from(timeout.as_nanos().div_ceil(1_000_000))
      .unwrap_or(u32::MAX)
      .saturating_add(1)
  }

  /// Now on the clock events are stamped with, to the millisecond: the time since SDL started.
  pub fn get_time(&self) -> Duration {
    Duration::from_millis(sdl3::timer::ticks())
  }

  /// Waits for an event until `timeout` has passed, to SDL's whole milliseconds, then appends every event waiting to
  /// `events`, oldest first, opening each gamepad as it is connected. An event waiting already ends the wait at once,
  /// and so does one arriving meanwhile; no timeout reads what waits without waiting.
  pub fn wait(&mut self, timeout: Duration, events: &mut Vec<PlatformEvent>) {
    let first: Option<Event> = self.pump.wait_event_timeout_ms(Self::get_wait_milliseconds(timeout));

    for event in first.into_iter().chain(self.pump.poll_iter()) {
      match &event {
        Event::GamepadAdded { which, .. } => match self.gamepads.open(*which) {
          Ok(gamepad) => {
            log::info!("Gamepad {which} connected: {}", gamepad.name().unwrap_or_default());
            self.opened.insert(which.raw(), gamepad);
          }
          Err(error) => log::warn!("Gamepad {which} cannot be opened: {error}"),
        },
        Event::GamepadRemoved { which, .. } => {
          self.opened.remove(&which.raw());
        }
        _ => {}
      }

      events.extend(PlatformEvent::read(event));
    }
  }
}
