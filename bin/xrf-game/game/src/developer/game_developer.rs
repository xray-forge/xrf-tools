use std::time::{Duration, Instant};

use xrf_game_developer::{DeveloperFrame, DeveloperLayer, DeveloperReadout, DeveloperView};
use xrf_game_loop::LoopFrame;
use xrf_game_movement::{MovementCamera, MovementHeading};
use xrf_game_platform::PlatformEvent;
use xrf_math::glam::Vec3;
use xrf_renderer::RenderFrameReport;

use crate::render::game_view::GameView;
use crate::state::game_frame::GameFrame;

/// The developer layer as the game runs it: the layer, when it was last sent and whether it took input since, and the
/// renderer's latest frame report, which its readout shows with the loop's tick and the camera.
#[derive(Default)]
pub(crate) struct GameDeveloper {
  layer: DeveloperLayer,
  /// When the layer was last sent, none before the first.
  sent: Option<Instant>,
  /// Whether the layer took input since it was last sent.
  is_changed: bool,
  /// The latest frame report the renderer sent, shown until the next.
  frame_report: Option<Box<RenderFrameReport>>,
}

impl GameDeveloper {
  /// How often the layer is built and sent: about 30 times a second, which its readouts need.
  const PERIOD: Duration = Duration::from_millis(33);

  /// How soon it is built again once it took input: about a frame at 120 Hz, so the console answers a key at once and
  /// the mouse moving over it cannot build it every poll.
  const INPUT_PERIOD: Duration = Duration::from_millis(8);

  /// Whether the console is open, taking the keyboard and mouse.
  pub(crate) fn is_open(&self) -> bool {
    self.layer.is_open()
  }

  /// The lines entered into the console since the last take, in order.
  pub(crate) fn take_entered(&mut self) -> Vec<String> {
    self.layer.get_console_mut().take_entered()
  }

  /// The line Tab asked to complete since the last take.
  pub(crate) fn take_completing(&mut self) -> Option<String> {
    self.layer.get_console_mut().take_completing()
  }

  /// Replaces the line being typed with `line`, as completed.
  pub(crate) fn complete(&mut self, line: String) {
    self.layer.get_console_mut().complete(line);
    self.is_changed = true;
  }

  /// Adds `lines` at the bottom of the console, shown at the next build.
  pub(crate) fn push_lines(&mut self, lines: impl IntoIterator<Item = String>) {
    let console = self.layer.get_console_mut();

    for line in lines {
      console.push(line);
      self.is_changed = true;
    }
  }

  /// Takes a platform event in `view`, answering whether the layer kept it from the game.
  pub(crate) fn take(&mut self, event: &PlatformEvent, view: DeveloperView) -> bool {
    let is_taken: bool = self.layer.take(event, view);

    self.is_changed |= is_taken;

    is_taken
  }

  /// Builds the layer over `view` and has `game_view` draw it, when its period has passed or sooner once it took input:
  /// the frame's readout with `latest`'s tick in the top left, `latest`'s camera in the bottom right.
  pub(crate) fn send(&mut self, game_view: &GameView, view: DeveloperView, latest: Option<LoopFrame<GameFrame>>) {
    let now: Instant = Instant::now();
    let period: Duration = if self.is_changed {
      Self::INPUT_PERIOD
    } else {
      Self::PERIOD
    };

    if self.sent.is_some_and(|sent| now.duration_since(sent) < period) {
      return;
    }

    self.sent = Some(now);
    self.is_changed = false;

    if let Some(report) = game_view.take_frame_report() {
      self.frame_report = Some(report);
    }

    let mut lines: Vec<String> = DeveloperReadout::get_lines(self.frame_report.as_deref());

    lines.push(latest.map_or_else(|| String::from("Starting"), |frame| format!("Tick {}", frame.tick)));

    let camera: Vec<String> = latest.map_or_else(Vec::new, |frame| Self::get_camera_lines(frame.state.camera));
    let frame: DeveloperFrame = self.layer.build(view, &lines, &camera);

    game_view.draw_layer(frame);
  }

  /// Where `camera` stands and which way it faces, as xrf-app's camera readout writes them: the engine's axes, then the
  /// heading and pitch in degrees.
  fn get_camera_lines(camera: MovementCamera) -> Vec<String> {
    let position: Vec3 = camera.position;
    let heading: MovementHeading = camera.get_heading();

    vec![
      format!("x {:.1} y {:.1} z {:.1}", position.x, position.y, position.z),
      format!(
        "h {:.1}° p {:.1}°",
        heading.yaw.to_degrees(),
        heading.pitch.to_degrees()
      ),
    ]
  }
}
