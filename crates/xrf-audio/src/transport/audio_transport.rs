use std::sync::Arc;
use std::time::Duration;

use xrf_error::{XrfError, XrfResult};

use crate::control::audio_engine::AudioEngine;
use crate::control::audio_play::AudioPlay;
use crate::decode::sound_decoder::SoundDecoder;
use crate::decode::stream::sound_stream::SoundStream;
use crate::decode::stream::sound_stream_start::SoundStreamStart;
use crate::decode::stream::stream_control::StreamControl;
use crate::mixer::audio_bus::AudioBus;
use crate::transport::audio_transport_playback::AudioTransportPlayback;
use crate::transport::audio_transport_state::AudioTransportState;
use crate::voice::audio_voice_id::AudioVoiceId;
use crate::voice::audio_voice_state::AudioVoiceState;
use crate::voice::voice_source::VoiceSource;

/// One sound a player plays, pauses, seeks and stops, as a media element would: each play streams the sound from where
/// the transport stands, and a pause or a seek ends that voice and remembers the frame.
pub struct AudioTransport {
  bytes: Arc<[u8]>,
  bus: AudioBus,
  sample_rate: u32,
  frames_count: u64,
  is_looped: bool,
  place: TransportPlace,
  failure: Option<String>,
}

/// What the transport has on the audio thread.
enum TransportPlace {
  /// No voice: the frame the next play starts from, and how the transport came to hold there.
  Held {
    frame: u64,
    playback: AudioTransportPlayback,
  },
  Playing {
    voice: AudioVoiceId,
    control: StreamControl,
  },
}

impl AudioTransport {
  /// A transport over the sound in `bytes`, stopped at its start, playing on `bus`.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a sound [`SoundDecoder`] opens.
  pub fn open(bytes: Arc<[u8]>, bus: AudioBus) -> XrfResult<Self> {
    let decoder: SoundDecoder = SoundDecoder::open(bytes.clone())?;

    Ok(Self {
      bytes,
      bus,
      sample_rate: decoder.get_sample_rate(),
      frames_count: decoder.get_frames_count(),
      is_looped: false,
      place: TransportPlace::Held {
        frame: 0,
        playback: AudioTransportPlayback::Stopped,
      },
      failure: None,
    })
  }

  /// The sound's bytes, for whoever reads more of it (its waveform).
  pub fn get_bytes(&self) -> &Arc<[u8]> {
    &self.bytes
  }

  /// Plays from where the transport holds, or from the start once it has ended. A playing transport is left alone.
  ///
  /// # Errors
  ///
  /// Returns an error when the sound cannot be streamed or the engine refuses the voice; the transport holds where it
  /// was and names the failure.
  pub fn play(&mut self, engine: &mut AudioEngine) -> XrfResult<()> {
    self.follow(engine);

    match self.place {
      TransportPlace::Playing { .. } => Ok(()),
      TransportPlace::Held {
        playback: AudioTransportPlayback::Ended,
        ..
      } => self.start(engine, 0),
      TransportPlace::Held { frame, .. } => self.start(engine, frame),
    }
  }

  /// Holds where the transport plays now.
  ///
  /// # Errors
  ///
  /// Returns an error when the engine refuses to stop the voice.
  pub fn pause(&mut self, engine: &mut AudioEngine) -> XrfResult<()> {
    self.follow(engine);

    if let TransportPlace::Playing { voice, .. } = self.place {
      let frame: u64 = engine.get_playback(voice).frame;

      self.end_voice(engine, voice)?;
      self.place = TransportPlace::Held {
        frame,
        playback: AudioTransportPlayback::Paused,
      };
    }

    Ok(())
  }

  /// Puts the transport back at the start, silent.
  ///
  /// # Errors
  ///
  /// Returns an error when the engine refuses to stop the voice.
  pub fn stop(&mut self, engine: &mut AudioEngine) -> XrfResult<()> {
    if let TransportPlace::Playing { voice, .. } = self.place {
      self.end_voice(engine, voice)?;
    }

    self.place = TransportPlace::Held {
      frame: 0,
      playback: AudioTransportPlayback::Stopped,
    };

    Ok(())
  }

