use std::cell::Cell;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use image::{ImageFormat, RgbaImage};
use serde_json::{Value, json};
use xrf_error::{XrfError, XrfResult};
use xrf_game_loop::LoopFrame;
use xrf_game_mcp::{GameEndpoint, GameEvent, GameRequest, GameRequestKind, GameResponse};
use xrf_game_movement::{MovementCamera, MovementCommand};
use xrf_math::glam::Vec3;
use xrf_renderer::RenderCapture;
use xrf_utils::StagedFile;

use crate::run::game_play::GamePlay;
use crate::state::game_command::GameCommand;
use crate::state::game_frame::GameFrame;

/// The game's answers to the control endpoint: where it stands, the camera stood where asked, the window's frame
/// written to a file, and a quit. Requests are taken between frames on the main thread.
pub(crate) struct GameControl {
  endpoint: GameEndpoint,
  level: String,
  captures: Option<PathBuf>,
  /// Whether the bridge was told the level stood ready.
  is_announced: Cell<bool>,
}

impl GameControl {
  /// How long a capture waits for the next frame: a minimised window draws none.
  const CAPTURE_WAIT: Duration = Duration::from_secs(10);

  /// Starts reading requests for the game showing `level`, writing captures to `captures`.
  ///
  /// # Errors
  ///
  /// Returns why the endpoint cannot start.
  pub(crate) fn start(level: &str, captures: Option<PathBuf>) -> XrfResult<Self> {
    Ok(Self {
      endpoint: GameEndpoint::start()?,
      level: level.to_owned(),
      captures,
      is_announced: Cell::new(false),
    })
  }

  /// Tells the bridge the level is ready once it first stands ready, then answers every request waiting, answering
  /// whether the game plays on: a quit ends it.
  pub(crate) fn serve(&self, play: &GamePlay<'_>) -> bool {
    if !self.is_announced.get() && play.view.is_ready() {
      self.is_announced.set(true);
      self.endpoint.announce(&GameEvent::Ready {
        level: self.level.clone(),
      });
    }

    while let Some(GameRequest { id, kind }) = self.endpoint.take_request() {
      if kind == GameRequestKind::Quit {
        self
          .endpoint
          .answer(&GameResponse::succeed(&id, json!({ "quitting": true })));

        return false;
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
      GameRequestKind::Quit => Ok(Value::Null),
    }
  }

  fn describe(&self, play: &GamePlay<'_>) -> XrfResult<Value> {
    let frame: LoopFrame<GameFrame> = play
      .feed
      .get_latest()
      .ok_or_else(|| XrfError::new_not_found_error("the game loop has published no frame yet"))?;
    let camera: MovementCamera = frame.state.camera;
    let minutes: u32 = (frame.time_of_day / 60.0) as u32;

    Ok(json!({
      "level": self.level,
      "tick": frame.tick,
      "timeOfDay": format!("{:02}:{:02}", minutes / 60, minutes % 60),
      "objects": frame.state.objects,
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
