use std::path::Path;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use image::RgbaImage;
use xrf_app_core::TauriResult;
use xrf_job::{JobHandle, JobOutcome, JobScope};
use xrf_renderer::{CaptureEvent, CaptureShot, RenderCapture};

use crate::{CaptureFile, CaptureOutcome};

/// A capture of many shots as its job names it: the capture in its errors and its log, its shots in its progress.
pub struct CaptureNames {
  pub capture: &'static str,
  pub shots: &'static str,
}

impl CaptureNames {
  /// How long a capture waits for the renderer's next word before it looks whether it was asked to stop.
  const CANCEL_POLL: Duration = Duration::from_millis(100);

  /// How long the renderer may say nothing before a capture is taken as stalled.
  const STALL: Duration = Duration::from_secs(60);

  /// Receives the capture's shots as they come back for `job`, handing each to `place`; false where it was cancelled,
  /// which drops `events` and so stops the renderer.
  ///
  /// # Errors
  ///
  /// Returns an error where the renderer stops, stalls or goes, or where `place` refuses a shot.
  pub fn receive_shots(
    &self,
    job: &JobHandle,
    events: &Receiver<CaptureEvent>,
    mut place: impl FnMut(CaptureShot, RenderCapture) -> TauriResult<()>,
  ) -> TauriResult<bool> {
    let name: &str = self.capture;
    // Entered once the renderer says how many shots it planned.
    let mut scope: Option<JobScope> = None;
    let mut silent: Duration = Duration::ZERO;
    let mut drawn: u32 = 0;

    loop {
      if job.is_cancelled() {
        return Ok(false);
      }

      match events.recv_timeout(Self::CANCEL_POLL) {
        Ok(CaptureEvent::Progress { drawn: now, total }) => {
          silent = Duration::ZERO;
          scope
            .get_or_insert_with(|| job.enter(self.shots, Some(u64::from(total))))
            .advance_by(u64::from(now.saturating_sub(drawn)));
          drawn = drawn.max(now);
        }
        Ok(CaptureEvent::Shot { shot, capture }) => {
          silent = Duration::ZERO;
          place(shot, capture)?;
        }
        Ok(CaptureEvent::Finished) => break,
        Ok(CaptureEvent::Stopped { error }) => return Err(format!("The {name} stopped: {error}")),
        Err(RecvTimeoutError::Timeout) => {
          silent += Self::CANCEL_POLL;

          if silent >= Self::STALL {
            return Err(format!(
              "The renderer drew nothing of the {name} for {} seconds",
              Self::STALL.as_secs()
            ));
          }
        }
        Err(RecvTimeoutError::Disconnected) => {
          return Err(format!("The renderer stopped before the {name} was read back"));
        }
      }
    }

    Ok(true)
  }

  /// Writes the capture's `picture` to `path` as a PNG, encoded whole first, so a failed write leaves no partial file.
  ///
  /// # Errors
  ///
  /// Returns an error where the PNG cannot be encoded or written.
  pub fn write(&self, picture: &RgbaImage, path: &Path) -> TauriResult<CaptureOutcome> {
    CaptureFile::write_png(picture, path, self.capture)?;
    log::info!("Captured a {}: {}", self.capture, path.display());

    Ok(CaptureOutcome {
      outcome: JobOutcome::Completed,
      width: picture.width(),
      height: picture.height(),
    })
  }
}
