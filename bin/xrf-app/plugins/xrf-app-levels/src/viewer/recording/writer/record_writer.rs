use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread::{self, JoinHandle};

use xrf_app_core::TauriResult;

use crate::viewer::recording::writer::{
  MediaProbe, MediaSession, MediaSink, RecordFormat, RecordSample, RecordSummary,
};

/// A recording written as fragmented MP4 on a thread of its own, which holds Media Foundation and the Sink Writer for
/// the recording's life: samples handed over queue a few deep, so one handed faster than the encoders take it waits.
/// Dropped unfinished, it finishes the file as far as it was handed.
pub struct RecordWriter {
  samples: SyncSender<RecordSample>,
  thread: Option<JoinHandle<TauriResult<RecordSummary>>>,
}

impl RecordWriter {
  /// Samples queued for the writer's thread before one more waits.
  const QUEUED: usize = 4;

  /// Starts writing a recording of `format` into `file`, replacing one there. A writer that cannot open leaves no file:
  /// the Sink Writer makes it before it refuses the stream.
  ///
  /// # Errors
  ///
  /// Returns an error when Media Foundation does not load or start, or the writer takes no stream of `format`.
  pub fn start(file: PathBuf, format: RecordFormat) -> TauriResult<Self> {
    MediaProbe::MEDIA_FOUNDATION.load()?;

    let (samples, received) = sync_channel::<RecordSample>(Self::QUEUED);
    let (opened, opening) = sync_channel::<TauriResult>(1);
    let written: PathBuf = file.clone();
    let thread: JoinHandle<TauriResult<RecordSummary>> = thread::Builder::new()
      .name(String::from("xrf-record"))
      .spawn(move || Self::write_all(&written, format, &received, &opened))
      .map_err(|error| format!("The recording cannot start: {error}"))?;
    let mut writer: Self = Self {
      samples,
      thread: Some(thread),
    };
    let refused: String = match opening.recv() {
      Ok(Ok(())) => return Ok(writer),
      Ok(Err(error)) => {
        let _ = Self::join(writer.thread.take());

        error
      }
      Err(_) => writer.take_error(),
    };

    let _ = std::fs::remove_file(&file);

    Err(refused)
  }

  /// Hands `sample` to the writer's thread, waiting while its queue is full.
  ///
  /// # Errors
  ///
  /// Returns why the writer stopped, where it did.
  pub fn write(&mut self, sample: RecordSample) -> TauriResult {
    self.samples.send(sample).map_err(|_| self.take_error())
  }

  /// Writes what was handed over to the file's end and closes it, answering what was written.
  ///
  /// # Errors
  ///
  /// Returns why the writer stopped, where it did, or the file could not be finished.
  pub fn finish(self) -> TauriResult<RecordSummary> {
    let Self { samples, thread } = self;

    drop(samples);

    Self::join(thread)
  }

  /// Why the writer's thread ended early, joined.
  fn take_error(&mut self) -> String {
    Self::join(self.thread.take())
      .err()
      .unwrap_or_else(|| String::from("The recording ended early"))
  }

  fn join(thread: Option<JoinHandle<TauriResult<RecordSummary>>>) -> TauriResult<RecordSummary> {
    thread
      .ok_or("The recording stopped already")?
      .join()
      .map_err(|_| String::from("The recording stopped on a panic"))?
  }

  /// The writer's thread: opens the file, telling `opened` whether it could, then writes each sample as it comes until
  /// the writer is finished or dropped, and closes the file.
  fn write_all(
    file: &Path,
    format: RecordFormat,
    samples: &Receiver<RecordSample>,
    opened: &SyncSender<TauriResult>,
  ) -> TauriResult<RecordSummary> {
    let session: MediaSession = MediaSession::start().inspect_err(|error| {
      let _ = opened.send(Err(error.clone()));
    })?;
    let mut sink: MediaSink = MediaSink::open(file, format).inspect_err(|error| {
      let _ = opened.send(Err(error.clone()));
    })?;
    let _ = opened.send(Ok(()));

    let written: TauriResult<RecordSummary> = samples
      .iter()
      .try_for_each(|sample| sink.write(sample))
      .and_then(|()| sink.finish());

    // The sink's objects are released before Media Foundation shuts down.
    drop(session);

    written
  }
}
