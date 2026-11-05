use std::sync::atomic::{AtomicU32, Ordering};

/// How the listener's reverb decays, written by the control side from the heard scene's simulation and read by the
/// mixer once a block: each band's decay time, in seconds, nought for no reverb.
#[derive(Debug, Default)]
pub(crate) struct ReverbParameters {
  times: [AtomicU32; 3],
}

impl ReverbParameters {
  pub(crate) fn get_times(&self) -> [f32; 3] {
    self
      .times
      .each_ref()
      .map(|time| f32::from_bits(time.load(Ordering::Relaxed)))
  }

  /// Sets each band's decay time; one not a finite number at or above nought is no reverb in that band.
  pub(crate) fn set_times(&self, times: [f32; 3]) {
    for (slot, time) in self.times.iter().zip(times) {
      let time: f32 = if time.is_finite() { time.max(0.0) } else { 0.0 };

      slot.store(time.to_bits(), Ordering::Relaxed);
    }
  }
}
