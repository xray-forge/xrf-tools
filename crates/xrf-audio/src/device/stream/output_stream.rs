use std::thread;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, ErrorKind, Stream, StreamConfig};
use rtrb::{Consumer, RingBuffer};

use crate::device::stream::mixer_holder::MixerHolder;
use crate::device::stream::open_failure::OpenFailure;
use crate::mixer::audio_mixer::AudioMixer;

/// A device stream playing the mixer, with what its callbacks report.
pub(crate) struct OutputStream {
  stream: Stream,
  pub(crate) device_id: Option<String>,
  pub(crate) config: StreamConfig,
  returned: Consumer<AudioMixer>,
  errors: Consumer<ErrorKind>,
}

impl OutputStream {
  /// How long closing waits for the callback to hand the mixer back.
  const RETURN_WAIT: Duration = Duration::from_secs(1);

  /// Opens `device` at its default rate and channels and starts it playing `mixer`; a failure carries the mixer back.
  pub(crate) fn open(
    device: &Device,
    device_id: Option<String>,
    mut mixer: AudioMixer,
  ) -> Result<Self, Box<OpenFailure>> {
    let config: StreamConfig = match device.default_output_config() {
      Ok(config) => config.config(),
      Err(error) => {
        return Err(Box::new(OpenFailure {
          mixer: Some(mixer),
          message: format!("Could not read the device's format: {error}"),
        }));
      }
    };
    let channels: usize = usize::from(config.channels);
    let (mixer_returned, mut returned) = RingBuffer::new(1);
    let (mut error_sink, errors) = RingBuffer::new(64);

    mixer.set_output_rate(config.sample_rate);

    let mut holder: MixerHolder = MixerHolder::new(mixer, mixer_returned);
    let built = device.build_output_stream(
      config,
      move |out: &mut [f32], _| holder.render(out, channels),
      move |error: cpal::Error| {
        let _ = error_sink.push(error.kind());
      },
      None,
    );

    match built.and_then(|stream| stream.play().map(|()| stream)) {
      Ok(stream) => Ok(Self {
        stream,
        device_id,
        config,
        returned,
        errors,
      }),
      // A stream that failed to build or start has dropped its callback, which handed the mixer back.
      Err(error) => Err(Box::new(OpenFailure {
        mixer: Self::await_mixer(&mut returned),
        message: format!("Could not open the device: {error}"),
      })),
    }
  }

  /// Error kinds the stream reported since the last call.
  pub(crate) fn take_errors(&mut self) -> Vec<ErrorKind> {
    std::iter::from_fn(|| self.errors.pop().ok()).collect()
  }

  /// Stops the stream and takes the mixer back from its callback, or `None` if the device never let it go.
  pub(crate) fn close(self) -> Option<AudioMixer> {
    let Self {
      stream, mut returned, ..
    } = self;

    drop(stream);
    Self::await_mixer(&mut returned)
  }

  fn await_mixer(returned: &mut Consumer<AudioMixer>) -> Option<AudioMixer> {
    let deadline: Instant = Instant::now() + Self::RETURN_WAIT;

    loop {
      if let Ok(mixer) = returned.pop() {
        return Some(mixer);
      }

      if Instant::now() >= deadline {
        return None;
      }

      thread::sleep(Duration::from_millis(1));
    }
  }
}
