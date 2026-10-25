use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use xrf_app_core::TauriResult;
use xrf_renderer::RenderViewportId;

/// What the render thread answers a command, waited on a blocking thread of its own rather than in the application's
/// pool, since it only waits.
pub struct RenderAnswer;

impl RenderAnswer {
  /// How long a viewport is waited on for what its render thread answers: a viewport minimised or hidden draws no frame.
  pub const TIMEOUT: Duration = Duration::from_secs(2);

  /// Waits for what the render thread answers; `missing` says what never came.
  ///
  /// # Errors
  ///
  /// Returns `missing` when nothing is answered within `timeout`, and says so apart when the render thread let the reply
  /// go unanswered: a viewport it does not draw, or a thread that stopped.
  pub async fn await_answer<T: Send + 'static>(
    answer: Receiver<T>,
    timeout: Duration,
    missing: String,
  ) -> TauriResult<T> {
    tauri::async_runtime::spawn_blocking(move || answer.recv_timeout(timeout))
      .await
      .map_err(|error| format!("{missing}: {error}"))?
      .map_err(|error| match error {
        RecvTimeoutError::Timeout => missing,
        RecvTimeoutError::Disconnected => format!("{missing}: the renderer does not draw it"),
      })
  }

  /// Waits for what `viewport`'s render thread answers within `TIMEOUT`; `what` names it where none comes.
  ///
  /// # Errors
  ///
  /// Returns an error naming the viewport and `what` when nothing is answered in time or the reply was let go.
  pub async fn await_viewport<T: Send + 'static>(
    answer: Receiver<T>,
    viewport: RenderViewportId,
    what: &str,
  ) -> TauriResult<T> {
    Self::await_answer(
      answer,
      Self::TIMEOUT,
      format!("Viewport {} answered no {what}", viewport.0),
    )
    .await
  }
}
