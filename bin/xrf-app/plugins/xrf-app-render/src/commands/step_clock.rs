use std::time::Duration;

use tauri::State;
use xrf_app_core::TauriResult;

use crate::Renderer;

/// Draw every scene's frames `seconds` apart on its clock from the next on, however long each takes, so what moves
/// moves alike at any frame rate; none draws them at the time each is drawn again.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "step_clock")]
pub fn render_step_clock(renderer: State<'_, Renderer>, seconds: Option<f32>) -> TauriResult {
  let step: Option<Duration> = seconds
    .map(|seconds| {
      Duration::try_from_secs_f32(seconds)
        .ok()
        .filter(|step| !step.is_zero())
        .ok_or_else(|| format!("A clock steps by a positive time, not {seconds} s"))
    })
    .transpose()?;

  renderer.step_clock(step);

  Ok(())
}
