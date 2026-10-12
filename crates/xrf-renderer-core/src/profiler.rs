//! The Tracy client a `tracy` build streams its zones to; every call does nothing without the feature.

/// Starts the client, before any thread opens a zone; call once, first thing in the process.
pub fn start_profiler() {
  #[cfg(feature = "tracy")]
  tracy_client::Client::start();
}

/// Puts a message on the capture's timeline, as a driver marks where each measured scene begins.
pub fn send_profiler_message(text: &str) {
  #[cfg(feature = "tracy")]
  if let Some(client) = tracy_client::Client::running() {
    client.message(text, 0);
  }

  #[cfg(not(feature = "tracy"))]
  let _ = text;
}
