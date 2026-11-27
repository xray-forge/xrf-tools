use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_error::XrfResult;
use xrf_game_console::{ConsoleChoice, ConsoleCompletion};
use xrf_game_developer::{DeveloperFrame, DeveloperLayer, DeveloperReadout, DeveloperSuggestion, DeveloperView};
use xrf_game_loop::LoopFrame;
use xrf_game_movement::{MovementCamera, MovementHeading};
use xrf_game_platform::{PlatformEvent, PlatformSize, PlatformWindow};
use xrf_logging::LogTail;
use xrf_math::glam::Vec3;
use xrf_renderer::RenderFrameReport;

use crate::layer::game_input_fate::GameInputFate;
use crate::layer::game_layer::GameLayer;
use crate::layer::game_layer_context::GameLayerContext;
use crate::run::game_running::GameRunning;
use crate::state::game_frame::GameFrame;

/// The developer layer as the game runs it, over every other layer: the console's drawer, which takes the keyboard and
/// the mouse while open and runs its lines through the game's console, the log's records shown in it, the frame's
/// readout with the loop's tick and the camera, and the load's view while one runs. It is built about 30 times a
/// second, sooner once it took input.
pub(crate) struct GameDeveloper {
  layer: DeveloperLayer,
  /// The log's records the console shows, taken as they come.
  log: Arc<LogTail>,
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
  /// the mouse moving over it cannot build it at every event.
  const INPUT_PERIOD: Duration = Duration::from_millis(8);

  /// The layer with its console closed, showing `log`'s records.
  pub(crate) fn new(log: Arc<LogTail>) -> Self {
    Self {
      layer: DeveloperLayer::default(),
      log,
      sent: None,
      is_changed: false,
      frame_report: None,
    }
  }

  /// Lists `suggestions` under the console's field.
  fn suggest(&mut self, suggestions: Vec<DeveloperSuggestion>) {
    self.layer.get_console_mut().suggest(suggestions);
    self.is_changed = true;
  }

  /// Replaces the line being typed with `line`, as completed.
  fn complete(&mut self, line: String) {
    self.layer.get_console_mut().complete(line);
    self.is_changed = true;
  }

  /// Adds `lines` at the bottom of the console, shown at the next build.
  fn push_lines(&mut self, lines: impl IntoIterator<Item = String>) {
    let console = self.layer.get_console_mut();

    for line in lines {
      console.push(line);
      self.is_changed = true;
    }
  }

  /// How long after the layer was sent it is built again.
  fn get_period(&self) -> Duration {
    if self.is_changed {
      Self::INPUT_PERIOD
    } else {
      Self::PERIOD
    }
  }

  /// `window` as the layer draws over it now.
  fn read_view(window: &PlatformWindow) -> DeveloperView {
    let PlatformSize { width, height } = window.get_size();

    DeveloperView {
      width,
      height,
      scale: window.get_scale(),
      pixel_density: window.get_pixel_density(),
    }
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

impl GameLayer<GameLayerContext<'_>> for GameDeveloper {
  /// The console key always, and keys, text and the mouse while the console is open; opening it seizes the keyboard and
  /// the mouse, closing it gives them back.
  fn take_event(&mut self, event: &PlatformEvent, context: &mut GameLayerContext<'_>) -> GameInputFate {
    let was_open: bool = self.layer.is_open();

    if !self.layer.take(event, Self::read_view(context.window)) {
      return GameInputFate::Passed;
    }

    self.is_changed = true;

    match (was_open, self.layer.is_open()) {
      (false, true) => GameInputFate::Seized,
      (true, false) => GameInputFate::Yielded,
      _ => GameInputFate::Kept,
    }
  }

  /// Runs the lines entered into the drawer, completes the line Tab asked to and suggests what the line typed may
  /// become, and takes the load's picture as it arrives.
  fn update(&mut self, context: &mut GameLayerContext<'_>) -> XrfResult<()> {
    for line in self.layer.get_console_mut().take_entered() {
      context.run_console(&line);
    }

    if let Some(line) = self.layer.get_console_mut().take_completing() {
      self.complete(context.complete_console(&line).line);
    }

    // After a completion, so the line it left is suggested from at once.
    if let Some(line) = self.layer.get_console_mut().take_typed() {
      let ConsoleCompletion { choices, .. } = context.complete_console(&line);

      self.suggest(
        choices
          .into_iter()
          .map(|ConsoleChoice { name, hint }| DeveloperSuggestion { text: name, hint })
          .collect(),
      );
    }

    if let Some(picture) = context.flow.take_loading_picture() {
      self.layer.set_loading_picture(&picture);
    }

    Ok(())
  }

  /// Shows the lines run through the console, whichever layer entered them, and the log's records since, then builds
  /// the layer and has the view draw it, when its period has passed or sooner once it took input: the frame's readout
  /// with the loop's tick in the top left, the camera in the bottom right, or the load's view over the whole window
  /// while the level loads.
  fn draw(&mut self, context: &mut GameLayerContext<'_>) {
    self.push_lines(context.take_console_lines());
    self.push_lines(self.log.take());

    let now: Instant = Instant::now();
    let period: Duration = self.get_period();

    if self.sent.is_some_and(|sent| now.duration_since(sent) < period) {
      return;
    }

    self.sent = Some(now);
    self.is_changed = false;

    if let Some(report) = context.view.take_frame_report() {
      self.frame_report = Some(report);
    }

    let latest: Option<Arc<LoopFrame<GameFrame>>> = context.flow.get_running().and_then(GameRunning::get_latest);
    let mut lines: Vec<String> = DeveloperReadout::get_lines(self.frame_report.as_deref());

    lines.push(
      latest
        .as_deref()
        .map_or_else(|| String::from("Starting"), |frame| format!("Tick {}", frame.tick)),
    );

    let camera: Vec<String> = latest
      .as_deref()
      .map_or_else(Vec::new, |frame| Self::get_camera_lines(frame.state.camera));
    let frame: DeveloperFrame = self.layer.build(
      Self::read_view(context.window),
      &lines,
      &camera,
      context.flow.get_loading_view(),
    );

    context.view.draw_layer(frame);
  }

  /// When the layer is next built: now before its first, else a period after the last.
  fn get_due(&self) -> Option<Instant> {
    Some(self.sent.map_or_else(Instant::now, |sent| sent + self.get_period()))
  }
}
