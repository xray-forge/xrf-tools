use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread::{self, JoinHandle};

use xrf_app_core::TauriResult;

use crate::viewer::recording::writer::{
  MediaProbe, MediaSession, MediaSink, RecordFormat, RecordSample, RecordSummary,
};

/// Writes a recording as fragmented MP4 on its own thread, which holds Media Foundation and the Sink Writer for the
/// recording's life. Samples queue a few deep, so a writer faster than the encoders waits. Dropped unfinished, it
/// finishes the file with what it was handed.
pub(crate) struct RecordWriter {
  samples: SyncSender<RecordSample>,
  thread: Option<JoinHandle<TauriResult<RecordSummary>>>,
  /// Why the writer's thread stopped early, kept once a write finds it gone, for `finish` to return.
  failure: Option<String>,
}

impl RecordWriter {
  /// Samples queued before a write waits.
  const QUEUED: usize = 4;

  /// Start writing a recording of `format` into `file`, replacing any there. A failed open removes the file, which the
  /// Sink Writer creates before refusing the stream.
  ///
  /// # Errors
  ///
  /// Returns an error when Media Foundation does not load or start, or the writer takes no stream of `format`.
  pub(crate) fn start(file: PathBuf, format: RecordFormat) -> TauriResult<Self> {
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
      failure: None,
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

  /// Hand `sample` to the writer's thread, waiting while its queue is full.
  ///
  /// # Errors
  ///
  /// Returns why the writer stopped, if it did.
  pub(crate) fn write(&mut self, sample: RecordSample) -> TauriResult {
    self.samples.send(sample).map_err(|_| self.take_error())
  }

  /// Write everything handed over, close the file, and answer what was written.
  ///
  /// # Errors
  ///
  /// Returns why the writer stopped, if it did, or why the file could not be finished.
  pub(crate) fn finish(self) -> TauriResult<RecordSummary> {
    let Self {
      samples,
      thread,
      failure,
    } = self;

    drop(samples);

    match failure {
      Some(failure) => Err(failure),
      None => Self::join(thread),
    }
  }

  /// Join the early-ended writer thread and keep its error for `finish`.
  fn take_error(&mut self) -> String {
    let failure: String = Self::join(self.thread.take())
      .err()
      .unwrap_or_else(|| String::from("The recording ended early"));

    self.failure = Some(failure.clone());

    failure
  }

  fn join(thread: Option<JoinHandle<TauriResult<RecordSummary>>>) -> TauriResult<RecordSummary> {
    thread
      .ok_or("The recording stopped already")?
      .join()
      .map_err(|_| String::from("The recording stopped on a panic"))?
  }

  /// The writer thread: open the file and report it on `opened`, write each sample until the writer is finished or
  /// dropped, then close the file.
  fn write_all(
    file: &Path,
    format: RecordFormat,
    samples: &Receiver<RecordSample>,
    opened: &SyncSender<TauriResult>,
  ) -> TauriResult<RecordSummary> {
    let session: MediaSession = MediaSession::start().inspect_err(|error| {
      let _ = opened.send(Err(error.clone()));
    })?;
    // The sink is scoped to this block so its objects are released before Media Foundation shuts down, success or
    // not: a hardware encoder released after shutdown can fault.
    let written: TauriResult<RecordSummary> = {
      let mut sink: MediaSink = MediaSink::open(file, format).inspect_err(|error| {
        let _ = opened.send(Err(error.clone()));
      })?;
      let _ = opened.send(Ok(()));

      samples
        .iter()
        .try_for_each(|sample| sink.write(sample))
        .and_then(|()| sink.finish())
    };

    drop(session);

    written
  }
}
