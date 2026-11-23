use std::collections::HashMap;

/// What the world lets of a level's lights show this frame, whatever a view sees of them: how much of each campfire's
/// idle light, and whether the lights of artefacts a detector must reveal are lit.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LightGates<'a> {
  /// By the campfire's spawned object id; a campfire missing from it shows whole.
  pub campfire_shares: &'a HashMap<u16, f32>,
  pub is_detector_hidden_lit: bool,
}
