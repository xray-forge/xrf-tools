use std::collections::VecDeque;
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
use xrf_utils::format_path;

use crate::viewer::recording::writer::{RecordFormat, RecordSample, RecordSoundFormat, RecordSummary, RecordWriter};
use crate::viewer::recording::{RecordingOutcome, RecordingProgress};

/// Records a viewport and what its camera hears into one video on its own thread: presented frames and the pre-master
/// mix, aligned by the audio clock. Time zero is the first frame's present and the mixer frame the clock anchor places
/// there; each sound block goes at its own frame from it, so a block the tap dropped is a gap, not a shift.
///
/// It ends when stopped, or by itself when the renderer stops recording, the audio rate changes, or the writer refuses
/// a sample; the file is finished up to there. A failed finish leaves the file playable to where it stopped; a
/// recording with no frame leaves no file.
pub(crate) struct ViewportRecorder {
  renderer: Renderer,
  viewport: RenderViewportId,
  /// The label of the window whose page started it; the recording ends with that page.
  window: String,
  is_stopped: Arc<AtomicBool>,
  thread: Option<JoinHandle<TauriResult<RecordingOutcome>>>,
}

/// The recorder thread's inputs.
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

/// A recording's state on its thread.
#[derive(Default)]
struct RecorderState {
  zero: Option<RecordZero>,
  /// Sound tapped before the first frame arrived, up to [`ViewportRecorder::EARLY`] of it. The first frame arrives
  /// after the sound playing at its present was tapped, so this lets the sound start with the picture.
  early: VecDeque<AudioTapBlock>,
  frames: u32,
  end: Duration,
  dropped_frames: u64,
  ended: Option<String>,
}

/// A recording's time zero: the first frame's present, and the mixer frame playing then.
#[derive(Clone, Copy)]
struct RecordZero {
  presented: Instant,
  frame: u64,
}

impl ViewportRecorder {
  /// Sound the tap holds before dropping a block; the recorder drains it every frame.
  const TAP_LENGTH: Duration = Duration::from_secs(2);

  /// Sound kept from before the first frame, covering the device latency and a frame's readback.
  const EARLY: Duration = Duration::from_millis(500);

  /// How often progress is sent to the page.
  const PROGRESS_EVERY: Duration = Duration::from_millis(250);

  /// Start recording `viewport` for `window` into `file` with `server`'s mix, at `size` rounded down to even sides as
  /// H.264 needs, reporting to `progress`.
  ///
  /// # Errors
  ///
  /// Returns an error when the mix cannot be tapped, the video cannot be written, or the thread cannot start.
  pub(crate) fn start(
    renderer: &Renderer,
    server: &AudioServer,
    viewport: RenderViewportId,
    window: String,
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
        frame: RecordFrame::INTERVAL,
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
      file,
      progress,
      is_stopped: Arc::clone(&is_stopped),
    };
    let thread: JoinHandle<TauriResult<RecordingOutcome>> = thread::Builder::new()
      .name(String::from("xrf-recorder"))
      .spawn(move || inputs.run())
      .map_err(|error| format!("The recording cannot start: {error}"))?;

