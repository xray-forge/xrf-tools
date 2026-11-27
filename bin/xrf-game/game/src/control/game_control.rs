use std::cell::{Cell, RefCell, RefMut};
use std::collections::VecDeque;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use image::{ImageFormat, RgbaImage};
use serde_json::{Value, json};
use xrf_error::{XrfError, XrfResult};
use xrf_game_console::ConsoleAnswer;
use xrf_game_loop::LoopFrame;
use xrf_game_mcp::{GameEndpoint, GameEvent, GameInput, GameRequest, GameRequestKind, GameResponse};
use xrf_game_movement::{MovementCamera, MovementCommand};
use xrf_game_platform::PlatformEvent;
use xrf_math::glam::Vec3;
use xrf_renderer::RenderCapture;
use xrf_utils::StagedFile;

use crate::control::game_input_steps::GameInputSteps;
use crate::control::scheduled_input::ScheduledInput;
use crate::control::waiting_answer::WaitingAnswer;
use crate::run::game_play::GamePlay;
use crate::run::game_running::GameRunning;
use crate::state::game_command::GameCommand;
use crate::state::game_frame::GameFrame;

/// The game's answers to the control endpoint: where it stands, the camera stood where asked, input played as the
/// window would deliver it, a console line run as the drawer runs it, the window's frame written to a file, and a quit.
/// Requests are taken between frames on the main thread.
pub(crate) struct GameControl {
  endpoint: GameEndpoint,
  level: String,
  captures: Option<PathBuf>,
  /// Whether the bridge was told the level stood ready.
  is_announced: Cell<bool>,
  /// Input played and not yet delivered, in the order it falls due.
  inputs: RefCell<VecDeque<ScheduledInput>>,
  /// Input requests answered once their last input is delivered, in the order they came.
  waiting: RefCell<VecDeque<WaitingAnswer>>,
  /// When input was last delivered up to.
  delivered: Cell<Option<Instant>>,
  /// When the input played so far runs out, its last wait included; the next request's input starts there.
  scheduled_until: Cell<Option<Instant>>,
}

impl GameControl {
  /// How long a capture waits for the next frame: a minimised window draws none.
  const CAPTURE_WAIT: Duration = Duration::from_secs(10);

  /// Answers `endpoint`'s requests for the game showing `level`, writing captures to `captures`.
  pub(crate) fn new(endpoint: GameEndpoint, level: &str, captures: Option<PathBuf>) -> Self {
    Self {
      endpoint,
      level: level.to_owned(),
      captures,
      is_announced: Cell::new(false),
      inputs: RefCell::new(VecDeque::new()),
      waiting: RefCell::new(VecDeque::new()),
      delivered: Cell::new(None),
      scheduled_until: Cell::new(None),
    }
  }

  /// Appends the played input due by `now` to `events`, stamped `at` on the platform's clock, for the game to take as
  /// it takes the window's.
  pub(crate) fn take_inputs(&self, now: Instant, at: Duration, events: &mut Vec<PlatformEvent>) {
    let mut inputs: RefMut<'_, VecDeque<ScheduledInput>> = self.inputs.borrow_mut();

    while let Some(input) = inputs.pop_front_if(|input| input.due <= now) {
      events.push(PlatformEvent { at, kind: input.kind });
    }

    self.delivered.set(Some(now));
  }

  /// Tells the bridge the level is ready once it first stands ready, answers input requests whose input is all
  /// delivered, then answers every request waiting, answering whether the game plays on: a quit ends it.
  pub(crate) fn serve(&self, play: &mut GamePlay<'_>) -> bool {
    if !self.is_announced.get() && play.view.is_ready() {
      self.is_announced.set(true);
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

        return false;
      }

      if let GameRequestKind::Input { inputs } = kind {
        if let Err(error) = self.schedule(id.clone(), inputs) {
          self.endpoint.answer(&GameResponse::fail(&id, error.to_string()));
        }

        continue;
      }

      // A refused line is answered in the console's own words, as the drawer shows it.
      if let GameRequestKind::Console { line } = kind {
        let answer: ConsoleAnswer = play.run_console(&line);

        self.endpoint.answer(&if answer.is_failed {
          GameResponse::fail(&id, answer.lines.join("\n"))
        } else {
          GameResponse::succeed(&id, json!({ "lines": answer.lines }))
        });

        continue;
      }

      let response: GameResponse = match self.answer(play, kind) {
        Ok(result) => GameResponse::succeed(&id, result),
        Err(error) => GameResponse::fail(&id, error.to_string()),
      };

      self.endpoint.answer(&response);
    }

    true
  }

  fn answer(&self, play: &GamePlay<'_>, kind: GameRequestKind) -> XrfResult<Value> {
    match kind {
      GameRequestKind::Status => self.describe(play),
      GameRequestKind::Pose { position, target } => {
        let camera: MovementCamera = MovementCamera::looking_at(Vec3::from(position), Vec3::from(target));

        if !play.send(GameCommand::Movement(MovementCommand::SetCamera(camera))) {
          return Err(XrfError::new_invalid_error("the game loop stopped"));
        }

        Ok(json!({ "position": position, "target": target }))
      }
      GameRequestKind::Capture => self.capture(play.view.capture()),
      GameRequestKind::Input { .. } | GameRequestKind::Console { .. } | GameRequestKind::Quit => Ok(Value::Null),
    }
  }

  /// Plays `inputs` once those played before have run out, their waits included, the request `id` answered once its
  /// own have.
  fn schedule(&self, id: String, inputs: Vec<GameInput>) -> XrfResult<()> {
    let now: Instant = Instant::now();
    let mut due: Instant = self.scheduled_until.get().map_or(now, |until| until.max(now));
    let scheduled: Vec<ScheduledInput> = GameInputSteps::read(inputs, &mut due)?;

    self.scheduled_until.set(Some(due));
    self.waiting.borrow_mut().push_back(WaitingAnswer {
      id,
      due,
      events: scheduled.len(),
    });
    self.inputs.borrow_mut().extend(scheduled);

    Ok(())
  }

  /// Answers the input requests whose last input was delivered.
  fn answer_delivered(&self) {
    let Some(delivered) = self.delivered.get() else {
      return;
    };
    let mut waiting: RefMut<'_, VecDeque<WaitingAnswer>> = self.waiting.borrow_mut();

    while let Some(answer) = waiting.pop_front_if(|answer| answer.due <= delivered) {
      self.endpoint.answer(&GameResponse::succeed(
        &answer.id,
        json!({ "delivered": answer.events }),
      ));
    }
  }

  fn describe(&self, play: &GamePlay<'_>) -> XrfResult<Value> {
    let frame: LoopFrame<GameFrame> = play
      .flow
      .get_running()
      .and_then(GameRunning::get_latest)
      .ok_or_else(|| XrfError::new_not_found_error("the game is still opening; its loop has published no frame yet"))?;
    let camera: MovementCamera = frame.state.camera;
    let minutes: u32 = (frame.state.time_of_day / 60.0) as u32;

    Ok(json!({
      "level": self.level,
      "tick": frame.tick,
      "timeOfDay": format!("{:02}:{:02}", minutes / 60, minutes % 60),
      "objects": frame.state.objects,
      "online": frame.state.online,
      "squads": frame.state.squads,
      "members": frame.state.members,
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
