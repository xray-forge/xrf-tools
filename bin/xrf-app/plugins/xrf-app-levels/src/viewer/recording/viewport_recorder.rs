use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use tauri::ipc::Channel;
use xrf_app_core::TauriResult;
use xrf_app_render::Renderer;
use xrf_audio::{AudioClockAnchor, AudioMixerMeter, AudioServer, AudioTap, AudioTapBlock};
use xrf_renderer::{RecordEvent, RecordFrame, RenderViewportId};

use crate::viewer::record::{RecordFormat, RecordSample, RecordSoundFormat, RecordSummary, RecordWriter};
use crate::viewer::recording::{RecordingOutcome, RecordingProgress};

/// A viewport recorded with what its camera hears into one video, on a thread of its own: the renderer's frames as they
/// were presented and the mix before the master as the device plays it, put on one line by the audio's clock. Time zero
/// is the first frame's present; the sound from the mixer's frame its anchor places there, each block at its own frame
/// on from it, so a block the tap had no room for is a gap, not a shift. It ends when stopped, or by itself where the
/// renderer stops recording or the audio goes on at another rate, the file finished to there either way.
pub struct ViewportRecorder {
  viewport: RenderViewportId,
  file: PathBuf,
  is_stopped: Arc<AtomicBool>,
  thread: Option<JoinHandle<TauriResult<RecordingOutcome>>>,
}

/// What the recorder's thread works with.
struct RecorderInputs {
  frames: Receiver<RecordEvent>,
  tap: AudioTap,
  meter: AudioMixerMeter,
  writer: RecordWriter,
  size: (u32, u32),
  file: PathBuf,
  progress: Channel<RecordingProgress>,
  is_stopped: Arc<AtomicBool>,
}

/// Where a recording stands on its thread.
#[derive(Default)]
struct RecorderState {
  /// The first frame's present, and the mixer's frame playing then.
  zero: Option<(Instant, u64)>,
  frames: u32,
  end: Duration,
  dropped_frames: u64,
  ended: Option<String>,
}

impl ViewportRecorder {
  /// The sound the tap holds before a block drops: the recorder takes it every frame.
  const TAP_LENGTH: Duration = Duration::from_secs(2);

  /// How long a frame shows where the next has not come: the renderer keeps one a sixtieth apart at most.
  const FRAME: Duration = Duration::from_nanos(16_666_667);

  /// How often the page is told where the recording stands.
  const PROGRESS_EVERY: Duration = Duration::from_millis(250);

  /// Starts recording `viewport` at `size`, even both ways as H.264 takes, into `file`, with `server`'s mix; tells
  /// `progress` where it stands.
  ///
  /// # Errors
  ///
  /// Returns an error when the mix cannot be tapped, the video cannot be written, or the thread cannot start.
  pub fn start(
    renderer: &Renderer,
    server: &AudioServer,
    viewport: RenderViewportId,
    size: (u32, u32),
    file: PathBuf,
    progress: Channel<RecordingProgress>,
  ) -> TauriResult<Self> {
    let size: (u32, u32) = (size.0 & !1, size.1 & !1);
    let tap: AudioTap = server.open_tap(Self::TAP_LENGTH).map_err(|error| error.to_string())?;
    let writer: RecordWriter = RecordWriter::start(
      file.clone(),
      RecordFormat {
        width: size.0,
        height: size.1,
        frame: Self::FRAME,
        sound: Some(RecordSoundFormat {
          rate: tap.get_rate(),
          channels: 2,
        }),
      },
    )?;
    let is_stopped: Arc<AtomicBool> = Arc::default();
    let inputs: RecorderInputs = RecorderInputs {
      frames: renderer.record(viewport, size),
      tap,
      meter: server.get_meter().clone(),
      writer,
      size,
      file: file.clone(),
      progress,
      is_stopped: Arc::clone(&is_stopped),
    };
    let thread: JoinHandle<TauriResult<RecordingOutcome>> = thread::Builder::new()
      .name(String::from("xrf-recorder"))
      .spawn(move || inputs.run())
      .map_err(|error| format!("The recording cannot start: {error}"))?;

    Ok(Self {
      viewport,
      file,
      is_stopped,
      thread: Some(thread),
    })
  }

  /// The viewport recorded.
  pub fn get_viewport(&self) -> RenderViewportId {
    self.viewport
  }

  /// Where it is written.
  pub fn get_file(&self) -> &PathBuf {
    &self.file
  }

  /// Whether it ended by itself, and is only to be collected.
  pub fn is_ended(&self) -> bool {
    self.thread.as_ref().is_none_or(JoinHandle::is_finished)
  }

