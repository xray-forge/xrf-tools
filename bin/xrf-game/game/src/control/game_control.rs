use std::collections::VecDeque;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use image::{ImageFormat, RgbaImage};
use serde_json::{Value, json};
use xrf_error::{XrfError, XrfResult};
use xrf_game_console::ConsoleAnswer;
use xrf_game_mcp::{GameEndpoint, GameEvent, GameInput, GameRequest, GameRequestKind, GameResponse};
use xrf_game_movement::{MovementCamera, MovementCommand};
use xrf_game_platform::PlatformEvent;
use xrf_math::glam::Vec3;
use xrf_renderer::RenderCapture;
use xrf_utils::StagedFile;

use crate::control::game_input_steps::GameInputSteps;
use crate::control::game_status::GameStatus;
use crate::control::scheduled_input::ScheduledInput;
use crate::control::waiting_answer::WaitingAnswer;
use crate::layer::game_layer::GameLayer;
use crate::layer::game_layer_context::GameLayerContext;
use crate::run::game_running::GameRunning;
use crate::state::game_command::GameCommand;

/// The game's answers to the control endpoint, as the topmost presentation layer: where it stands, the camera stood
/// where asked, input played as the window would deliver it (fed to every layer with the window's own), a console line
/// run as the drawer runs it, the window's frame written to a file, and a quit. Requests are taken once a frame on the
/// main thread.
pub(crate) struct GameControl {
  endpoint: GameEndpoint,
  level: String,
  captures: Option<PathBuf>,
  /// Whether the bridge was told the level stood ready.
  is_announced: bool,
  /// Input played and not yet delivered, in the order it falls due.
  inputs: VecDeque<ScheduledInput>,
  /// Input requests answered once their last input is delivered, in the order they came.
  waiting: VecDeque<WaitingAnswer>,
  /// When input was last delivered up to.
  delivered: Option<Instant>,
  /// When the input played so far runs out, its last wait included; the next request's input starts there.
  scheduled_until: Option<Instant>,
}

impl GameControl {
  /// How long a capture waits for the next frame: a minimised window draws none.
  const CAPTURE_WAIT: Duration = Duration::from_secs(10);

  /// How long a status waits for the loop to read it between two ticks.
  const STATUS_WAIT: Duration = Duration::from_secs(1);

  /// Answers `endpoint`'s requests for the game showing `level`, writing captures to `captures`.
  pub(crate) fn new(endpoint: GameEndpoint, level: &str, captures: Option<PathBuf>) -> Self {
    Self {
      endpoint,
      level: level.to_owned(),
      captures,
      is_announced: false,
      inputs: VecDeque::new(),
      waiting: VecDeque::new(),
      delivered: None,
      scheduled_until: None,
    }
  }

  /// Tells the bridge the level is ready once it first stands ready, answers input requests whose input is all
  /// delivered, then answers every request waiting; a quit ends play and leaves the requests after it.
  fn serve(&mut self, context: &mut GameLayerContext<'_>) {
    if !self.is_announced && context.view.is_ready() {
      self.is_announced = true;
      self.endpoint.announce(&GameEvent::Ready {
        level: self.level.clone(),
      });
    }

    self.answer_delivered();

    while let Some(GameRequest { id, kind }) = self.endpoint.take_request() {
      if kind == GameRequestKind::Quit {
        self
          .endpoint
          .answer(&GameResponse::succeed(&id, json!({ "quitting": true })));
        context.quit();

        return;
      }

      if let GameRequestKind::Input { inputs } = kind {
        if let Err(error) = self.schedule(id.clone(), inputs) {
          self.endpoint.answer(&GameResponse::fail(&id, error.to_string()));
        }

        continue;
      }

      // A refused line is answered in the console's own words, as the drawer shows it.
      if let GameRequestKind::Console { line } = kind {
        let answer: ConsoleAnswer = context.run_console(&line);

        self.endpoint.answer(&if answer.is_failed {
          GameResponse::fail(&id, answer.lines.join("\n"))
        } else {
          GameResponse::succeed(&id, json!({ "lines": answer.lines }))
        });

        continue;
      }

      let response: GameResponse = match self.answer(context, kind) {
        Ok(result) => GameResponse::succeed(&id, result),
        Err(error) => GameResponse::fail(&id, error.to_string()),
      };

      self.endpoint.answer(&response);
    }
  }

