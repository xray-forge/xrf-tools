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

  /// Opens the game's window, centred on the main display and shown.
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
      .build()
      .map_err(|error| format!("The window cannot be opened: {error}"))?;
    let mut window: PlatformWindow = PlatformWindow::new(window, self.context.mouse(), self.video.text_input());

    if request.mode != PlatformWindowMode::Windowed {
      window.apply(PlatformWindowCommand::SetMode { mode: request.mode })?;
    }

    Ok(window)
  }

  /// Now on the clock events are stamped with, to the millisecond: the time since SDL started.
  pub fn get_time(&self) -> Duration {
    Duration::from_millis(sdl3::timer::ticks())
  }

  /// Appends every event waiting to `events`, oldest first, opening each gamepad as it is connected.
  pub fn poll(&mut self, events: &mut Vec<PlatformEvent>) {
    for event in self.pump.poll_iter() {
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
