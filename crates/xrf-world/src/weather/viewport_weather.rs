use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use xrf_renderer::{RenderLighting, RenderWorkers};

use crate::contract::weather::world_ambient_report::WorldAmbientReport;
use crate::contract::weather::world_weather_control::WorldWeatherControl;
use crate::contract::weather::world_weather_play::WorldWeatherPlay;
use crate::contract::weather::world_weather_report::WorldWeatherReport;
use crate::contract::weather::world_weather_transition::WorldWeatherTransition;
use crate::host::world_level_source::WorldLevelSource;
use crate::host::world_level_weather::WorldLevelWeather;
use crate::weather::played_weather::PlayedWeather;
use crate::weather::thunder::thunder_play::ThunderPlay;
use crate::weather::weather_load::WeatherLoad;
use crate::weather::weather_player::WeatherPlayer;

/// How often a running clock's report is sent.
const REPORT_INTERVAL: Duration = Duration::from_millis(100);

/// One viewport's weather: what it was asked to play over what its scene read of the level's weathers, its cycles read
/// through the level's source on the loader threads, played on, and what the scene is lit by as a result. A request
/// outlives the level it was made for, so one sent before the level is shown plays once it is.
pub struct ViewportWeather {
  player: WeatherPlayer,
  /// What was last asked to play, and how it takes over.
  request: (WorldWeatherPlay, WorldWeatherTransition),
  source: Option<Arc<dyn WorldLevelSource>>,
  /// What every weather of the shown level plays with, once read.
  level: Option<Arc<WorldLevelWeather>>,
  /// Bumped with each level shown and each request, so a read finishing late for an older one is dropped.
  level_generation: u64,
  request_generation: u64,
  sender: Sender<WeatherLoad>,
  receiver: Receiver<WeatherLoad>,
  lighting: RenderLighting,
  /// Whether `lighting` is the default one nothing playing leaves, so an idle frame need not compare against another.
  is_default_lighting: bool,
  sent_report: Option<Option<WorldWeatherReport>>,
  report_due: Instant,
  workers: RenderWorkers,
}

impl ViewportWeather {
  pub fn new(now: Instant, workers: &RenderWorkers) -> Self {
    let (sender, receiver) = channel();

    Self {
      player: WeatherPlayer::default(),
      request: (WorldWeatherPlay::None, WorldWeatherTransition::Cut),
      source: None,
      level: None,
      level_generation: 0,
      request_generation: 0,
      sender,
      receiver,
      lighting: RenderLighting::default(),
      is_default_lighting: true,
      sent_report: None,
      report_due: now,
      workers: workers.clone(),
    }
  }

  /// What the scene is lit by now.
  pub fn get_lighting(&self) -> &RenderLighting {
    &self.lighting
  }

  pub fn get_player(&self) -> &WeatherPlayer {
    &self.player
  }

  /// What every weather of the shown level plays with, once read.
  pub fn get_level(&self) -> Option<&Arc<WorldLevelWeather>> {
    self.level.as_ref()
  }

  /// Plays the weather of another level, or of none: nothing plays until its scene has read what the level's weathers
  /// play with (`advance`), and the last request plays over it once it has.
  pub fn show(&mut self, source: Option<Arc<dyn WorldLevelSource>>) {
    self.level_generation += 1;
    self.level = None;
    self.player.take(None, WorldWeatherTransition::Cut);
    self.source = source;
  }

  /// Reads its cycles from `source`, a next source of the level shown, from now on; what its weathers play with follows
  /// its scene, which reads it again where a reload names the weather.
  pub fn follow(&mut self, source: Arc<dyn WorldLevelSource>) {
    self.source = Some(source);
  }

  /// Takes what every weather of the level plays with as its scene read it (`level`), where that is another read than
  /// the one it plays: the last request plays over it, the clock kept. While its scene reads it again, it plays on.
  fn follow_level(&mut self, level: Option<&Arc<WorldLevelWeather>>) {
    let Some(level) = level.filter(|level| !self.level.as_ref().is_some_and(|held| Arc::ptr_eq(held, level))) else {
      return;
    };

    // A cycle still read for the level before is dropped once it lands.
    self.level_generation += 1;
    self.level = Some(Arc::clone(level));
    self.request.1 = WorldWeatherTransition::Cut;
    self.start();
  }

