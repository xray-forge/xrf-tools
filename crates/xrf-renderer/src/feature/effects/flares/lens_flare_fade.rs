use std::time::Instant;

use xrf_math::EPS_S;

/// Where the fade between two lens flares stands, `CLensFlare::LFState`.
#[derive(Clone, Copy, Debug, PartialEq)]
enum FadeStep {
  /// Coming in to full.
  Show,
  /// At full.
  Idle,
  /// Going out, for the next one to come in.
  Hide,
}

/// The lens flare a viewport draws, faded out and the next one in when the keyframes name another, over its own
/// `blend_down_time` and the next one's `blend_rise_time` in game seconds, as `CLensFlare::OnFrame` steps it.
#[derive(Debug)]
pub struct LensFlareFade {
  current: Option<String>,
  step: Option<FadeStep>,
  /// `m_StateBlend`.
  blend: f32,
  last: Option<Instant>,
}

impl LensFlareFade {
  pub fn new() -> Self {
    Self {
      current: None,
      step: None,
      blend: 0.0,
      last: None,
    }
  }

  /// Steps the fade towards the lens flare named now, by the game seconds since the last step: real ones times `rate`;
  /// at a rate of zero, a paused clock, every fade completes at once. `times` gives a lens flare's rise and down times.
  pub fn advance(&mut self, named: Option<&str>, times: impl Fn(&str) -> Option<(f32, f32)>, now: Instant, rate: f32) {
    let delta: f32 = self
      .last
      .map_or(0.0, |last| now.saturating_duration_since(last).as_secs_f32());
    let game: f32 = if rate > 0.0 { delta * rate } else { f32::INFINITY };
    let speed = |seconds: f32| 1.0 / (seconds.max(0.0) + EPS_S);
    let rise = |name: Option<&String>| {
      name
        .and_then(|it| times(it))
        .map_or(1.0 / EPS_S, |(rise, _)| speed(rise))
    };
    let down = |name: Option<&String>| {
      name
        .and_then(|it| times(it))
        .map_or(1.0 / EPS_S, |(_, down)| speed(down))
    };

    self.last = Some(now);

    match self.step {
      None => {
        self.step = Some(FadeStep::Show);
        self.current = named.map(str::to_owned);
      }
      Some(FadeStep::Idle) => {
        if named != self.current.as_deref() {
          self.step = Some(FadeStep::Hide);
        }
      }
      Some(FadeStep::Show) => {
        self.blend += rise(self.current.as_ref()) * game;

        if self.blend >= 1.0 {
          self.step = Some(FadeStep::Idle);
        }
      }
      Some(FadeStep::Hide) => {
        self.blend -= down(self.current.as_ref()) * game;

        if self.blend <= 0.0 {
          self.step = Some(FadeStep::Show);
          self.current = named.map(str::to_owned);
          self.blend = 0.0;
        }
      }
    }

    self.blend = self.blend.clamp(0.0, 1.0);
  }

  /// Settles at its next step on the lens flare named then, shown whole, as if it had stood since long before, and
  /// steps from that frame on: what it shows from then on follows from the frames since alone, without a fade from
  /// whatever stood before.
  pub fn settle(&mut self) {
    self.current = None;
    self.step = None;
    self.blend = 1.0;
    self.last = None;
  }

  /// The lens flare drawn now, if any.
  pub fn get_shown(&self) -> Option<&str> {
    self.current.as_deref()
  }

  /// How far the lens flare drawn now has faded in.
  pub fn get_blend(&self) -> f32 {
    self.blend
  }
}
