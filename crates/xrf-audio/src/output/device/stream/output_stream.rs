use std::thread;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, ErrorKind, OutputCallbackInfo, OutputStreamTimestamp, Stream, StreamConfig};
use rtrb::{Consumer, RingBuffer};

use crate::mixer::audio_mixer::AudioMixer;
use crate::output::device::stream::mixer_holder::MixerHolder;
use crate::output::device::stream::open_failure::OpenFailure;

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
          mixer: Ok(mixer),
          message: format!("Could not read the device's format: {error}"),
        }));
      }
    };
    let channels: usize = usize::from(config.channels);
    let (mixer_returned, returned) = RingBuffer::new(1);
    let (mut error_sink, errors) = RingBuffer::new(64);

    mixer.set_output_rate(config.sample_rate);

    let mut holder: MixerHolder = MixerHolder::new(mixer, mixer_returned);
    let built = device.build_output_stream(
      config,
      move |out: &mut [f32], info: &OutputCallbackInfo| {
        let timestamp: OutputStreamTimestamp = info.timestamp();

        holder.render(
          out,
          channels,
          timestamp.playback.saturating_duration_since(timestamp.callback),
        );
      },
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
      // A stream that failed to build or start drops its callback, which hands the mixer back, late on some devices.
      Err(error) => Err(Box::new(OpenFailure {
        mixer: Self::await_mixer(returned),
        message: format!("Could not open the device: {error}"),
      })),
    }
  }

  /// Error kinds the stream reported since the last call.
  pub(crate) fn take_errors(&mut self) -> Vec<ErrorKind> {
    std::iter::from_fn(|| self.errors.pop().ok()).collect()
  }

  /// Stops the stream and takes the mixer back from its callback. A device slow to let it go answers with the ring the
  /// callback hands it back through, to look in again later.
  pub(crate) fn close(self) -> Result<AudioMixer, Consumer<AudioMixer>> {
    let Self { stream, returned, .. } = self;

    drop(stream);
    Self::await_mixer(returned)
  }

  /// The mixer the callback hands back through `returned` within [`Self::RETURN_WAIT`], else the ring itself, to look
  /// in again later.
  fn await_mixer(mut returned: Consumer<AudioMixer>) -> Result<AudioMixer, Consumer<AudioMixer>> {
    let deadline: Instant = Instant::now() + Self::RETURN_WAIT;

    loop {
      if let Ok(mixer) = returned.pop() {
        return Ok(mixer);
      }

      if Instant::now() >= deadline {
        return Err(returned);
      }

      thread::sleep(Duration::from_millis(1));
    }
  }
}
