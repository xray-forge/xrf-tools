use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use crate::contract::render_weather_control::RenderWeatherControl;
use crate::contract::render_weather_play::RenderWeatherPlay;
use crate::contract::render_weather_report::RenderWeatherReport;
use crate::contract::render_weather_transition::RenderWeatherTransition;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::lighting::render_lighting::RenderLighting;
use crate::thread::render_workers::RenderWorkers;
use crate::weather::played_weather::PlayedWeather;
use crate::weather::weather_load::WeatherLoad;
use crate::weather::weather_player::WeatherPlayer;

/// How often a running clock's report is sent.
const REPORT_INTERVAL: Duration = Duration::from_millis(100);

/// One viewport's weather: what it was asked to play, read through its level's source on the loader threads, played
/// on, and what the scene is lit by as a result. A request outlives the level it was made for, so one sent before the
/// level is shown plays once it is.
pub struct ViewportWeather {
  player: WeatherPlayer,
  /// What was last asked to play, and how it takes over.
  request: (RenderWeatherPlay, RenderWeatherTransition),
  source: Option<Arc<dyn RenderLevelSource>>,
  /// What every weather of the shown level plays with, once read.
  level: Option<Arc<RenderLevelWeather>>,
  /// Bumped with each level shown and each request, so a read finishing late for an older one is dropped.
  level_generation: u64,
  request_generation: u64,
  sender: Sender<WeatherLoad>,
  receiver: Receiver<WeatherLoad>,
  lighting: RenderLighting,
  sent_report: Option<Option<RenderWeatherReport>>,
  report_due: Instant,
  workers: RenderWorkers,
}

impl ViewportWeather {
  pub fn new(now: Instant, workers: &RenderWorkers) -> Self {
    let (sender, receiver) = channel();

    Self {
      player: WeatherPlayer::default(),
      request: (RenderWeatherPlay::None, RenderWeatherTransition::Cut),
      source: None,
      level: None,
      level_generation: 0,
      request_generation: 0,
      sender,
      receiver,
      lighting: RenderLighting::default(),
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
  pub fn get_level(&self) -> Option<&Arc<RenderLevelWeather>> {
    self.level.as_ref()
  }

  /// Plays the weather of another level, or of none: what it plays with is read anew, and the last request plays
  /// over it once it is.
  pub fn show(&mut self, source: Option<Arc<dyn RenderLevelSource>>) {
    self.level_generation += 1;
    self.level = None;
    self.player.take(None, RenderWeatherTransition::Cut);
    self.source = source;

    let Some(source) = &self.source else {
      return;
    };
    let (sender, source, level) = (self.sender.clone(), Arc::clone(source), self.level_generation);

    self.workers.spawn(move || {
      let read = source.read_weather().map(Arc::new);

      if let Err(error) = &read {
        log::warn!("The level's weather cannot be played: {error}");
      }

      let _ = sender.send(WeatherLoad::Level { level, read });
    });
  }

  /// Plays something else from now on.
  pub fn play(&mut self, play: RenderWeatherPlay, transition: RenderWeatherTransition) {
    self.request = (play, transition);
    self.request_generation += 1;
    self.start();
  }

  pub fn set_control(&mut self, control: RenderWeatherControl) {
    self.player.set_control(control);
  }

  pub fn seek(&mut self, time: f32) {
    self.player.seek(time);
  }

  pub fn play_effect(&mut self, name: Option<&str>) {
    self.player.play_effect(name);
  }

  /// Takes what the loaders read, moves the clock on to a frame and lights the scene by it. `view` is where the camera
  /// stands, in engine space; `is_up` says whether a lighting's skies are up.
  pub fn advance(
    &mut self,
    now: Instant,
    view: [f32; 3],
    is_thundering: bool,
    is_up: impl FnMut(&RenderLighting) -> bool,
  ) {
    while let Ok(load) = self.receiver.try_recv() {
      self.take(load);
    }

    if let Some(lighting) = self.player.advance(now, view, is_thundering, is_up) {
      self.lighting = lighting;
    } else if !self.player.is_playing() && self.lighting != RenderLighting::default() {
      self.lighting = RenderLighting::default();
    }
  }

  /// The weather's report when it changed, at most every [`REPORT_INTERVAL`]; none inside while nothing plays.
  pub fn take_report(&mut self, now: Instant) -> Option<Option<RenderWeatherReport>> {
    if now < self.report_due {
      return None;
    }

    let report: Option<RenderWeatherReport> = self.player.report();

    if self.sent_report.as_ref() == Some(&report) {
      return None;
    }

    self.sent_report = Some(report.clone());
    self.report_due = now + REPORT_INTERVAL;

    Some(report)
  }

  fn take(&mut self, load: WeatherLoad) {
    match load {
      // A level whose configs do not read is lit as though nothing played.
      WeatherLoad::Level { level, read: Ok(read) } if level == self.level_generation => {
        self.level = Some(read);
        self.request.1 = RenderWeatherTransition::Cut;
        self.start();
      }
      WeatherLoad::Cycle {
        level,
        request,
        transition,
        read,
      } if level == self.level_generation && request == self.request_generation => match (read, &self.level) {
        (Ok(keyframes), Some(weather)) => {
          let played: PlayedWeather = PlayedWeather::cycle(keyframes, Arc::clone(weather));

          self.player.take(Some(played), transition);
        }
        (Err(error), _) => log::warn!("The weather cycle cannot be played: {error}"),
        _ => {}
      },
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
      RenderWeatherPlay::None => self.player.take(None, *transition),
      RenderWeatherPlay::Keyframe { keyframe } => {
        let played: PlayedWeather = PlayedWeather::keyframe(keyframe.as_ref().clone(), Arc::clone(level));

        self.player.take(Some(played), *transition);
      }
      RenderWeatherPlay::Cycle { name } => {
        let (sender, source, name, transition) = (self.sender.clone(), Arc::clone(source), name.clone(), *transition);
        let (level, request) = (self.level_generation, self.request_generation);

        self.workers.spawn(move || {
          let read = source.read_weather_cycle(&name);

          let _ = sender.send(WeatherLoad::Cycle {
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