    Ok(Self {
      renderer: renderer.clone(),
      viewport,
      window,
      is_stopped,
      thread: Some(thread),
    })
  }

  pub(crate) fn get_window(&self) -> &str {
    &self.window
  }

  /// Whether it ended by itself and only awaits collection.
  pub(crate) fn is_ended(&self) -> bool {
    self.thread.as_ref().is_none_or(JoinHandle::is_finished)
  }

  /// Stop and finish the file, answering what was written. The renderer is told before its next command, so a
  /// recording of an idle viewport blocks no other long job and the next recording queues after it.
  ///
  /// # Errors
  ///
  /// Returns an error when the file could not be finished, or its thread stopped on a panic.
  pub(crate) fn stop(mut self) -> TauriResult<RecordingOutcome> {
    self.is_stopped.store(true, Ordering::Release);
    self.renderer.stop_record(self.viewport);

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
  /// Take frames and sound until stopped or ended, reporting progress, then finish the file.
  fn run(mut self) -> TauriResult<RecordingOutcome> {
    let mut state: RecorderState = RecorderState::default();
    let mut told: Instant = Instant::now();

    // A refused frame or sound ends the recording as a renderer stop does, so its file is still finished, or removed
    // when empty.
    while !self.is_stopped.load(Ordering::Acquire) && state.ended.is_none() {
      let received: TauriResult = match self.frames.recv_timeout(Duration::from_millis(5)) {
        Ok(RecordEvent::Frame(frame)) => self.take_frame(frame, &mut state),
        Ok(RecordEvent::Stopped { error }) => Err(error.to_string()),
        Err(RecvTimeoutError::Timeout) => Ok(()),
        Err(RecvTimeoutError::Disconnected) => Err(String::from("The renderer stopped recording")),
      };
      // Sound tapped up to the end is still written.
      let sounded: TauriResult = self.take_sound(&mut state);

      if let Err(error) = received.and(sounded) {
        state.ended = Some(error);
      }

      if told.elapsed() >= ViewportRecorder::PROGRESS_EVERY || state.ended.is_some() {
        told = Instant::now();
        let _ = self.progress.send(self.describe(&state));
      }
    }

    let finished: TauriResult<RecordSummary> = self.writer.finish();

    // A recording refused before its first frame, such as while another long job runs, leaves no empty file.
    if state.frames == 0 {
      let _ = std::fs::remove_file(&self.file);

      return Err(state.ended.unwrap_or_else(|| String::from("No frame was recorded")));
    }

    // A writer stopped mid-way leaves the fragmented file playable up to there, so the error names it.
    let summary: RecordSummary = finished.map_err(|cause| {
      format!(
        "The recording stopped early; {} holds it up to there: {cause}",
        format_path(&self.file)
      )
    })?;

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

  /// Write `frame` at its present time; the first frame sets the recording's zero.
  fn take_frame(&mut self, frame: RecordFrame, state: &mut RecorderState) -> TauriResult {
    let presented: Instant = frame.presented;
    let rate: u32 = self.tap.get_rate();
    let zero: RecordZero = *state.zero.get_or_insert_with(|| RecordZero {
      presented,
      frame: self
        .meter
        .get_anchor()
        .map_or(self.meter.get_frames(), |anchor: AudioClockAnchor| {
          anchor.get_frame(presented, rate)
        }),
    });
    let time: Duration = presented.saturating_duration_since(zero.presented);

    state.dropped_frames = frame.dropped;

    let bgra: Vec<u8> = frame.read().map_err(|error| error.to_string())?;

    self.writer.write(RecordSample::Frame {
      bgra,
      time,
      duration: RecordFrame::INTERVAL,
    })?;
    state.frames += 1;
    state.end = time + RecordFrame::INTERVAL;

    Ok(())
  }

  /// Write the tapped sound from the recording's zero on, each block at its offset from the zero frame. Before the
  /// first frame, keep the last [`ViewportRecorder::EARLY`] of it for the zero to start from.
  fn take_sound(&mut self, state: &mut RecorderState) -> TauriResult {
    if let Some(zero) = state.zero {
      while let Some(block) = state.early.pop_front() {
        self.write_sound(block, zero)?;
      }
    }

    loop {
      let block: AudioTapBlock = match self.tap.read() {
        Ok(Some(block)) => block,
        Ok(None) => return Ok(()),
        Err(end) => return Err(format!("The recording ended: {end}")),
      };

      match state.zero {
        Some(zero) => self.write_sound(block, zero)?,
        None => {
          let most: usize =
            (ViewportRecorder::EARLY.as_secs_f64() * f64::from(self.tap.get_rate())) as usize / AudioTapBlock::FRAMES;

          state.early.push_back(block);

          while state.early.len() > most.max(1) {
            state.early.pop_front();
          }
        }
      }
    }
  }

  /// Write the part of `block` from `zero`'s frame on, at its offset from it.
  fn write_sound(&mut self, block: AudioTapBlock, zero: RecordZero) -> TauriResult {
    let end: u64 = block.frame + AudioTapBlock::FRAMES as u64;

    if end <= zero.frame {
      return Ok(());
    }

    let skipped: usize = zero.frame.saturating_sub(block.frame) as usize;

    self.writer.write(RecordSample::Sound {
      samples: block.samples[skipped * 2..].to_vec(),
      at: block.frame.max(zero.frame) - zero.frame,
    })
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