  fn answer(&self, context: &GameLayerContext<'_>, kind: GameRequestKind) -> XrfResult<Value> {
    match kind {
      GameRequestKind::Status => self.describe(context),
      GameRequestKind::Pose { position, target } => {
        let camera: MovementCamera = MovementCamera::looking_at(Vec3::from(position), Vec3::from(target));

        // The look faces the pose's way, which the simulation is told with the camera.
        context.flow.get_look().face(camera.get_heading());

        if !context.send(GameCommand::Movement(MovementCommand::SetCamera(camera))) {
          return Err(XrfError::new_invalid_error("the game loop stopped"));
        }

        Ok(json!({ "position": position, "target": target }))
      }
      GameRequestKind::Capture => self.capture(context.view.capture()),
      GameRequestKind::Input { .. } | GameRequestKind::Console { .. } | GameRequestKind::Quit => Ok(Value::Null),
    }
  }

  /// Plays `inputs` once those played before have run out, their waits included, the request `id` answered once its
  /// own have.
  fn schedule(&mut self, id: String, inputs: Vec<GameInput>) -> XrfResult<()> {
    let now: Instant = Instant::now();
    let mut due: Instant = self.scheduled_until.map_or(now, |until| until.max(now));
    let scheduled: Vec<ScheduledInput> = GameInputSteps::read(inputs, &mut due)?;

    self.scheduled_until = Some(due);
    self.waiting.push_back(WaitingAnswer {
      id,
      due,
      events: scheduled.len(),
    });
    self.inputs.extend(scheduled);

    Ok(())
  }

  /// Answers the input requests whose last input was delivered.
  fn answer_delivered(&mut self) {
    let Some(delivered) = self.delivered else {
      return;
    };

    while let Some(answer) = self.waiting.pop_front_if(|answer| answer.due <= delivered) {
      self.endpoint.answer(&GameResponse::succeed(
        &answer.id,
        json!({ "delivered": answer.events }),
      ));
    }
  }

  /// Where the game stands, read from its state between two of the loop's ticks.
  fn describe(&self, context: &GameLayerContext<'_>) -> XrfResult<Value> {
    let running: &GameRunning = context
      .flow
      .get_running()
      .ok_or_else(|| XrfError::new_not_found_error("the game is still opening; no loop runs yet"))?;
    let status: GameStatus = running
      .snapshot(GameStatus::read)
      .recv_timeout(Self::STATUS_WAIT)
      .map_err(|_| XrfError::new_unexpected_error("the game loop did not answer its status"))?;
    let camera: MovementCamera = status.camera;
    let minutes: u32 = (status.time_of_day / 60.0) as u32;

    Ok(json!({
      "level": self.level,
      "tick": status.tick,
      "timeOfDay": format!("{:02}:{:02}", minutes / 60, minutes % 60),
      "objects": status.objects,
      "online": status.online,
      "squads": status.squads,
      "members": status.members,
      "camera": {
        "position": camera.position.to_array(),
        "forward": camera.get_forward().to_array(),
      },
    }))
  }

  /// Waits for the frame `answer` brings and writes it to a PNG file in the captures folder, answering its path.
  fn capture(&self, answer: Receiver<XrfResult<RenderCapture>>) -> XrfResult<Value> {
    let folder: &PathBuf = self
      .captures
      .as_ref()
      .ok_or_else(|| XrfError::new_not_found_error("the game has no folder to write captures to"))?;
    let capture: RenderCapture = answer
      .recv_timeout(Self::CAPTURE_WAIT)
      .map_err(|_| XrfError::new_not_found_error("the window drew no frame to capture"))??;
    let stamp: u128 = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap_or_default()
      .as_millis();
    let path: PathBuf = folder.join(format!("capture-{stamp}-{}.png", capture.frame));
    let width: u32 = capture.width;
    let height: u32 = capture.height;
    let image: RgbaImage = RgbaImage::from_raw(width, height, capture.pixels)
      .ok_or_else(|| XrfError::new_invalid_error("the capture's pixels do not fill its size"))?;
    let mut png: Vec<u8> = Vec::new();

    image
      .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
      .map_err(|error| XrfError::new_invalid_error(format!("the capture cannot be encoded: {error}")))?;
    StagedFile::write_creating_directories(&path, &png)?;

    Ok(json!({ "path": path.display().to_string(), "width": width, "height": height }))
  }
}

impl GameLayer<GameLayerContext<'_>> for GameControl {
  /// Appends the played input that fell due, for every layer to be offered as the window's own.
  fn feed(&mut self, at: Duration, events: &mut Vec<PlatformEvent>) {
    let now: Instant = Instant::now();

    while let Some(input) = self.inputs.pop_front_if(|input| input.due <= now) {
      events.push(PlatformEvent { at, kind: input.kind });
    }

    self.delivered = Some(now);
  }

  fn update(&mut self, context: &mut GameLayerContext<'_>) -> XrfResult<()> {
    self.serve(context);

    Ok(())
  }

  /// When the next played input falls due, none while none waits; a request arriving wakes the main thread itself.
  fn get_due(&self) -> Option<Instant> {
    self.inputs.front().map(|input| input.due)
  }
}