  /// Stops it and finishes the file, answering what was written.
  ///
  /// # Errors
  ///
  /// Returns an error when the file could not be finished, or its thread stopped on a panic.
  pub fn stop(mut self) -> TauriResult<RecordingOutcome> {
    self.is_stopped.store(true, Ordering::Release);

    self
      .thread
      .take()
      .ok_or("The recording was stopped already")?
      .join()
      .map_err(|_| String::from("The recording stopped on a panic"))?
  }
}

impl Drop for ViewportRecorder {
  fn drop(&mut self) {
    self.is_stopped.store(true, Ordering::Release);
  }
}

impl RecorderInputs {
  /// Takes frames and sound until stopped or ended, telling the page where it stands, then finishes the file.
  fn run(mut self) -> TauriResult<RecordingOutcome> {
    let mut state: RecorderState = RecorderState::default();
    let mut told: Instant = Instant::now();

    while !self.is_stopped.load(Ordering::Acquire) && state.ended.is_none() {
      match self.frames.recv_timeout(Duration::from_millis(5)) {
        Ok(RecordEvent::Frame(frame)) => self.take_frame(frame, &mut state)?,
        Ok(RecordEvent::Stopped { error }) => state.ended = Some(error.to_string()),
        Err(RecvTimeoutError::Timeout) => {}
        Err(RecvTimeoutError::Disconnected) => {
          state.ended = Some(String::from("The renderer stopped recording"));
        }
      }

      self.take_sound(&mut state)?;

      if told.elapsed() >= ViewportRecorder::PROGRESS_EVERY || state.ended.is_some() {
        told = Instant::now();
        let _ = self.progress.send(self.describe(&state));
      }
    }

    let finished: TauriResult<RecordSummary> = self.writer.finish();

    // A recording refused before its first frame, as while another long job runs, leaves no empty file behind.
    if state.frames == 0 {
      let _ = std::fs::remove_file(&self.file);

      return Err(state.ended.unwrap_or_else(|| String::from("No frame was recorded")));
    }

    let summary: RecordSummary = finished?;

    Ok(RecordingOutcome {
      file: self.file,
      duration: summary.duration.as_secs_f64(),
      frames: summary.frames,
      dropped_frames: state.dropped_frames,
      dropped_sound: self.tap.get_dropped(),
      width: self.size.0,
      height: self.size.1,
      encoder: summary.encoder,
    })
  }

  /// Writes `frame` at its present, the first setting the recording's zero and the mixer's frame playing then.
  fn take_frame(&mut self, frame: RecordFrame, state: &mut RecorderState) -> TauriResult {
    let presented: Instant = frame.presented;
    let rate: u32 = self.tap.get_rate();
    let (zero, _) = *state.zero.get_or_insert_with(|| {
      let mixer: u64 = self
        .meter
        .get_anchor()
        .map_or(self.meter.get_frames(), |anchor: AudioClockAnchor| {
          anchor.get_frame(presented, rate)
        });

      (presented, mixer)
    });
    let time: Duration = presented.saturating_duration_since(zero);

    state.dropped_frames = frame.dropped;

    let bgra: Vec<u8> = frame.read().map_err(|error| error.to_string())?;

    self.writer.write(RecordSample::Frame {
      bgra,
      time,
      duration: ViewportRecorder::FRAME,
    })?;
    state.frames += 1;
    state.end = time + ViewportRecorder::FRAME;

    Ok(())
  }

  /// Writes the sound the tap holds from the recording's zero on, each block at its frame from the zero's; before the
  /// first frame it is let go.
  fn take_sound(&mut self, state: &mut RecorderState) -> TauriResult {
    loop {
      let block: AudioTapBlock = match self.tap.read() {
        Ok(Some(block)) => block,
        Ok(None) => return Ok(()),
        Err(end) => {
          state.ended = Some(format!("The recording ended: {end}"));

          return Ok(());
        }
      };
      let Some((_, start)) = state.zero else {
        continue;
      };
      let end: u64 = block.frame + AudioTapBlock::FRAMES as u64;

      if end <= start {
        continue;
      }

      let skipped: usize = start.saturating_sub(block.frame) as usize;

      self.writer.write(RecordSample::Sound {
        samples: block.samples[skipped * 2..].to_vec(),
        at: block.frame.max(start) - start,
      })?;
    }
  }

  fn describe(&self, state: &RecorderState) -> RecordingProgress {
    RecordingProgress {
      elapsed: state.end.as_secs_f64(),
      frames: state.frames,
      dropped_frames: state.dropped_frames,
      dropped_sound: self.tap.get_dropped(),
      ended: state.ended.clone(),
    }
  }
}