  /// Plays something else from now on.
  pub fn play(&mut self, play: WorldWeatherPlay, transition: WorldWeatherTransition) {
    self.request = (play, transition);
    self.request_generation += 1;
    self.start();
  }

  pub fn set_control(&mut self, control: WorldWeatherControl) {
    self.player.set_control(control);
  }

  pub fn seek(&mut self, time: f32) {
    self.player.seek(time);
  }

  pub fn play_effect(&mut self, name: Option<&str>) {
    self.player.play_effect(name);
  }

  /// Takes what its scene read of the level (`level`) and the cycles the loaders read, moves the clock on to a frame and
  /// lights the scene by it, striking as `thunder` lets it. `view` is where the camera stands, in engine space; `is_up`
  /// says whether a lighting's skies are up.
  pub fn advance(
    &mut self,
    now: Instant,
    view: [f32; 3],
    thunder: ThunderPlay,
    level: Option<&Arc<WorldLevelWeather>>,
    is_up: impl FnMut(&RenderLighting) -> bool,
  ) {
    self.follow_level(level);

    while let Ok(load) = self.receiver.try_recv() {
      self.take(load);
    }

    if let Some(lighting) = self.player.advance(now, view, thunder, is_up) {
      self.lighting = lighting;
      self.is_default_lighting = false;
    } else if !self.player.is_playing() && !self.is_default_lighting {
      self.lighting = RenderLighting::default();
      self.is_default_lighting = true;
    }
  }

  /// The weather's report, with the ambient effects' near the camera, when it changed, at most every
  /// [`REPORT_INTERVAL`]; none inside while nothing plays.
  pub fn take_report(
    &mut self,
    now: Instant,
    ambient: Option<WorldAmbientReport>,
  ) -> Option<Option<WorldWeatherReport>> {
    if now < self.report_due {
      return None;
    }

    // Checked once an interval whether or not it changed, so a still weather is not mixed again every frame.
    self.report_due = now + REPORT_INTERVAL;

    let report: Option<WorldWeatherReport> = self
      .player
      .report()
      .map(|report| WorldWeatherReport { ambient, ..report });

    if self.sent_report.as_ref() == Some(&report) {
      return None;
    }

    self.sent_report = Some(report.clone());

    Some(report)
  }

  /// Plays a cycle read for the level and the request it plays now; one read for another is dropped.
  fn take(
    &mut self,
    WeatherLoad {
      level,
      request,
      transition,
      read,
    }: WeatherLoad,
  ) {
    if level != self.level_generation || request != self.request_generation {
      return;
    }

    match (read, &self.level) {
      (Ok(keyframes), Some(weather)) => {
        let played: PlayedWeather = PlayedWeather::cycle(keyframes, Arc::clone(weather));

        self.player.take(Some(played), transition);
      }
      (Err(error), _) => log::warn!("The weather cycle cannot be played: {error}"),
      _ => {}
    }
  }

  /// Plays the request over the level's weather, reading a cycle first; waits for the level's weather to be read.
  fn start(&mut self) {
    let (Some(source), Some(level)) = (&self.source, &self.level) else {
      return;
    };
    let (play, transition) = &self.request;

    match play {
      WorldWeatherPlay::None => self.player.take(None, *transition),
      WorldWeatherPlay::Keyframe { keyframe } => {
        let played: PlayedWeather = PlayedWeather::keyframe(keyframe.as_ref().clone(), Arc::clone(level));

        self.player.take(Some(played), *transition);
      }
      WorldWeatherPlay::Cycle { name } => {
        let (sender, source, name, transition) = (self.sender.clone(), Arc::clone(source), name.clone(), *transition);
        let (level, request) = (self.level_generation, self.request_generation);

        self.workers.spawn(move || {
          let read = source.read_weather_cycle(&name);

          let _ = sender.send(WeatherLoad {
            level,
            request,
            transition,
            read,
          });
        });
      }
    }
  }
}
