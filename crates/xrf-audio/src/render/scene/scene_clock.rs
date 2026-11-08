use std::time::Duration;

/// A scene's clock on the mixer's: the scene's time a mixer frame plays, by an offset taken from the frames the scene
/// sends and moved only once they drift past [`Self::DRIFT`], so a clock stepped, paced in bursts or late by a frame
/// never makes a playing voice jump.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SceneClock {
  /// The mixer frame the scene's clock stood at nought on, in frames of the mixer's rate.
  offset: Option<i64>,
}

impl SceneClock {
  /// How far the scene's clock may drift from the mixer's before it is anchored again.
  pub(crate) const DRIFT: Duration = Duration::from_millis(100);

  /// Takes a frame stamped `time` on the scene's clock arriving at mixer frame `frame` of `rate`, answering whether the
  /// clock was anchored again.
  pub(crate) fn follow(&mut self, time: Duration, frame: u64, rate: u32) -> bool {
    let anchored: i64 = frame.cast_signed() - Self::get_frames(time, rate);
    let drift: i64 = Self::get_frames(Self::DRIFT, rate);

    match self.offset {
      Some(offset) if (anchored - offset).abs() <= drift => false,
      _ => {
        self.offset = Some(anchored);
        true
      }
    }
  }

  /// The scene's time mixer frame `frame` of `rate` plays, none before the first frame anchored it.
  pub(crate) fn get_time(&self, frame: u64, rate: u32) -> Option<Duration> {
    let offset: i64 = self.offset?;
    let frames: i64 = (frame.cast_signed() - offset).max(0);

    Some(Duration::from_secs_f64(frames as f64 / f64::from(rate.max(1))))
  }

  /// The mixer frame of `rate` the scene's `time` plays at, none before the first frame anchored it.
  pub(crate) fn get_frame(&self, time: Duration, rate: u32) -> Option<u64> {
    let offset: i64 = self.offset?;

    Some((offset + Self::get_frames(time, rate)).max(0).cast_unsigned())
  }

  fn get_frames(time: Duration, rate: u32) -> i64 {
    (time.as_secs_f64() * f64::from(rate)).round() as i64
  }
}
