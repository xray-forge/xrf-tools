use std::sync::mpsc::Receiver;
use std::time::Duration;

use crate::core::types::TauriResult;

/// How long a viewport is waited on for what its render thread answers: a viewport minimised or hidden draws no frame.
pub const ANSWER_TIMEOUT: Duration = Duration::from_secs(2);

/// Waits for what the render thread answers, on a blocking thread of its own rather than in the application's pool,
/// since it only waits; `missing` says what never came.
///
/// # Errors
///
/// Returns `missing` when nothing is answered within `timeout`.
pub async fn await_answer<T: Send + 'static>(
  answer: Receiver<T>,
  timeout: Duration,
  missing: String,
) -> TauriResult<T> {
  tauri::async_runtime::spawn_blocking(move || answer.recv_timeout(timeout))
    .await
    .map_err(|error| format!("{missing}: {error}"))?
    .map_err(|_| missing)
}
