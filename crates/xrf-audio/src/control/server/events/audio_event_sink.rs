use crate::control::server::events::audio_event::AudioEvent;

/// Where the control thread sends its [`AudioEvent`]s, called on that thread: a host hands them on and returns soon.
pub trait AudioEventSink: Send + 'static {
  fn send(&mut self, event: AudioEvent);
}

impl<F: FnMut(AudioEvent) + Send + 'static> AudioEventSink for F {
  fn send(&mut self, event: AudioEvent) {
    self(event);
  }
}
