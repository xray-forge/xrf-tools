use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use xrf_app_core::TauriResult;
use xrf_audio::{
  AudioAcoustics, AudioEngineSettings, AudioSceneFrame, AudioSceneInbox, AudioServer, AudioSpatialization,
  OfflineSession,
};
use xrf_renderer::RenderBurst;

use crate::viewer::burst::{BurstSoundChunk, BurstSoundFrame, BurstSoundOutcome, BurstSoundTee};

/// A burst's sound, rendered as the burst draws on a thread of its own: the scene's frames the burst drew, played in
/// order through an offline session of their own on the burst's clock, the scene's walls and rooms simulated in step,
/// so a burst from one time plays alike however fast it draws. Its sample `n` plays at the burst's start plus
/// `n / RATE` seconds; what plays from the first frame read back to the last one's end is kept, and handed to the
/// burst's video as it is where one is made.
///
/// The scene fades in over its first quarter of a second, and a sound starting with the burst's first frame lands up to
/// a tick late, which a lead-in covers. Dropped before it is written, it stops taking frames and its thread ends.
pub struct BurstSound {
  tee: BurstSoundTee,
  thread: Option<JoinHandle<TauriResult<Vec<f32>>>>,
  offsets: Vec<u64>,
}

/// Where the burst's sound stands on its own thread.
struct BurstSoundRender {
  timing: BurstSoundTiming,
  acoustics: AudioAcoustics,
  spatialization: AudioSpatialization,
  /// The scene's time of the burst's first frame, which the sound starts at: the level's sound runs on a clock of its
  /// own, begun as the level opened rather than as its picture did.
  started: Duration,
  played: Option<PlayedScene>,
  /// Frames the session has rendered.
  rendered: u64,
  /// What plays from the first frame read back on, interleaved stereo.
  kept: Vec<f32>,
  /// Where each stretch kept goes as it is, for the burst's video; gone with the video.
  video: Option<Sender<BurstSoundChunk>>,
  /// Where each render lands before what is kept of it is.
  scratch: Vec<f32>,
}

/// The session the burst's scene plays in, and the scene's inbox.
struct PlayedScene {
  session: OfflineSession,
  inbox: AudioSceneInbox,
}

/// The burst's frames on the sound's clock: a step apart from the first, and which of them are kept.
#[derive(Clone, Copy)]
struct BurstSoundTiming {
  step: f64,
  lead_in: u32,
  frames: u32,
}

impl BurstSound {
  /// Frames a second of `sound.wav`, which takes its two channels as 32-bit floats.
  pub const RATE: u32 = 48_000;

  /// Frames rendered at once, a tenth of a second.
  const CHUNK_FRAMES: usize = Self::RATE as usize / 10;

  /// Starts taking the frames `burst` draws through `tee`, played through `server`'s acoustics for its ears, each
  /// stretch kept sent on to `video` where there is one.
  ///
  /// # Errors
  ///
  /// Returns an error while another burst records its sound, or when the sound's thread cannot start.
  pub fn start(
    burst: &RenderBurst,
    server: &AudioServer,
    tee: &BurstSoundTee,
    video: Option<Sender<BurstSoundChunk>>,
  ) -> TauriResult<Self> {
    let timing: BurstSoundTiming = BurstSoundTiming {
      step: f64::from(burst.step),
      lead_in: burst.lead_in,
      frames: burst.frames,
    };
    let render: BurstSoundRender = BurstSoundRender {
      timing,
      acoustics: server.get_acoustics().clone(),
      spatialization: server.get_spatialization(),
      started: Duration::ZERO,
      played: None,
      rendered: 0,
      kept: Vec::new(),
      video,
      scratch: vec![0.0; Self::CHUNK_FRAMES * 2],
    };
    let frames: Receiver<BurstSoundFrame> = tee.arm()?;
    let thread: JoinHandle<TauriResult<Vec<f32>>> = thread::Builder::new()
      .name(String::from("xrf-burst-sound"))
      .spawn(move || render.play(&frames))
      .map_err(|error| {
        tee.disarm();
        format!("The burst's sound cannot start: {error}")
      })?;

    Ok(Self {
      tee: tee.clone(),
      thread: Some(thread),
      offsets: (0..burst.frames)
        .map(|index| timing.get_position(burst.lead_in + index) - timing.get_position(burst.lead_in))
        .collect(),
    })
  }

  /// Stops taking frames, renders the sound to the last frame's end and writes it into `folder` as `sound.wav`,
  /// answering where each frame read back starts in it.
  ///
  /// # Errors
  ///
  /// Returns an error when no frame of the burst's sound came, or the file cannot be written.
  pub fn write(mut self, folder: &Path) -> TauriResult<BurstSoundOutcome> {
    self.tee.disarm();

    let samples: Vec<f32> = self
      .thread
      .take()
      .ok_or("The burst's sound was written already")?
      .join()
      .map_err(|_| String::from("The burst's sound stopped on a panic"))??;
    let file = folder.join("sound.wav");

    std::fs::write(&file, Self::to_wave(&samples))
      .map_err(|error| format!("The burst's sound cannot be written to {}: {error}", file.display()))?;

    Ok(BurstSoundOutcome {
      file,
      rate: Self::RATE,
      channels: 2,
      offsets: std::mem::take(&mut self.offsets),
    })
  }

