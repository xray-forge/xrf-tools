use serde::{Deserialize, Serialize};

/// Where the render thread's time goes a frame, mean milliseconds over a report's span.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderFramePhases {
  /// Moving the cameras and weathers on, feeding the scenes what the world posted, and uploading the textures that
  /// arrived.
  pub update: f32,
  /// Waiting for the window's next image.
  pub acquire: f32,
  /// Asking for what each view draws with: its overlays, its weather's textures and the heights its parallax reads.
  pub load: f32,
  /// Readying the frame: its uniforms, culling arguments, lights, particles and water.
  pub prepare: f32,
  /// Recording the level's passes and the window's own pass, which every viewport's picture is drawn into.
  pub record: f32,
  /// Readying the window pass's pipelines for each window's image format.
  pub compose: f32,
  /// Encoding what was recorded into the graphics API's commands, which wgpu leaves until the encoder is finished.
  pub encode: f32,
  /// Handing the encoded frame to the queue.
  pub submit: f32,
  /// Presenting it.
  pub present: f32,
}
