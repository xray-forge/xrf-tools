use std::time::{Duration, Instant};

use xrf_environment::{
  WeatherEffectStart, WeatherEffectTimeline, WeatherMix, WeatherMixPoint, WeatherMixer, WeatherPair,
  WeatherPlayedKeyframe, WeatherTime,
};
use xrf_renderer::RenderLighting;

use crate::contract::world_weather_control::WorldWeatherControl;
use crate::contract::world_weather_effect_report::WorldWeatherEffectReport;
use crate::contract::world_weather_report::WorldWeatherReport;
use crate::contract::world_weather_transition::WorldWeatherTransition;
use crate::weather::played_weather::PlayedWeather;
use crate::weather::playing_effect::PlayingEffect;
use crate::weather::weather_fader::WeatherFader;
use crate::weather::weather_lighting::to_weather_lighting;
use crate::weather::weather_thunder::WeatherThunder;
use crate::weather::weather_thundered_lighting::to_thundered_lighting;

/// Seconds from midnight to noon.
const NOON: f32 = 12.0 * 60.0 * 60.0;

/// The longest real step the clock takes at once, so a view shown again after a while does not skip hours.
const LONGEST_STEP: Duration = Duration::from_secs(1);

/// Metres the view moves before the modifiers are weighed again.
const VIEW_STEP: f32 = 0.5;

/// The weather a viewport plays by itself: its clock, the engine's pair of keyframes kept from frame to frame, the
/// effect over it, the mix seen from the view, and a fade from what was shown when another weather is handed over.
pub struct WeatherPlayer {
  weather: Option<PlayedWeather>,
  control: WorldWeatherControl,
  /// Seconds since midnight.
  time: f32,
  effect: Option<PlayingEffect>,
  /// Where the view stood when the modifiers were last weighed, in engine space.
  view: [f32; 3],
  mix: Option<WeatherMix>,
  /// What was last drawn, which a fade starts from.
  shown: Option<RenderLighting>,
  pair: WeatherPair,
  fader: WeatherFader,
  advanced_at: Option<Instant>,
  is_changed: bool,
  /// Whether the next change is forced, as a seek or a cut is, and shown at once.
  is_forced: bool,
  /// Whether a pair the clock never walked to was set since the last frame, as an effect starting or ending sets one.
  is_jumped: bool,
  thunder: WeatherThunder,
  /// Whether the last frame was lit by a strike, which the frame after it ends lights without.
  is_flashing: bool,
  /// What real seconds are counted from, which the bolts strike by.
  epoch: Instant,
}

impl Default for WeatherPlayer {
  fn default() -> Self {
    Self {
      weather: None,
      control: WorldWeatherControl::default(),
      time: NOON,
      effect: None,
      view: [0.0; 3],
      mix: None,
      shown: None,
      pair: WeatherPair::default(),
      fader: WeatherFader::default(),
      advanced_at: None,
      is_changed: false,
      is_forced: false,
      is_jumped: false,
      thunder: WeatherThunder::new(
        std::time::SystemTime::now()
          .duration_since(std::time::UNIX_EPOCH)
          .map_or(1, |since| since.as_nanos() as u64),
      ),
      is_flashing: false,
      epoch: Instant::now(),
    }
  }
}

impl WeatherPlayer {
  /// Whether a weather plays, lighting the scene in place of the default lighting.
  pub fn is_playing(&self) -> bool {
    self.weather.is_some()
  }

  /// The keyframe the pair walks to next, whose skies are fetched before the clock reaches it.
  pub fn get_next(&self) -> Option<&WeatherPlayedKeyframe> {
    let [_, to] = self.pair.get()?;

    WeatherPair::select_next(self.get_keyframes()?, to.time + 0.5)
  }

  /// Plays a weather from now on, or nothing; it takes over from what was shown as the transition says.
  pub fn take(&mut self, weather: Option<PlayedWeather>, transition: WorldWeatherTransition) {
    let duration: Duration = transition.get_duration();

    match &self.shown {
      Some(shown) if weather.is_some() && !duration.is_zero() => self.fader.start(shown.clone(), duration),
      _ => self.fader.stop(),
    }

    self.is_forced = duration.is_zero();
    self.weather = weather;
    self.effect = None;
    self.mix = None;
    self.pair.reset();
    self.thunder.reset();
    self.is_changed = true;

    if self.weather.is_none() {
      self.shown = None;
    }
  }

