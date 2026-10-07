/// Put a message on a Tracy capture's timeline, as a driver marks each measured scene; nothing in a build without Tracy.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "mark_profile"))]
#[tauri::command(rename = "mark_profile")]
pub fn render_mark_profile(text: String) {
  xrf_renderer::send_profiler_message(&text);
}
