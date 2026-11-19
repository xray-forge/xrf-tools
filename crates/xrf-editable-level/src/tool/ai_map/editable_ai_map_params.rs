/// The settings the AI map was generated with, `SAIParams` (`ESceneAIMapTools_Export.h`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditableAiMapParams {
  /// The side of one node's square cell, in metres.
  pub patch_size: f32,
  /// The height of the tester the editor walks over the scene, and how far up and down a neighbour may lie.
  pub test_height: f32,
  pub can_up: f32,
  pub can_down: f32,
}

impl Default for EditableAiMapParams {
  /// `SAIParams`' defaults.
  fn default() -> Self {
    Self {
      patch_size: 0.7,
      test_height: 1.0,
      can_up: 1.5,
      can_down: 4.0,
    }
  }
}