  pub fn set_control(&mut self, control: WorldWeatherControl) {
    self.control = control;
    self.is_changed = true;
  }

  /// Game seconds the clock runs a real second: its factor, or zero while it is paused.
  pub fn get_clock_rate(&self) -> f32 {
    if self.control.is_paused {
      0.0
    } else {
      self.control.factor
    }
  }

  /// A forced start from a time of day, which ends the effect playing.
  pub fn seek(&mut self, time: f32) {
    self.time = WeatherTime::of_day(time);
    self.effect = None;
    self.pair.reset();
    self.is_forced = true;
    self.is_changed = true;
  }

  /// `SetWeatherFX`, or `StopWFX` for none: an effect already playing gives the cycle back first.
  pub fn play_effect(&mut self, name: Option<&str>) {
    self.stop_effect();

    let (Some(name), Some(weather)) = (name, &self.weather) else {
      return;
    };
    let Some(effect) = weather.get_effect(name) else {
      log::warn!("There is no weather effect '{name}' to play");

      return;
    };

    self.pair.advance(&weather.keyframes, self.time);

    let Some(current) = self.pair.get() else {
      return;
    };
    let Some(timeline) = WeatherEffectTimeline::new(WeatherEffectStart {
      name,
      effect: &effect,
      cycle: &weather.keyframes,
      current,
      time: self.time,
      factor: self.control.factor,
    }) else {
      return;
    };

    self.pair.set(timeline.start.clone());
    self.effect = Some(PlayingEffect {
      remaining: timeline.duration,
      timeline,
    });
    self.is_changed = true;
    self.is_jumped = true;
  }

  /// Moves the clock on to a frame, mixes the weather again where anything changed, and strikes over it while
  /// thundering.
  ///
  /// Returns what the scene is lit by now, or none where that did not change or nothing plays. `is_up` says whether a
  /// lighting's skies are up, which a fade waits for.
  pub fn advance(
    &mut self,
    now: Instant,
    view: [f32; 3],
    is_thundering: bool,
    is_up: impl FnMut(&RenderLighting) -> bool,
  ) -> Option<RenderLighting> {
    let step: Duration = self
      .advanced_at
      .map_or(Duration::ZERO, |at| now.saturating_duration_since(at).min(LONGEST_STEP));

    self.advanced_at = Some(now);

    let weather: &PlayedWeather = self.weather.as_ref()?;
    let control: WorldWeatherControl = self.control;
    let has_modifiers: bool = !weather.level.modifiers.is_empty();

    if !control.is_paused && control.factor > 0.0 && !step.is_zero() {
      self.run(step.as_secs_f32() * control.factor);
    }

    if has_modifiers && distance(view, self.view) > VIEW_STEP {
      self.view = view;
      self.is_changed = true;
    }

    if self.fader.is_fading() {
      self.fader.ask(now);
      self.is_changed = true;
    }

    let lighting: Option<RenderLighting> = if self.is_changed {
      self.relight(now, is_up)
    } else {
      None
    };

    self.flash(now, view, is_thundering, lighting)
  }

  /// What is shown lit by the strike under way, what is shown once more the frame after one ends, or the same.
  fn flash(
    &mut self,
    now: Instant,
    view: [f32; 3],
    is_thundering: bool,
    lighting: Option<RenderLighting>,
  ) -> Option<RenderLighting> {
    let seconds: f32 = now.saturating_duration_since(self.epoch).as_secs_f32();
    let (Some(weather), Some(mix), Some(shown)) = (&self.weather, &self.mix, &self.shown) else {
      return lighting;
    };
    let Some(thunder) = &weather.level.thunder else {
      return lighting;
    };
    let flash = self
      .thunder
      .advance(thunder, mix, is_thundering, seconds, glam::Vec3::from(view));

    if let (Some(flash), Some(settings)) = (flash, thunder.settings) {
      self.is_flashing = true;

      return Some(to_thundered_lighting(
        lighting.as_ref().unwrap_or(shown),
        &flash,
        &settings,
      ));
    }

    if self.is_flashing {
      self.is_flashing = false;

      return lighting.or_else(|| Some(shown.clone()));
    }

    lighting
  }

