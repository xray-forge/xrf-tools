use std::time::{Duration, Instant};

use xrf_renderer::RenderLighting;

use crate::weather::weather_fade::to_faded_lighting;

/// Real time a fade waits for the skies it fades into before it starts without them.
const LONGEST_WAIT: Duration = Duration::from_secs(2);

/// A fade from what was shown into what the weather shows now.
struct Fading {
  from: RenderLighting,
  /// When it was first asked for, or none until a frame has.
  asked_at: Option<Instant>,
  /// When it started, or none while the skies it fades into are still going up.
  started_at: Option<Instant>,
  duration: Duration,
}

/// A fade from what was shown into what the weather shows now: it waits for the skies it fades into to go up, two
/// seconds at most, then runs its time.
#[derive(Default)]
pub struct WeatherFader {
  fading: Option<Fading>,
}

impl WeatherFader {
  pub fn is_fading(&self) -> bool {
    self.fading.is_some()
  }

  /// Starts a fade, in place of any before it, which waits from the frame that first asks for it.
  pub fn start(&mut self, from: RenderLighting, duration: Duration) {
    self.fading = Some(Fading {
      from,
      asked_at: None,
      started_at: None,
      duration,
    });
  }

  pub fn stop(&mut self) {
    self.fading = None;
  }

  /// Notes a frame asking for the fade, which it waits for its skies from the first time.
  pub fn ask(&mut self, now: Instant) {
    if let Some(fading) = &mut self.fading {
      fading.asked_at.get_or_insert(now);
    }
  }

  /// What to draw: the fade under way, or the weather where none is.
  pub fn apply(&mut self, now: Instant, target: &RenderLighting, is_up: bool) -> RenderLighting {
    let Some(fading) = &mut self.fading else {
      return target.clone();
    };

    if fading.started_at.is_none() && (is_up || now.duration_since(fading.asked_at.unwrap_or(now)) >= LONGEST_WAIT) {
      fading.started_at = Some(now);
    }

    let progress: f32 = fading.started_at.map_or(0.0, |started| {
      now.duration_since(started).as_secs_f32() / fading.duration.as_secs_f32().max(f32::EPSILON)
    });
    let lighting: RenderLighting = to_faded_lighting(&fading.from, target, progress);

    if progress >= 1.0 {
      self.fading = None;
    }

    lighting
  }
}
