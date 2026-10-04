use serde::{Deserialize, Serialize};

/// Where the render thread's time goes a frame, mean milliseconds over a report's span, in the order it spends them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderFramePhases {
  /// Moving the cameras and weathers on, and uploading the textures that arrived.
  pub update: f32,
  /// Waiting for the window's next image.
  pub acquire: f32,
  /// Taking in what the level's workers loaded.
  pub load: f32,
  /// Readying the frame: its uniforms, culling arguments, lights, particles and water.
  pub prepare: f32,
  /// Recording the level's passes.
  pub record: f32,
  /// Recording the window's own pass, which every viewport's picture is drawn into.
  pub compose: f32,
  /// Encoding what was recorded into the graphics API's commands, which wgpu leaves until the encoder is finished.
  pub encode: f32,
  /// Handing the encoded frame to the queue.
  pub submit: f32,
  /// Presenting it.
  pub present: f32,
}