  /// Moves to `position`, clamped to the sound: a playing transport plays on from there, any other holds there paused.
  ///
  /// # Errors
  ///
  /// Returns an error when the engine refuses to stop the voice, or the sound cannot be streamed from there.
  pub fn seek(&mut self, engine: &mut AudioEngine, position: Duration) -> XrfResult<()> {
    let frame: u64 = ((position.as_secs_f64() * f64::from(self.sample_rate)) as u64).min(self.frames_count);

    self.follow(engine);

    match self.place {
      TransportPlace::Playing { voice, .. } => {
        self.end_voice(engine, voice)?;
        self.start(engine, frame)
      }
      TransportPlace::Held { .. } => {
        self.place = TransportPlace::Held {
          frame,
          playback: AudioTransportPlayback::Paused,
        };

        Ok(())
      }
    }
  }

  /// Whether the sound starts over at its end, from the playing stream's next pass on.
  pub fn set_looped(&mut self, is_looped: bool) {
    self.is_looped = is_looped;

    if let TransportPlace::Playing { control, .. } = &self.place {
      control.set_looped(is_looped);
    }
  }

  /// Where the transport is, having taken in a voice that ended since it last looked.
  pub fn describe(&mut self, engine: &AudioEngine) -> AudioTransportState {
    self.follow(engine);

    let playback: AudioTransportPlayback = match &self.place {
      TransportPlace::Playing { .. } => AudioTransportPlayback::Playing,
      TransportPlace::Held { playback, .. } => *playback,
    };
    let frame: u64 = match &self.place {
      TransportPlace::Playing { voice, .. } => engine.get_playback(*voice).frame,
      TransportPlace::Held { frame, .. } => *frame,
    };

    AudioTransportState {
      playback,
      position: self.to_duration(frame),
      duration: self.to_duration(self.frames_count),
      is_looped: self.is_looped,
      failure: self.failure.clone(),
    }
  }

  fn start(&mut self, engine: &mut AudioEngine, frame: u64) -> XrfResult<()> {
    let start: SoundStreamStart = SoundStreamStart {
      frame,
      is_looped: self.is_looped,
    };
    let reader = SoundStream::start(self.bytes.clone(), start).inspect_err(|error| {
      self.failure = Some(error.to_string());
    })?;
    let control: StreamControl = reader.get_control();

    match engine.play(AudioPlay::new(VoiceSource::Streamed(reader), self.bus)) {
      Ok(voice) => {
        self.place = TransportPlace::Playing { voice, control };
        self.failure = None;

        Ok(())
      }
      Err(refusal) => {
        self.failure = Some(format!("Could not play: {refusal}"));

        Err(XrfError::new_unexpected_error(format!(
          "Could not play the sound: {refusal}"
        )))
      }
    }
  }

  fn end_voice(&mut self, engine: &mut AudioEngine, voice: AudioVoiceId) -> XrfResult<()> {
    engine
      .stop(voice)
      .map_err(|refusal| XrfError::new_unexpected_error(format!("Could not stop the sound: {refusal}")))
  }

  /// Takes in a voice that played to its end, or whose stream failed.
  fn follow(&mut self, engine: &AudioEngine) {
    let TransportPlace::Playing { voice, control } = &self.place else {
      return;
    };

    if engine.get_playback(*voice).state != AudioVoiceState::Ended {
      return;
    }

    self.failure = control.get_failure();
    self.place = TransportPlace::Held {
      frame: self.frames_count,
      playback: AudioTransportPlayback::Ended,
    };
  }

  fn to_duration(&self, frame: u64) -> Duration {
    Duration::from_secs_f64(frame as f64 / f64::from(self.sample_rate))
  }
}