  /// A WAV file of interleaved stereo 32-bit float `samples` at [`Self::RATE`].
  fn to_wave(samples: &[f32]) -> Vec<u8> {
    const CHANNELS: u16 = 2;
    const FLOAT: u16 = 3;
    let data_length: u32 = (samples.len() * 4) as u32;
    let block_align: u16 = CHANNELS * 4;
    let mut bytes: Vec<u8> = Vec::with_capacity(58 + samples.len() * 4);

    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(50 + data_length).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&18u32.to_le_bytes());
    bytes.extend_from_slice(&FLOAT.to_le_bytes());
    bytes.extend_from_slice(&CHANNELS.to_le_bytes());
    bytes.extend_from_slice(&Self::RATE.to_le_bytes());
    bytes.extend_from_slice(&(Self::RATE * u32::from(block_align)).to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&32u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    // A float format names how many frames it holds.
    bytes.extend_from_slice(b"fact");
    bytes.extend_from_slice(&4u32.to_le_bytes());
    bytes.extend_from_slice(&((samples.len() / 2) as u32).to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_length.to_le_bytes());

    for sample in samples {
      bytes.extend_from_slice(&sample.to_le_bytes());
    }

    bytes
  }
}

impl Drop for BurstSound {
  fn drop(&mut self) {
    self.tee.disarm();
  }
}

impl BurstSoundRender {
  /// Plays each frame as it comes, until the tee stops sending, then renders on to the last frame's end and answers
  /// what was kept.
  fn play(mut self, frames: &Receiver<BurstSoundFrame>) -> TauriResult<Vec<f32>> {
    while let Ok(frame) = frames.recv() {
      self.take(frame);
    }

    if self.played.is_none() {
      return Err(String::from(
        "No sound frame of the burst came: its level plays no sound",
      ));
    }

    self.render_to(self.timing.get_position(self.timing.lead_in + self.timing.frames));

    Ok(self.kept)
  }

  /// Sends `sound`'s frame to the scene, the session opened on the first, the burst's first, which the sound starts
  /// at and the scene's clock is anchored by; each after it goes as far ahead of its time as the session asks, so every
  /// sound it starts lands on its frame.
  fn take(&mut self, sound: BurstSoundFrame) {
    let BurstSoundFrame {
      frame,
      assets,
      geometry,
    } = sound;
    let sent_at: u64 = match &self.played {
      Some(played) => self
        .get_position(frame.time)
        .saturating_sub(played.session.get_lead_frames()),
      None => {
        let mut session: OfflineSession = OfflineSession::with_stepped_acoustics(
          AudioEngineSettings::default(),
          BurstSound::RATE,
          self.acoustics.clone(),
        );

        session.set_spatialization(self.spatialization);

        let inbox: AudioSceneInbox = session.open_scene(assets, Some(geometry));

        self.played = Some(PlayedScene { session, inbox });
        self.started = frame.time;

        0
      }
    };

    self.render_to(sent_at);
    self.send(frame);
  }

  /// The sound's frame the scene's `time` plays at.
  fn get_position(&self, time: Duration) -> u64 {
    (time.saturating_sub(self.started).as_secs_f64() * f64::from(BurstSound::RATE)).round() as u64
  }

  /// Hands `frame` to the scene; one its inbox has no room for is dropped, the next carrying everything again.
  fn send(&mut self, frame: Box<AudioSceneFrame>) {
    if let Some(played) = &mut self.played {
      while played.inbox.take_spent().is_some() {}

      let _ = played.inbox.send(frame);
    }
  }

  /// Renders the session on to its frame `end`, keeping what plays from the first frame read back on.
  fn render_to(&mut self, end: u64) {
    let Some(played) = &mut self.played else {
      return;
    };
    let kept_from: u64 = self.timing.get_position(self.timing.lead_in);

    while self.rendered < end {
      let count: usize = ((end - self.rendered) as usize).min(BurstSound::CHUNK_FRAMES);
      let rendered: &mut [f32] = &mut self.scratch[..count * 2];

      played.session.render_into(rendered);

      let skipped: usize = (kept_from.saturating_sub(self.rendered) as usize).min(count);
      let kept: &[f32] = &rendered[skipped * 2..];

      if let Some(video) = &self.video
        && !kept.is_empty()
        && video
          .send(BurstSoundChunk {
            at: (self.kept.len() / 2) as u64,
            samples: kept.to_vec(),
          })
          .is_err()
      {
        // The video stopped, and says why itself.
        self.video = None;
      }

      self.kept.extend_from_slice(kept);
      self.rendered += count as u64;
    }
  }
}

impl BurstSoundTiming {
  /// The sound's frame the burst's frame `index` is drawn at, the lead-in's counted.
  fn get_position(&self, index: u32) -> u64 {
    (f64::from(index) * self.step * f64::from(BurstSound::RATE)).round() as u64
  }
}
