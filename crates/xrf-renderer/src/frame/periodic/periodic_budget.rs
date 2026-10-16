use std::time::Duration;

use crate::contract::render_frame_rate::RenderFrameRate;

/// What a frame gives the work that may wait, from the headroom its frames leave: where a frame cap or the display's
/// refresh holds the frames, the interval they are held to less the GPU's smoothed frame time and a margin, between the
/// settings' floor and `CEILING`; the floor alone where nothing holds them, as any more would slow every frame, and
/// while the GPU's frame time is unknown. The GPU time and the refresh are smoothed over about half a second, and the
/// budget moves only past `HYSTERESIS`, so it neither follows one slow frame nor wavers.
#[derive(Debug, Default)]
pub struct PeriodicBudget {
  /// Smoothed milliseconds the GPU spends a frame.
  gpu: Option<f32>,
  /// Milliseconds between frames at the fastest the display presents them, as the frames found it.
  refresh: Option<f32>,
  /// The budget granted last.
  budget: Option<f32>,
}

impl PeriodicBudget {
  /// Milliseconds a frame gives the work that may wait at most, so none takes a burst of it at once.
  pub const CEILING: f32 = 4.0;
  /// Seconds the GPU time is smoothed over.
  const SMOOTHING: f32 = 0.5;
  /// The headroom kept clear: this share of the interval, at least `MARGIN_LEAST` milliseconds.
  const MARGIN: f32 = 0.1;
  const MARGIN_LEAST: f32 = 1.0;
  /// Milliseconds the budget must move by before it is changed.
  const HYSTERESIS: f32 = 0.25;
  /// How far a slower interval moves the refresh up a frame: it follows the fastest, and only slowly what is slower.
  const REFRESH_RISE: f32 = 0.01;

  /// Takes a frame `delta` after the last, `span` the milliseconds of the latest GPU frame read back where one came
  /// back, held to `frame_rate`: answers what the frame gives the work that may wait, at least `floor`.
  pub fn update(&mut self, (delta, span): (Duration, Option<f32>), frame_rate: RenderFrameRate, floor: f32) -> f32 {
    let seconds: f32 = delta.as_secs_f32();
    let interval: f32 = seconds * 1000.0;

    if let Some(span) = span {
      let share: f32 = 1.0 - (-seconds / Self::SMOOTHING).exp();

      self.gpu = Some(self.gpu.map_or(span, |gpu| gpu + (span - gpu) * share));
    }

    if interval > 0.0 {
      self.refresh = Some(self.refresh.map_or(interval, |refresh| {
        if interval < refresh {
          interval
        } else {
          refresh + (interval - refresh) * Self::REFRESH_RISE
        }
      }));
    }

    let held: Option<f32> = match frame_rate.get_interval() {
      Some(cap) => Some(cap.as_secs_f32() * 1000.0),
      None if frame_rate.is_vsync => self.refresh,
      None => None,
    };
    let wanted: f32 = match (held, self.gpu) {
      (Some(target), Some(gpu)) => {
        (target - gpu - (target * Self::MARGIN).max(Self::MARGIN_LEAST)).clamp(floor, Self::CEILING.max(floor))
      }
      _ => floor,
    };
    let budget: f32 = match self.budget {
      Some(budget) if (wanted - budget).abs() < Self::HYSTERESIS && budget >= floor => budget,
      _ => wanted,
    };

    self.budget = Some(budget);

    budget
  }
}
