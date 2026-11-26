use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use xrf_app_core::TauriResult;
use xrf_audio::{
  AudioAcoustics, AudioEngineSettings, AudioSceneFrame, AudioSceneInbox, AudioServer, AudioSpatialization,
  OfflineSession,
};
use xrf_level_source::{BurstSoundFrame, BurstSoundTee};
use xrf_renderer::RenderBurst;

use crate::viewer::burst::{BurstSoundChunk, BurstSoundOutcome};

/// A burst's sound, rendered on its own thread as the burst draws: the burst's scene frames play in order through an
/// offline session on the burst's clock, acoustics stepped with them, so a burst sounds the same however fast it draws.
/// Sample `n` plays `n / RATE` seconds after the burst's start; what plays from the first frame read back to the last
/// one's end is kept and streamed to the video when one is made.
///
/// The scene fades in over its first quarter second, and a sound starting on the first frame lands up to a tick late;
/// a lead-in covers both. Dropping it before writing disarms the tee and ends its thread.
pub(crate) struct BurstSound {
  tee: BurstSoundTee,
  thread: Option<JoinHandle<TauriResult<Vec<f32>>>>,
  offsets: Vec<u64>,
}

/// The burst sound's state on its own thread.
struct BurstSoundRender {
  timing: BurstSoundTiming,
  acoustics: AudioAcoustics,
  spatialization: AudioSpatialization,
  /// The scene time of the burst's first frame, where the sound starts; the level's sound clock began when the level
  /// opened, not with its picture.
  started: Duration,
  played: Option<PlayedScene>,
  /// Frames the session has rendered.
  rendered: u64,
  /// Interleaved stereo from the first frame read back on.
  kept: Vec<f32>,
  /// Receives each kept stretch for the burst's video; dropped once the video stops.
  video: Option<Sender<BurstSoundChunk>>,
  /// Each render's buffer before its kept part is copied out.
  scratch: Vec<f32>,
}

/// The offline session the burst's scene plays in, and the scene's inbox.
struct PlayedScene {
  session: OfflineSession,
  inbox: AudioSceneInbox,
}

/// The burst's frames on the sound's clock: `step` seconds apart, the first `lead_in` not kept.
#[derive(Clone, Copy)]
struct BurstSoundTiming {
  step: f64,
  lead_in: u32,
  frames: u32,
}

impl BurstSound {
  /// Sample rate of `sound.wav`, stereo 32-bit float.
  pub(crate) const RATE: u32 = 48_000;

  /// Frames rendered per call, a tenth of a second.
  const CHUNK_FRAMES: usize = Self::RATE as usize / 10;

  /// Start taking `burst`'s frames through `tee`, played with `server`'s acoustics and spatialization, streaming kept
  /// stretches to `video` when given.
  ///
  /// # Errors
  ///
  /// Returns an error while another burst records its sound, or when the sound's thread cannot start.
  pub(crate) fn start(
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
    let frames: Receiver<BurstSoundFrame> = tee.arm().map_err(|error| error.to_string())?;
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

  /// Stop taking frames, render to the last frame's end and write `sound.wav` into `folder`, with where each frame
  /// starts in it.
  ///
  /// # Errors
  ///
  /// Returns an error when no sound frame came, or the file cannot be written.
  pub(crate) fn write(mut self, folder: &Path) -> TauriResult<BurstSoundOutcome> {
    self.tee.disarm();

    let samples: Vec<f32> = self
      .thread
      .take()
      .ok_or("The burst's sound was written already")?
      .join()
      .map_err(|_| String::from("The burst's sound stopped on a panic"))??;
    let file = folder.join("sound.wav");

    std::fs::write(&file, Self::encode_wave(&samples))
      .map_err(|error| format!("The burst's sound cannot be written to {}: {error}", file.display()))?;

    Ok(BurstSoundOutcome {
      file,
      rate: Self::RATE,
      channels: 2,
      offsets: std::mem::take(&mut self.offsets),
    })
  }

  /// A WAV file of interleaved stereo 32-bit float `samples` at [`Self::RATE`].
  fn encode_wave(samples: &[f32]) -> Vec<u8> {
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
    // A float format needs a `fact` chunk with its frame count.
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
  /// Play each frame until the tee closes, then render to the last frame's end and answer what was kept.
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

  /// Send `sound`'s frame to the scene. The first opens the session and anchors the clock; each later one is sent
  /// the session's lead ahead of its time, so the sounds it starts land on its frame.
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

  /// The sound frame scene `time` plays at.
  fn get_position(&self, time: Duration) -> u64 {
    (time.saturating_sub(self.started).as_secs_f64() * f64::from(BurstSound::RATE)).round() as u64
  }

  /// Hand `frame` to the scene; dropped when the inbox is full, as the next carries everything again.
  fn send(&mut self, frame: Box<AudioSceneFrame>) {
    if let Some(played) = &mut self.played {
      while played.inbox.take_spent().is_some() {}

      let _ = played.inbox.send(frame);
    }
  }

  /// Render the session up to frame `end`, keeping what plays from the first frame read back on.
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
        // The video stopped and reports why itself.
        self.video = None;
      }

      self.kept.extend_from_slice(kept);
      self.rendered += count as u64;
    }
  }
}

impl BurstSoundTiming {
  /// The sound frame burst frame `index` is drawn at, counting lead-in frames.
  fn get_position(&self, index: u32) -> u64 {
    (f64::from(index) * self.step * f64::from(BurstSound::RATE)).round() as u64
  }
}