  /// Where the weather stands, or none while nothing plays.
  pub fn report(&self) -> Option<WorldWeatherReport> {
    let (mix, pair, weather) = (self.mix.as_ref()?, self.pair.get()?, self.weather.as_ref()?);
    let current: WeatherMix = WeatherMixer {
      engine: weather.get_engine(),
      sun: weather.get_sun(&self.control),
      modifiers: &[],
    }
    .mix_pair([pair[0].as_mixed(), pair[1].as_mixed()], self.get_point());
    let heavier: &WeatherPlayedKeyframe = &pair[usize::from(current.weight >= 0.5)];

    Some(WorldWeatherReport {
      time: self.time,
      between: mix.between,
      weight: mix.weight,
      effect: self.effect.as_ref().map(|effect| WorldWeatherEffectReport {
        name: effect.timeline.name.clone(),
        remaining: effect.remaining,
      }),
      modifiers: mix.modifiers,
      current: Box::new(current.to_descriptor(&heavier.descriptor)),
      ambient: None,
    })
  }

  /// What plays now: the effect's keyframes while one plays, else the weather's.
  fn get_keyframes(&self) -> Option<&[WeatherPlayedKeyframe]> {
    match &self.effect {
      Some(effect) => Some(&effect.timeline.keyframes),
      None => self.weather.as_ref().map(|weather| weather.keyframes.as_slice()),
    }
  }

  fn get_point(&self) -> WeatherMixPoint {
    WeatherMixPoint {
      time: self.time,
      view: self.view,
    }
  }

  /// The weather mixed again and faded into from what was shown.
  fn relight(&mut self, now: Instant, mut is_up: impl FnMut(&RenderLighting) -> bool) -> Option<RenderLighting> {
    self.is_changed = false;

    let point: WeatherMixPoint = self.get_point();
    let weather: &PlayedWeather = self.weather.as_ref()?;
    let keyframes: &[WeatherPlayedKeyframe] = match &self.effect {
      Some(effect) => &effect.timeline.keyframes,
      None => &weather.keyframes,
    };

    self.pair.advance(keyframes, self.time);

    let pair: &[WeatherPlayedKeyframe; 2] = self.pair.get()?;
    let mix: WeatherMix = WeatherMixer {
      engine: weather.get_engine(),
      sun: weather.get_sun(&self.control),
      modifiers: &weather.level.modifiers,
    }
    .mix_pair([pair[0].as_mixed(), pair[1].as_mixed()], point);
    let target: RenderLighting = to_weather_lighting(&mix, pair, weather.get_engine());

    // Faded over rather than cut to, as the skies of the engine's own blend never jump.
    if self.is_jumped
      && !self.is_forced
      && !self.fader.is_fading()
      && let Some(shown) = &self.shown
    {
      self
        .fader
        .start(shown.clone(), WorldWeatherTransition::Fade.get_duration());
      self.fader.ask(now);
    }

    self.is_forced = false;
    self.is_jumped = false;

    let lighting: RenderLighting = self.fader.apply(now, &target, is_up(&target));

    self.mix = Some(mix);
    self.shown = Some(lighting.clone());

    Some(lighting)
  }

  /// Runs the clock on, and the effect playing with it, which gives the cycle back once it has run out.
  fn run(&mut self, seconds: f32) {
    self.time = WeatherTime::of_day(self.time + seconds);
    self.is_changed = true;

    if let Some(effect) = &mut self.effect {
      effect.remaining -= seconds;

      if effect.remaining <= 0.0 {
        self.stop_effect();
      }
    }
  }

  /// `StopWFX`: the cycle takes over from the effect's end, blending its keyframe there and the one after.
  fn stop_effect(&mut self) {
    if let Some(effect) = self.effect.take() {
      self.pair.set(effect.timeline.end);
      self.is_changed = true;
      self.is_jumped = true;
    }
  }
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
  (0..3).map(|axis| (a[axis] - b[axis]).powi(2)).sum::<f32>().sqrt()
}
